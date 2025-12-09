use napi::bindgen_prelude::AsyncTask;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use dicom_core::{dicom_value, header::Tag, DataElement, VR};
use dicom_dictionary_std::{tags, uids};
use dicom_encoding::transfer_syntax;
use dicom_encoding::TransferSyntax;
use dicom_object::{mem::InMemDicomObject, DefaultDicomObject, StandardDataDictionary};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use tracing::{error, info, warn, Level};
use indicatif::{ProgressBar, ProgressStyle};
use snafu::prelude::*;
use snafu::{Report, Whatever};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use transfer_syntax::TransferSyntaxIndex;
use walkdir::WalkDir;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

mod store_async;
mod s3_storage;

type EventSender = broadcast::Sender<(Event, EventData)>;
type EventReceiver = broadcast::Receiver<(Event, EventData)>;

lazy_static::lazy_static! {
    static ref EVENT_CHANNEL: (EventSender, EventReceiver) = broadcast::channel(100);
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct S3Config {
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: Option<String>,
}

#[napi(string_enum)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    OnTransferStarted,
    OnFileSending,
    OnFileSent,
    OnFileError,
    OnTransferCompleted,
    OnError,
}

#[napi(object)]
#[derive(Clone)]
pub struct EventData {
    pub message: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
enum FileSource {
    Local(PathBuf),
    S3(String), // S3 key/path
}

/// DICOM C-STORE SCU
#[napi]
pub struct StoreSCU {
    /// socket address to Store SCP,
    /// optionally with AE title
    /// (example: "STORE-SCP@127.0.0.1:104")
    addr: String,
    /// the DICOM file(s) to store (local or S3)
    file_sources: Vec<FileSource>,
    /// S3 configuration for reading files from S3
    s3_config: Option<S3Config>,
    /// verbose mode
    verbose: bool,
    /// the C-STORE message ID [default: 1]
    message_id: u16,
    /// the calling Application Entity title, [default: STORE-SCU]
    calling_ae_title: String,
    /// the called Application Entity title,
    /// overrides AE title in address if present [default: ANY-SCP]
    called_ae_title: Option<String>,
    /// the maximum PDU length (range 4096..=131_072) accepted by the SCU [default: 16384]
    max_pdu_length: u32,
    /// fail if not all DICOM files can be transferred
    fail_first: bool,
    /// fail file transfer if it cannot be done without transcoding
    // hide option if transcoding is disabled [default: true]
    never_transcode: bool,
    /// accept files with any SOP class UID in storage, not only those specified in the presentation contexts
    ignore_sop_class: bool,
    /// User Identity username
    username: Option<String>,
    /// User Identity password
    password: Option<String>,
    /// User Identity Kerberos service ticket
    kerberos_service_ticket: Option<String>,
    /// User Identity SAML assertion
    saml_assertion: Option<String>,
    /// User Identity JWT
    jwt: Option<String>,
    /// Dispatch these many service users to send files in parallel
    concurrency: Option<u32>
}

struct DicomFile {
    /// File source (local path or S3 key)
    source: FileSource,
    /// Storage SOP Class UID
    sop_class_uid: String,
    /// Storage SOP Instance UID
    sop_instance_uid: String,
    /// File Transfer Syntax
    file_transfer_syntax: String,
    /// Transfer Syntax selected
    ts_selected: Option<String>,
    /// Presentation Context selected
    pc_selected: Option<dicom_ul::pdu::PresentationContextNegotiated>,
    /// Reserved for future use - not used to save memory (S3 files downloaded on-demand)
    data: Option<Vec<u8>>,
}

#[derive(Debug, Snafu)]
enum Error {
    /// Could not initialize SCU
    Scu {
        source: Box<dicom_ul::association::Error>,
    },

    /// Could not construct DICOM command
    CreateCommand {
        source: Box<dicom_object::WriteError>,
    },

    /// Unsupported file transfer syntax {uid}
    UnsupportedFileTransferSyntax {
        uid: std::borrow::Cow<'static, str>,
    },

    /// Unsupported file
    FileNotSupported,

    /// Error reading a file
    ReadFilePath {
        path: String,
        source: Box<dicom_object::ReadError>,
    },
    /// No matching presentation contexts
    NoPresentationContext,
    /// No TransferSyntax
    NoNegotiatedTransferSyntax,
    /// Transcoding error
    Transcode {
        source: dicom_pixeldata::TranscodeError,
    },
    /// Error writing dicom file to buffer
    WriteDataset {
        source: Box<dicom_object::WriteError>,
    },
    ReadDataset {
        source: dicom_object::ReadError,
    },
    MissingAttribute {
        tag: Tag,
        source: dicom_object::AccessError,
    },
    ConvertField {
        tag: Tag,
        source: dicom_core::value::ConvertValueError,
    }
}

#[napi(string_enum)]
#[derive(Debug)]
pub enum ResultStatus {
    Success,
    Error
}

#[napi(object)]
#[derive(Debug)]
pub struct ResultObject {
    /// Transfer Syntax UID
    pub status: ResultStatus,
    pub message: String
}

#[napi(object)]
pub struct StoreSCUOptions {
    /// socket address to Store SCP,
    /// optionally with AE title
    /// (example: "STORE-SCP@127.0.0.1:104")
    pub addr: String,
    /// verbose mode
    pub verbose: Option<bool>,
    /// the C-STORE message ID
    pub message_id: Option<u16>,
    /// the calling Application Entity title, [default: STORE-SCU]
    pub calling_ae_title: Option<String>,
    /// the called Application Entity title,
    /// overrides AE title in address if present [default: ANY-SCP]
    pub called_ae_title: Option<String>,
    /// the maximum PDU length accepted by the SCU [default: 16384]
    pub max_pdu_length: Option<u32>,
    /// fail if not all DICOM files can be transferred
    pub fail_first: Option<bool>,
    /// fail file transfer if it cannot be done without transcoding
    // hide option if transcoding is disabled [default: true]
    pub never_transcode: Option<bool>,
    /// accept files with any SOP class UID in storage
    pub ignore_sop_class: Option<bool>,
    /// User Identity username
    pub username: Option<String>,
    /// User Identity password
    pub password: Option<String>,
    /// User Identity Kerberos service ticket
    pub kerberos_service_ticket: Option<String>,
    /// User Identity SAML assertion
    pub saml_assertion: Option<String>,
    /// User Identity JWT
    pub jwt: Option<String>,
    /// Dispatch these many service users to send files in parallel
    pub concurrency: Option<u32>,
    /// S3 configuration for reading files from S3
    pub s3_config: Option<S3Config>
}

#[napi]
impl StoreSCU {

    #[napi(constructor)]
    pub fn new(options: StoreSCUOptions) -> Self {
        let file_sources: Vec<FileSource> = vec![];
        let mut verbose: bool = false;
        if options.verbose.is_some() {
            verbose = options.verbose.unwrap();
        }
        let mut message_id: u16 = 1;
        if options.message_id.is_some() {
            message_id = options.message_id.unwrap();
        }
        let mut calling_ae_title: String = String::from("STORE-SCU");
        if options.calling_ae_title.is_some() {
            calling_ae_title = options.calling_ae_title.unwrap();
        }
        let mut max_pdu_length: u32 = 16384;
        if options.max_pdu_length.is_some() {
            max_pdu_length = options.max_pdu_length.unwrap();
        }
        let mut fail_first: bool = false;
        if options.fail_first.is_some() {
            fail_first = options.fail_first.unwrap();
        }
        let mut never_transcode: bool = true;
        if options.never_transcode.is_some() {
            never_transcode = options.never_transcode.unwrap();
        }
        let mut ignore_sop_class: bool = false;
        if options.ignore_sop_class.is_some() {
            ignore_sop_class = options.ignore_sop_class.unwrap();
        }
        let mut concurrency: Option<u32> = None;
        if options.concurrency.is_some() {
            concurrency = Some(options.concurrency.unwrap());
        }

        // Only set global logger if not already set (it can only be set once per process)
        // Use RUST_LOG env var if set, otherwise use verbose flag
        use tracing_subscriber::EnvFilter;
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                if verbose {
                    EnvFilter::new("debug")
                } else {
                    EnvFilter::new("error")
                }
            });
        
        let _ = tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_env_filter(filter)
                .finish(),
        );

        StoreSCU {
            addr: options.addr,
            file_sources: file_sources,
            s3_config: options.s3_config,
            verbose: verbose,
            message_id: message_id,
            calling_ae_title: calling_ae_title,
            called_ae_title: options.called_ae_title,
            max_pdu_length: max_pdu_length,
            fail_first: fail_first,
            never_transcode: never_transcode,
            ignore_sop_class: ignore_sop_class,
            username: options.username.or(None),
            password: options.password.or(None),
            kerberos_service_ticket: options.kerberos_service_ticket.or(None),
            saml_assertion: options.saml_assertion.or(None),
            jwt: options.jwt.or(None),
            concurrency: concurrency
        }
    }

    #[napi]
    pub fn add_file(&mut self, path: String) {
        if self.s3_config.is_some() {
            // S3 path - normalize by removing ./ prefix
            let normalized = path.trim_start_matches("./").to_string();
            self.file_sources.push(FileSource::S3(normalized));
        } else {
            // Local file
            self.file_sources.push(FileSource::Local(PathBuf::from(path)));
        }
    }

    #[napi]
    pub fn add_folder(&mut self, path: String) {
        if self.s3_config.is_some() {
            // S3 folder (prefix) - normalize and ensure trailing slash for proper prefix matching
            let mut normalized = path.trim_start_matches("./").to_string();
            if !normalized.is_empty() && !normalized.ends_with('/') {
                normalized.push('/');
            }
            self.file_sources.push(FileSource::S3(normalized));
        } else {
            // Local folder
            let path = PathBuf::from(path);
            if path.is_dir() {
                for file in WalkDir::new(path.as_path())
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|f| !f.file_type().is_dir())
                {
                    self.file_sources.push(FileSource::Local(file.into_path()));
                }
            }
        }
    }

    #[napi]
    pub fn send(&self) -> AsyncTask<StoreSCUHandler> {
        AsyncTask::new(StoreSCUHandler {
            addr: self.addr.clone(),
            file_sources: self.file_sources.clone(),
            s3_config: self.s3_config.clone(),
            verbose: self.verbose,
            message_id: self.message_id,
            calling_ae_title: self.calling_ae_title.clone(),
            called_ae_title: self.called_ae_title.clone(),
            max_pdu_length: self.max_pdu_length,
            fail_first: self.fail_first,
            never_transcode: self.never_transcode,
            ignore_sop_class: self.ignore_sop_class,
            username: self.username.clone(),
            password: self.password.clone(),
            kerberos_service_ticket: self.kerberos_service_ticket.clone(),
            saml_assertion: self.saml_assertion.clone(),
            jwt: self.jwt.clone(),
            concurrency: self.concurrency,
        })
    }

    #[napi]
    pub fn add_event_listener(&self, event: Event, handler: ThreadsafeFunction<EventData>) {
        let mut rx = EVENT_CHANNEL.0.subscribe();
        std::thread::spawn(move || loop {
            match rx.blocking_recv() {
                Ok((ev, data)) => {
                    if ev == event {
                        handler.call(Ok(data), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                Err(_) => break,
            }
        });
    }

    pub(crate) fn emit_event(event: Event, data: EventData) {
        let _ = EVENT_CHANNEL.0.send((event, data));
    }
}

pub struct StoreSCUHandler {
    addr: String,
    file_sources: Vec<FileSource>,
    s3_config: Option<S3Config>,
    verbose: bool,
    message_id: u16,
    calling_ae_title: String,
    called_ae_title: Option<String>,
    max_pdu_length: u32,
    fail_first: bool,
    never_transcode: bool,
    ignore_sop_class: bool,
    username: Option<String>,
    password: Option<String>,
    kerberos_service_ticket: Option<String>,
    saml_assertion: Option<String>,
    jwt: Option<String>,
    concurrency: Option<u32>,
}

#[napi]
impl napi::Task for StoreSCUHandler {
    type JsValue = Vec<ResultObject>;
    type Output = Vec<ResultObject>;

    fn compute(&mut self) -> napi::bindgen_prelude::Result<Self::Output> {
        let args = StoreSCU {
            addr: self.addr.clone(),
            file_sources: self.file_sources.clone(),
            s3_config: self.s3_config.clone(),
            verbose: self.verbose,
            message_id: self.message_id,
            calling_ae_title: self.calling_ae_title.clone(),
            called_ae_title: self.called_ae_title.clone(),
            max_pdu_length: self.max_pdu_length,
            fail_first: self.fail_first,
            never_transcode: self.never_transcode,
            ignore_sop_class: self.ignore_sop_class,
            username: self.username.clone(),
            password: self.password.clone(),
            kerberos_service_ticket: self.kerberos_service_ticket.clone(),
            saml_assertion: self.saml_assertion.clone(),
            jwt: self.jwt.clone(),
            concurrency: self.concurrency,
        };

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            run_async(args).await.map_err(|e| {
                napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("Failed to send files: {}", e),
                )
            })
        })
    }

    fn resolve(
        &mut self,
        _env: napi::Env,
        output: Self::Output,
    ) -> napi::bindgen_prelude::Result<Self::JsValue> {
        Ok(output)
    }
}

async fn run_async(args: StoreSCU) -> Result<Vec<ResultObject>, Error> {
    use dicom_ul::ClientAssociationOptions;
    let StoreSCU {
        addr,
        file_sources,
        s3_config,
        verbose,
        message_id: _,
        calling_ae_title,
        called_ae_title,
        max_pdu_length,
        fail_first,
        mut never_transcode,
        ignore_sop_class,
        username,
        password,
        kerberos_service_ticket,
        saml_assertion,
        jwt,
        concurrency,
    } = args;

    // never transcode if the feature is disabled
    if cfg!(not(feature = "transcode")) {
        never_transcode = true;
    }

    if verbose {
        info!("Establishing association with '{}'...", &addr);
    }
    
    // Expand S3 folders if needed
    let expanded_sources = if s3_config.is_some() {
        expand_s3_sources(file_sources, &s3_config).await?
    } else {
        file_sources
    };
    
    // Clone s3_config for check_files (will be moved into blocking task)
    let s3_config_clone = s3_config.clone();
    
    let (dicom_files, presentation_contexts) =
        tokio::task::spawn_blocking(move || check_files(expanded_sources, s3_config_clone, verbose, never_transcode))
            .await
            .unwrap();
    let num_files = dicom_files.len();
    let dicom_files = Arc::new(Mutex::new(dicom_files));
    let mut tasks = tokio::task::JoinSet::new();
    
    // Setup S3 bucket once and share it across all tasks (memory-efficient)
    let s3_bucket = s3_config.as_ref().map(|config| Arc::new(s3_storage::build_s3_bucket(config)));

    let progress_bar;
    if !verbose {
        progress_bar = Some(Arc::new(Mutex::new(ProgressBar::new(num_files as u64))));
        if let Some(pb) = progress_bar.as_ref() {
            let bar = pb.lock().await;
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40} {pos}/{len} {wide_msg}")
                    .expect("Invalid progress bar template"),
            );
            bar.enable_steady_tick(Duration::new(0, 480_000_000));
        };
    } else {
        progress_bar = None;
    }

    for _ in 0..concurrency.unwrap_or(1) {
        let pbx = progress_bar.clone();
        let d_files = dicom_files.clone();
        let s3_bucket = s3_bucket.clone();
        let pc = presentation_contexts.clone();
        let addr = addr.clone();
        let jwt = jwt.clone();
        let saml_assertion = saml_assertion.clone();
        let kerberos_service_ticket = kerberos_service_ticket.clone();
        let username = username.clone();
        let password = password.clone();
        let called_ae_title = called_ae_title.clone();
        let calling_ae_title = calling_ae_title.clone();
        tasks.spawn(async move {
            // Emit OnTransferStarted event before moves
            StoreSCU::emit_event(Event::OnTransferStarted, EventData {
                message: "Transfer started".to_string(),
                data: Some(serde_json::json!({
                    "address": &addr,
                    "callingAeTitle": &calling_ae_title,
                    "totalFiles": num_files,
                }).to_string()),
            });
            
            let mut scu_init = ClientAssociationOptions::new()
                .calling_ae_title(calling_ae_title)
                .max_pdu_length(max_pdu_length);

            for (storage_sop_class_uid, transfer_syntax) in &pc {
                scu_init = scu_init.with_presentation_context(storage_sop_class_uid, vec![transfer_syntax]);
            }

            if let Some(called_ae_title) = called_ae_title {
                scu_init = scu_init.called_ae_title(called_ae_title);
            }

            if let Some(username) = username {
                scu_init = scu_init.username(username);
            }

            if let Some(password) = password {
                scu_init = scu_init.password(password);
            }

            if let Some(kerberos_service_ticket) = kerberos_service_ticket {
                scu_init = scu_init.kerberos_service_ticket(kerberos_service_ticket);
            }

            if let Some(saml_assertion) = saml_assertion {
                scu_init = scu_init.saml_assertion(saml_assertion);
            }

            if let Some(jwt) = jwt {
                scu_init = scu_init.jwt(jwt);
            }

            let scu = scu_init
                .establish_with_async(&addr)
                .await
                .map_err(Box::from)
                .context(ScuSnafu)?;

            store_async::inner(
                scu,
                d_files,
                s3_bucket,
                pbx.as_ref(),
                fail_first,
                verbose,
                never_transcode,
                ignore_sop_class,
            )
            .await?;

            Ok::<(), Error>(())
        });
    }
    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result {
            error!("{}", Report::from_error(e));
            if fail_first {
                std::process::exit(-2);
            }
        }
    }

    if let Some(pb) = progress_bar {
        pb.lock().await.finish_with_message("done")
    };

    // Emit OnTransferCompleted event
    StoreSCU::emit_event(Event::OnTransferCompleted, EventData {
        message: "All files transferred successfully".to_string(),
        data: Some(serde_json::json!({
            "totalFiles": num_files,
            "status": "completed",
        }).to_string()),
    });

    Ok(vec![ResultObject {
        status: ResultStatus::Success,
        message: "All files sent successfully".to_string(),
    }])
}

async fn expand_s3_sources(
    sources: Vec<FileSource>,
    s3_config: &Option<S3Config>,
) -> Result<Vec<FileSource>, Error> {
    if s3_config.is_none() {
        return Ok(sources);
    }
    
    let config = s3_config.as_ref().unwrap();
    let bucket = s3_storage::build_s3_bucket(config);
    
    let mut expanded = Vec::new();
    
    for source in sources {
        match source {
            FileSource::Local(path) => expanded.push(FileSource::Local(path)),
            FileSource::S3(key) => {
                // List all objects with this prefix (recursively)
                let objects = s3_storage::s3_list_objects(&bucket, &key)
                    .await
                    .map_err(|e| {
                        Error::ReadFilePath {
                            path: format!("s3://{}", key),
                            source: Box::new(dicom_object::ReadError::ReadFile {
                                filename: format!("s3://{}", key).into(),
                                source: std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Failed to list S3 objects: {}", e),
                                ),
                                backtrace: std::backtrace::Backtrace::capture(),
                            }),
                        }
                    })?;
                
                if objects.is_empty() {
                    // No objects found
                    continue;
                }
                
                // Filter to only include actual files (not folder markers)
                // S3 folder markers either end with '/' or have no file extension
                for obj_key in objects {
                    // Skip folder markers (keys ending with /)
                    if obj_key.ends_with('/') {
                        continue;
                    }
                    
                    // Only include files that look like actual files (have an extension or are explicitly files)
                    // This helps avoid folder-like keys in S3
                    if obj_key.contains('.') || !obj_key.ends_with('/') {
                        expanded.push(FileSource::S3(obj_key));
                    }
                }
            }
        }
    }
    
    Ok(expanded)
}

fn store_req_command(
    storage_sop_class_uid: &str,
    storage_sop_instance_uid: &str,
    message_id: u16,
) -> InMemDicomObject<StandardDataDictionary> {
    InMemDicomObject::command_from_element_iter([
        // SOP Class UID
        DataElement::new(
            tags::AFFECTED_SOP_CLASS_UID,
            VR::UI,
            dicom_value!(Str, storage_sop_class_uid),
        ),
        // command field
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x0001])),
        // message ID
        DataElement::new(tags::MESSAGE_ID, VR::US, dicom_value!(U16, [message_id])),
        //priority
        DataElement::new(tags::PRIORITY, VR::US, dicom_value!(U16, [0x0000])),
        // data set type
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0000]),
        ),
        // affected SOP Instance UID
        DataElement::new(
            tags::AFFECTED_SOP_INSTANCE_UID,
            VR::UI,
            dicom_value!(Str, storage_sop_instance_uid),
        ),
    ])
}


fn check_files(
    sources: Vec<FileSource>,
    s3_config: Option<S3Config>,
    verbose: bool,
    never_transcode: bool,
) -> (Vec<DicomFile>, HashSet<(String, String)>) {
    let mut dicom_files: Vec<DicomFile> = vec![];
    let mut presentation_contexts = HashSet::new();

    // Setup S3 bucket if needed
    let bucket = s3_config.as_ref().map(|config| s3_storage::build_s3_bucket(config));

    for source in sources {
        let display_name = match &source {
            FileSource::Local(path) => path.display().to_string(),
            FileSource::S3(key) => format!("s3://{}", key),
        };

        if verbose {
            info!("Checking file '{}'...", display_name);
        }

        match check_file_source(&source, bucket.as_ref()) {
            Ok(dicom_file) => {
                presentation_contexts.insert((
                    dicom_file.sop_class_uid.to_string(),
                    dicom_file.file_transfer_syntax.clone(),
                ));

                // also accept uncompressed transfer syntaxes
                // as mandated by the standard
                // (though it might not always be able to fulfill this)
                if !never_transcode {
                    presentation_contexts.insert((
                        dicom_file.sop_class_uid.to_string(),
                        uids::EXPLICIT_VR_LITTLE_ENDIAN.to_string(),
                    ));
                    presentation_contexts.insert((
                        dicom_file.sop_class_uid.to_string(),
                        uids::IMPLICIT_VR_LITTLE_ENDIAN.to_string(),
                    ));
                }

                dicom_files.push(dicom_file);
            }
            Err(e) => {
                warn!("Could not open {} as DICOM: {:?}", display_name, e);
            }
        }
    }

    if dicom_files.is_empty() {
        eprintln!("No supported files to transfer");
        std::process::exit(-1);
    }
    (dicom_files, presentation_contexts)
}

fn check_file_source(source: &FileSource, bucket: Option<&s3::Bucket>) -> Result<DicomFile, Error> {
    match source {
        FileSource::Local(path) => check_file(path),
        FileSource::S3(key) => check_s3_file(key, bucket.unwrap()),
    }
}

fn check_s3_file(key: &str, bucket: &s3::Bucket) -> Result<DicomFile, Error> {
    // Download file data from S3 temporarily to read metadata
    let rt = tokio::runtime::Handle::current();
    let data = rt.block_on(async {
        s3_storage::s3_get_object(bucket, key).await
    }).map_err(|e| {
        Error::ReadFilePath {
            path: format!("s3://{}", key),
            source: Box::new(dicom_object::ReadError::ReadFile {
                filename: format!("s3://{}", key).into(),
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to download S3 object: {}", e),
                ),
                backtrace: std::backtrace::Backtrace::capture(),
            }),
        }
    })?;

    // Auto-detect file format by checking for DICM magic bytes at position 128
    // Full DICOM files have: 128-byte preamble + "DICM" (4 bytes) + meta header
    // Dataset-only files start directly with DICOM data elements
    let has_dicm_magic = data.len() > 132 && &data[128..132] == b"DICM";
    
    let (storage_sop_class_uid, storage_sop_instance_uid, transfer_syntax_uid) = 
        if has_dicm_magic {
            // Full DICOM file with meta header
            let dicom_file = dicom_object::from_reader(&data[..])
                .map_err(Box::from)
                .context(ReadFilePathSnafu {
                    path: format!("s3://{}", key),
                })?;
            let meta = dicom_file.meta();
            (
                meta.media_storage_sop_class_uid.clone(),
                meta.media_storage_sop_instance_uid.clone(),
                meta.transfer_syntax.trim_end_matches('\0').to_string(),
            )
        } else {
                // Dataset-only file (no meta header) - read as InMemDicomObject
                use dicom_dictionary_std::tags;
                use dicom_encoding::TransferSyntaxIndex;
                
                // Try reading as dataset with explicit VR little endian (most common)
                let obj = InMemDicomObject::read_dataset_with_ts(
                    &data[..],
                    &dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.erased(),
                )
                .or_else(|_| {
                    // Fall back to implicit VR little endian
                    InMemDicomObject::read_dataset_with_ts(
                        &data[..],
                        &dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN.erased(),
                    )
                })
                .map_err(Box::from)
                .context(ReadFilePathSnafu {
                    path: format!("s3://{}", key),
                })?;
                
                // Extract metadata from dataset attributes
                let sop_class_uid = obj
                    .element(tags::SOP_CLASS_UID)
                    .context(MissingAttributeSnafu { tag: tags::SOP_CLASS_UID })?
                    .to_str()
                    .context(ConvertFieldSnafu { tag: tags::SOP_CLASS_UID })?
                    .to_string();
                    
                let sop_instance_uid = obj
                    .element(tags::SOP_INSTANCE_UID)
                    .context(MissingAttributeSnafu { tag: tags::SOP_INSTANCE_UID })?
                    .to_str()
                    .context(ConvertFieldSnafu { tag: tags::SOP_INSTANCE_UID })?
                    .to_string();
                
                // Assume explicit VR little endian as transfer syntax for dataset-only files
                let transfer_syntax = uids::EXPLICIT_VR_LITTLE_ENDIAN.to_string();
                
                (sop_class_uid, sop_instance_uid, transfer_syntax)
        };
    
    let transfer_syntax_uid = transfer_syntax_uid.trim_end_matches('\0');
    let ts = TransferSyntaxRegistry
        .get(transfer_syntax_uid)
        .with_context(|| UnsupportedFileTransferSyntaxSnafu {
            uid: transfer_syntax_uid.to_string(),
        })?;
    
    // Don't cache data - we'll download again during send to save memory
    Ok(DicomFile {
        source: FileSource::S3(key.to_string()),
        sop_class_uid: storage_sop_class_uid.to_string(),
        sop_instance_uid: storage_sop_instance_uid.to_string(),
        file_transfer_syntax: String::from(ts.uid()),
        ts_selected: None,
        pc_selected: None,
        data: None,
    })
}

fn check_file(file: &Path) -> Result<DicomFile, Error> {
    // Ignore DICOMDIR files until better support is added
    let _ = (file.file_name() != Some(OsStr::new("DICOMDIR")))
        .then_some(false)
        .context(FileNotSupportedSnafu)?;
    let dicom_file = dicom_object::OpenFileOptions::new()
        .read_until(Tag(0x0001, 0x000))
        .open_file(file)
        .map_err(Box::from)
        .context(ReadFilePathSnafu {
            path: file.display().to_string(),
        })?;

    let meta = dicom_file.meta();

    let storage_sop_class_uid = &meta.media_storage_sop_class_uid;
    let storage_sop_instance_uid = &meta.media_storage_sop_instance_uid;
    let transfer_syntax_uid = &meta.transfer_syntax.trim_end_matches('\0');
    let ts = TransferSyntaxRegistry
        .get(transfer_syntax_uid)
        .with_context(|| UnsupportedFileTransferSyntaxSnafu {
            uid: transfer_syntax_uid.to_string(),
        })?;
    Ok(DicomFile {
        source: FileSource::Local(file.to_path_buf()),
        sop_class_uid: storage_sop_class_uid.to_string(),
        sop_instance_uid: storage_sop_instance_uid.to_string(),
        file_transfer_syntax: String::from(ts.uid()),
        ts_selected: None,
        pc_selected: None,
        data: None,
    })
}

fn check_presentation_contexts(
    file: &DicomFile,
    pcs: &[dicom_ul::pdu::PresentationContextNegotiated],
    ignore_sop_class: bool,
    never_transcode: bool,
) -> Result<(dicom_ul::pdu::PresentationContextNegotiated, String), Error> {
    let file_ts = TransferSyntaxRegistry
        .get(&file.file_transfer_syntax)
        .with_context(|| UnsupportedFileTransferSyntaxSnafu {
            uid: file.file_transfer_syntax.to_string(),
        })?;

    // Try to find an exact match for the file's transfer syntax first
    let exact_match_pc = pcs.iter().find(|pc| pc.transfer_syntax == file_ts.uid());

    if let Some(pc) = exact_match_pc {
        return Ok((pc.clone(), pc.transfer_syntax.clone()));
    }

    let pc = pcs.iter().find(|pc| {
        if !ignore_sop_class && pc.abstract_syntax != file.sop_class_uid {
            return false;
        }
        // Check support for this transfer syntax.
        // If it is the same as the file, we're good.
        // Otherwise, uncompressed data set encoding
        // and native pixel data is required on both ends.
        let ts = &pc.transfer_syntax;
        ts == file_ts.uid()
            || TransferSyntaxRegistry
                .get(&pc.transfer_syntax)
                .filter(|ts| file_ts.is_codec_free() && ts.is_codec_free())
                .map(|_| true)
                .unwrap_or(false)
    });

    let pc = match pc {
        Some(pc) => pc,
        None => {
            if never_transcode || !file_ts.can_decode_all() {
                NoPresentationContextSnafu.fail()?
            }

            // Else, if transcoding is possible, we go for it.
            pcs.iter()
                .filter(|pc| ignore_sop_class || pc.abstract_syntax == file.sop_class_uid)
                // accept explicit VR little endian
                .find(|pc| pc.transfer_syntax == uids::EXPLICIT_VR_LITTLE_ENDIAN)
                .or_else(||
                // accept implicit VR little endian
                pcs.iter()
                    .find(|pc| pc.transfer_syntax == uids::IMPLICIT_VR_LITTLE_ENDIAN))
                .context(NoPresentationContextSnafu)?
        }
    };

    let ts = TransferSyntaxRegistry
        .get(&pc.transfer_syntax)
        .context(NoNegotiatedTransferSyntaxSnafu)?;

    Ok((pc.clone(), String::from(ts.uid())))
}

// transcoding functions

#[cfg(feature = "transcode")]
fn into_ts(
    dicom_file: DefaultDicomObject,
    ts_selected: &TransferSyntax,
    verbose: bool,
) -> Result<DefaultDicomObject, Error> {
    if ts_selected.uid() != dicom_file.meta().transfer_syntax() {
        use dicom_pixeldata::Transcode;
        let mut file = dicom_file;
        if verbose {
            info!(
                "Transcoding file from {} to {}",
                file.meta().transfer_syntax(),
                ts_selected.uid()
            );
        }
        file.transcode(ts_selected).context(TranscodeSnafu)?;
        Ok(file)
    } else {
        Ok(dicom_file)
    }
}

#[cfg(not(feature = "transcode"))]
fn into_ts(
    dicom_file: DefaultDicomObject,
    ts_selected: &TransferSyntax,
    _verbose: bool,
) -> Result<DefaultDicomObject, Error> {
    if ts_selected.uid() != dicom_file.meta().transfer_syntax() {
        panic!("Transcoding feature is disabled, should not have tried to transcode")
    } else {
        Ok(dicom_file)
    }
}