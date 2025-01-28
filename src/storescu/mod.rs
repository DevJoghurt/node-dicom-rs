use napi::bindgen_prelude::AsyncTask;
use dicom_core::{dicom_value, header::Tag, DataElement, VR};
use dicom_dictionary_std::{tags, uids};
use dicom_encoding::transfer_syntax;
use dicom_encoding::TransferSyntax;
use dicom_object::{mem::InMemDicomObject, DefaultDicomObject, StandardDataDictionary};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use tracing::{debug, error, info, warn, Level};
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
use tokio::sync::Mutex;

mod store_async;

/// DICOM C-STORE SCU
#[napi]
pub struct StoreSCU {
    /// socket address to Store SCP,
    /// optionally with AE title
    /// (example: "STORE-SCP@127.0.0.1:104")
    addr: String,
    /// the DICOM file(s) to store
    files: Vec<PathBuf>,
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
    /// File path
    file: PathBuf,
    /// Storage SOP Class UID
    sop_class_uid: String,
    /// Storage SOP Instance UID
    sop_instance_uid: String,
    /// File Transfer Syntax
    file_transfer_syntax: String,
    /// Transfer Syntax selected
    ts_selected: Option<String>,
    /// Presentation Context selected
    pc_selected: Option<dicom_ul::pdu::PresentationContextResult>,
}

#[derive(Debug, Snafu)]
enum Error {
    /// Could not initialize SCU
    Scu {
        source: Box<dicom_ul::association::client::Error>,
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
pub enum ResultStatus {
    Success,
    Error
}

#[napi(object)]
pub struct ResultObject {
    /// Transfer Syntax UID
    pub status: ResultStatus,
    pub message: String
}

pub struct StoreSCUHandler  {
    addr: String,
    files: Vec<PathBuf>,
    verbose: bool,
    message_id: u16,
    calling_ae_title: String,
    called_ae_title: Option<String>,
    max_pdu_length: u32,
    fail_first: bool,
    never_transcode: bool,
    username: Option<String>,
    password: Option<String>,
    kerberos_service_ticket: Option<String>,
    saml_assertion: Option<String>,
    jwt: Option<String>,
    concurrency: Option<u32>
  }


#[napi]
impl napi::Task for StoreSCUHandler {
  type JsValue = ();
  type Output = ();

  fn compute(&mut self) -> napi::bindgen_prelude::Result<()> {

    let args = StoreSCU {
        addr: self.addr.clone(),
        files: self.files.clone(),
        verbose: self.verbose,
        message_id: self.message_id,
        calling_ae_title: self.calling_ae_title.clone(),
        called_ae_title: self.called_ae_title.clone(),
        max_pdu_length: self.max_pdu_length,
        fail_first: self.fail_first,
        never_transcode: self.never_transcode,
        username: self.username.clone(),
        password: self.password.clone(),
        kerberos_service_ticket: self.kerberos_service_ticket.clone(),
        saml_assertion: self.saml_assertion.clone(),
        jwt: self.jwt.clone(),
        concurrency: self.concurrency
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            run_async(args).await.unwrap_or_else(|e| {
                error!("{}", Report::from_error(e));
                std::process::exit(-2);
            });
        });

    Ok(())
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::bindgen_prelude::Result<Self::JsValue> {
    Ok(output)
  }
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
    pub concurrency: Option<u32>
}

#[napi]
impl StoreSCU {

    #[napi(constructor)]
    pub fn new(options: StoreSCUOptions) -> Self {
        let files: Vec<PathBuf> = vec![];
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
        let mut concurrency: Option<u32> = None;
        if options.concurrency.is_some() {
            concurrency = Some(options.concurrency.unwrap());
        }

        tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(if verbose { Level::DEBUG } else { Level::INFO })
                .finish(),
        )
        .whatever_context("Could not set up global logging subscriber")
        .unwrap_or_else(|e: Whatever| {
            eprintln!("[ERROR] {}", Report::from_error(e));
        });

        StoreSCU {
            addr: options.addr,
            files: files,
            verbose: verbose,
            message_id: message_id,
            calling_ae_title: calling_ae_title,
            called_ae_title: options.called_ae_title,
            max_pdu_length: max_pdu_length,
            fail_first: fail_first,
            never_transcode: never_transcode,
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
        self.files.push(PathBuf::from(path));
    }

    #[napi]
    pub fn add_folder(&mut self, path: String) {
        let path = PathBuf::from(path);
        if path.is_dir() {
            for file in WalkDir::new(path.as_path())
                .into_iter()
                .filter_map(Result::ok)
                .filter(|f| !f.file_type().is_dir())
            {
                self.files.push(file.into_path());
            }
        }
    }

    #[napi]
    pub fn send(&self) -> AsyncTask<StoreSCUHandler> {
        AsyncTask::new(StoreSCUHandler {
            addr: self.addr.clone(),
            files: self.files.clone(),
            verbose: self.verbose,
            message_id: self.message_id,
            calling_ae_title: self.calling_ae_title.clone(),
            called_ae_title: self.called_ae_title.clone(),
            max_pdu_length: self.max_pdu_length,
            fail_first: self.fail_first,
            never_transcode: self.never_transcode,
            username: self.username.clone(),
            password: self.password.clone(),
            kerberos_service_ticket: self.kerberos_service_ticket.clone(),
            saml_assertion: self.saml_assertion.clone(),
            jwt: self.jwt.clone(),
            concurrency: self.concurrency
          })
    }


}

async fn run_async(args: StoreSCU) -> Result<(), Error> {
    use store_async::{get_scu, send_file};
    let StoreSCU {
        addr,
        files,
        verbose,
        message_id,
        calling_ae_title,
        called_ae_title,
        max_pdu_length,
        fail_first,
        mut never_transcode,
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
    let (dicom_files, presentation_contexts) =
        tokio::task::spawn_blocking(move || check_files(files, verbose, never_transcode))
            .await
            .unwrap();
    let num_files = dicom_files.len();
    let dicom_files = Arc::new(Mutex::new(dicom_files));
    let mut tasks = tokio::task::JoinSet::new();

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
            let mut scu = get_scu(
                addr,
                calling_ae_title,
                called_ae_title,
                max_pdu_length,
                username,
                password,
                kerberos_service_ticket,
                saml_assertion,
                jwt,
                pc,
            )
            .await?;
            loop {
                let file = {
                    let mut files = d_files.lock().await;
                    files.pop()
                };
                let mut file = match file {
                    Some(file) => file,
                    None => break,
                };
                let r: Result<_, Error> = check_presentation_contexts(
                    &file,
                    scu.presentation_contexts(),
                    never_transcode,
                );
                match r {
                    Ok((pc, ts)) => {
                        if verbose {
                            debug!(
                                "{}: Selected presentation context: {:?}",
                                file.file.display(),
                                pc
                            );
                        }
                        file.pc_selected = Some(pc);
                        file.ts_selected = Some(ts);
                    }
                    Err(e) => {
                        error!("{}", Report::from_error(e));
                        if fail_first {
                            let _ = scu.abort().await;
                            std::process::exit(-2);
                        }
                    }
                }
                scu = send_file(scu, file, message_id, pbx.as_ref(), verbose, fail_first).await?;
            }
            let _ = scu.release().await;
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

    Ok(())
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
    files: Vec<PathBuf>,
    verbose: bool,
    never_transcode: bool,
) -> (Vec<DicomFile>, HashSet<(String, String)>) {
    let mut checked_files: Vec<PathBuf> = vec![];
    let mut dicom_files: Vec<DicomFile> = vec![];
    let mut presentation_contexts = HashSet::new();

    for file in files {
        if file.is_dir() {
            for file in WalkDir::new(file.as_path())
                .into_iter()
                .filter_map(Result::ok)
                .filter(|f| !f.file_type().is_dir())
            {
                checked_files.push(file.into_path());
            }
        } else {
            checked_files.push(file);
        }
    }

    for file in checked_files {
        if verbose {
            info!("Opening file '{}'...", file.display());
        }

        match check_file(&file) {
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
            Err(_) => {
                warn!("Could not open file {} as DICOM", file.display());
            }
        }
    }

    if dicom_files.is_empty() {
        eprintln!("No supported files to transfer");
        std::process::exit(-1);
    }
    (dicom_files, presentation_contexts)
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
        file: file.to_path_buf(),
        sop_class_uid: storage_sop_class_uid.to_string(),
        sop_instance_uid: storage_sop_instance_uid.to_string(),
        file_transfer_syntax: String::from(ts.uid()),
        ts_selected: None,
        pc_selected: None,
    })
}

fn check_presentation_contexts(
    file: &DicomFile,
    pcs: &[dicom_ul::pdu::PresentationContextResult],
    never_transcode: bool,
) -> Result<(dicom_ul::pdu::PresentationContextResult, String), Error> {
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