use napi::bindgen_prelude::AsyncTask;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use snafu::Report;
use tracing::{error, info, Level};

use tokio::sync::{broadcast, Notify, Mutex};
use tokio::runtime::Runtime;

use dicom_core::{dicom_value, DataElement, VR};
use dicom_dictionary_std::tags;
use dicom_object::{InMemDicomObject, StandardDataDictionary};

mod transfer;
mod store_async;
mod s3_storage;
use store_async::run_store_async;
use s3_storage::{build_s3_bucket, check_s3_connectivity};

type EventSender = broadcast::Sender<(Event, EventData)>;
type EventReceiver = broadcast::Receiver<(Event, EventData)>;

lazy_static::lazy_static! {
    static ref EVENT_CHANNEL: (EventSender, EventReceiver) = broadcast::channel(100);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref SHUTDOWN_NOTIFY: Arc<Notify> = Arc::new(Notify::new());
}

/// Storage backend type
#[napi(string_enum)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageBackendType {
    Filesystem,
    S3,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct S3Config {
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: Option<String>,
}

/// DICOM C-STORE SCP
#[napi]
pub struct StoreSCP {
    /// Verbose mode
    // short = 'v', long = "verbose"
    verbose: bool,
    /// Calling Application Entity title
    // long = "calling-ae-title", default_value = "STORE-SCP"
    calling_ae_title: String,
    /// Enforce max pdu length
    // short = 's', long = "strict"
    strict: bool,
    /// Only accept native/uncompressed transfer syntaxes
    // long
    uncompressed_only: bool,
    /// Accept unknown SOP classes
    // long
    promiscuous: bool,
    /// Maximum PDU length
    // short = 'm', long = "max-pdu-length", default_value = "16384"
    max_pdu_length: u32,
    /// Which port to listen on
    // short, default_value = "11111"
    port: u16,
    /// Study completion callback timeout
    /// Default is 30 seconds
    study_timeout: u32,
    /// Storage backend type
    // long = "storage-backend", default_value = "Filesystem"
    storage_backend: StorageBackendType,
    /// S3 configuration if using S3 as storage backend
    s3_config: Option<S3Config>,
    /// Output directory for incoming objects using Filesystem storage backend
    // short = 'o', default_value = "."
    out_dir: Option<String>,
    /// Store files with complete DICOM file meta header (true) or dataset-only (false)
    /// Default is false (dataset-only), which is more efficient and standard for PACS systems
    store_with_file_meta: bool,
}


#[napi(string_enum)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    OnServerStarted,
    OnError,
    OnConnection,
    OnFileStored,
    OnStudyCompleted
}

#[napi(object)]
#[derive(Clone)]
pub struct EventData {
    pub message: String,
    pub data: Option<String>
}

pub struct StoreSCPServer  {
    verbose: bool,
    calling_ae_title: String,
    strict: bool,
    uncompressed_only: bool,
    promiscuous: bool,
    max_pdu_length: u32,
    out_dir: String,
    port: u16,
    study_timeout: u32,
    storage_backend: StorageBackendType,
    s3_config: Option<S3Config>,
    store_with_file_meta: bool,
}

#[napi]
impl napi::Task for StoreSCPServer {
  type JsValue = ();
  type Output = ();

  fn compute(&mut self) -> napi::bindgen_prelude::Result<()> {

    let args = StoreSCP {
      verbose: self.verbose,
      calling_ae_title: self.calling_ae_title.clone(),
      strict: self.strict,
      uncompressed_only: self.uncompressed_only,
      promiscuous: self.promiscuous,
      max_pdu_length: self.max_pdu_length,
      port: self.port,
      out_dir: Some(self.out_dir.clone()),
      study_timeout: self.study_timeout,
      storage_backend: self.storage_backend.clone(),
      s3_config: self.s3_config.clone(),
      store_with_file_meta: self.store_with_file_meta,
    };

    RUNTIME.block_on(async move {
      let server_task = RUNTIME.spawn(async move {
        run(args).await.unwrap_or_else(|e| {
          error!("{:?}", e);
          std::process::exit(-2);
        });
      });

      tokio::select! {
            _ = SHUTDOWN_NOTIFY.notified() => {
                info!("Shutting down connection task...");
                server_task.abort();
            }
            // shutdown on ctrl-c or SIGINT/SIGTERM
            _ = async { shutdown_signal().await } => {
                info!("Shutting down signal received...");
                SHUTDOWN_NOTIFY.notify_waiters();
                server_task.abort();
            }
        }
    });
    info!("Server stopped");
    Ok(())
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::bindgen_prelude::Result<Self::JsValue> {
    Ok(output)
  }
}

async fn run(args: StoreSCP) -> Result<(), Box<dyn std::error::Error>> {

  std::fs::create_dir_all(args.out_dir.as_deref().unwrap_or(".")).unwrap_or_else(|e| {
      error!("Could not create output directory: {}", e);
      std::process::exit(-2);
  });

  let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), args.port);
  let listener = tokio::net::TcpListener::bind(listen_addr).await?;
  info!(
      "{} listening on: tcp://{}",
      &args.calling_ae_title, listen_addr
  );

  StoreSCP::emit_event(Event::OnServerStarted, EventData {
      message: "Server started".to_string(),
      data: None,
  });

  let shutdown_notify = SHUTDOWN_NOTIFY.clone();

  loop {
      tokio::select! {
          _ = shutdown_notify.notified() => {
              info!("Shutting down run task...");
              break;
          }
          result = listener.accept() => {
              let (socket, _addr) = result?;
              StoreSCP::emit_event(Event::OnConnection, EventData {
                  message: "New connection".to_string(),
                  data: None,
              });

              let args = StoreSCP {
                  verbose: args.verbose,
                  calling_ae_title: args.calling_ae_title.clone(),
                  strict: args.strict,
                  uncompressed_only: args.uncompressed_only,
                  promiscuous: args.promiscuous,
                  max_pdu_length: args.max_pdu_length,
                  port: args.port,
                  out_dir: args.out_dir.clone(),
                  study_timeout: args.study_timeout,
                  storage_backend: args.storage_backend.clone(),
                  s3_config: args.s3_config.clone(),
                  store_with_file_meta: args.store_with_file_meta,
              };

              let shutdown_notify = SHUTDOWN_NOTIFY.clone();
              RUNTIME.spawn(async move {
                  tokio::select! {
                      _ = shutdown_notify.notified() => {
                          info!("Shutting down connection task...");
                      }
                      result = run_store_async(socket, &args, |data| {
                          StoreSCP::emit_event(Event::OnFileStored, EventData {
                              message: "File stored successfully".to_string(),
                              data: Some(data.to_string()),
                          });
                      }, Arc::new(Mutex::new(|data| {
                          let json_data = serde_json::json!(data);
                          StoreSCP::emit_event(Event::OnStudyCompleted, EventData {
                              message: "Study completed successfully".to_string(),
                              data: Some(json_data.to_string()),
                          });
                      }))) => {
                          if let Err(e) = result {
                              StoreSCP::emit_event(Event::OnError, EventData {
                                  message: "Error storing file".to_string(),
                                  data: Some(e.to_string()),
                              });
                              error!("{}", Report::from_error(e));
                          }
                      }
                  }
              });
          }
      }
  }

  Ok(())
}

#[napi(object)]
pub struct StoreSCPOptions {
    /// Verbose mode
    // short = 'v', long = "verbose"
    pub verbose: Option<bool>,
    /// Calling Application Entity title
    // long = "calling-ae-title", default_value = "STORE-SCP"
    pub calling_ae_title: Option<String>,
    /// Enforce max pdu length
    // short = 's', long = "strict"
    pub strict: Option<bool>,
    /// Only accept native/uncompressed transfer syntaxes
    // long
    pub uncompressed_only: Option<bool>,
    /// Accept unknown SOP classes
    // long
    pub promiscuous: Option<bool>,
    /// Maximum PDU length
    // short = 'm', long = "max-pdu-length", default_value = "16384"
    pub max_pdu_length: Option<u32>,
    /// Which port to listen on
    // short, default_value = "11111"
    pub port: u16,
    /// Study completion callback timeout
    /// Default is 30 seconds
    pub study_timeout: Option<u32>,
    /// Storage backend type
    // long = "storage-backend", default_value = "Filesystem"
    pub storage_backend: Option<StorageBackendType>,
    /// S3 configuration if using S3 as storage backend
    // long = "s3-config"
    pub s3_config: Option<S3Config>,
    /// Output directory for incoming objects using Filesystem storage backend
    // short = 'o', default_value = "."
    pub out_dir: Option<String>,
    /// Store files with complete DICOM file meta header (true) or dataset-only (false)
    /// Default is false (dataset-only), which is more efficient and standard for PACS systems
    pub store_with_file_meta: Option<bool>,
}

#[napi]
impl StoreSCP {

    #[napi(constructor)]
    pub fn new(options: StoreSCPOptions) -> Self {
        let mut verbose: bool = false;
        if options.verbose.is_some() {
            verbose = options.verbose.unwrap();
        }
        // set up global logger
        tracing::subscriber::set_global_default(
          tracing_subscriber::FmtSubscriber::builder()
              .with_max_level(if verbose {
                  Level::DEBUG
              } else {
                  Level::INFO
              })
              .finish(),
        )
        .unwrap_or_else(|e| {
            eprintln!(
                "Could not set up global logger: {}",
                snafu::Report::from_error(e)
            );
        });

        let mut calling_ae_title: String = String::from("STORE-SCP");
        if options.calling_ae_title.is_some() {
            calling_ae_title = options.calling_ae_title.unwrap();
        }
        let mut strict: bool = false;
        if options.strict.is_some() {
            strict = options.strict.unwrap();
        }
        let mut uncompressed_only: bool = false;
        if options.uncompressed_only.is_some() {
            uncompressed_only = options.uncompressed_only.unwrap();
        }
        let mut promiscuous: bool = false;
        if options.promiscuous.is_some() {
            promiscuous = options.promiscuous.unwrap();
        }
        let mut max_pdu_length: u32 = 16384;
        if options.max_pdu_length.is_some() {
            max_pdu_length = options.max_pdu_length.unwrap();
        }
        let mut study_timeout: u32 = 30;
        if options.study_timeout.is_some() {
            study_timeout = options.study_timeout.unwrap();
        }
        let storage_backend = options.storage_backend.unwrap_or(StorageBackendType::Filesystem);
        let s3_config = options.s3_config;
        let store_with_file_meta = options.store_with_file_meta.unwrap_or(false);
        StoreSCP {
            verbose: verbose,
            calling_ae_title: calling_ae_title,
            strict: strict,
            uncompressed_only: uncompressed_only,
            promiscuous: promiscuous,
            max_pdu_length: max_pdu_length,
            port: options.port,
            out_dir: options.out_dir,
            study_timeout: study_timeout,
            storage_backend,
            s3_config,
            store_with_file_meta,
        }
    }

    #[napi]
    pub fn listen(&self) -> AsyncTask<StoreSCPServer> {
        info!("Starting server...");
        if self.storage_backend == StorageBackendType::S3 {
            if let Some(ref s3_config) = self.s3_config {
                //log the bucket name and region and endpoint for S3 config
                info!("Using S3 storage backend");
                info!("S3 Bucket: {}", s3_config.bucket);
                if let Some(ref endpoint) = s3_config.endpoint {
                    info!("S3 Endpoint: {}", endpoint);
                } else {
                    info!("S3 Endpoint: Not specified");
                }
                // S3 connectivity check at server startup
                let config = s3_config.clone();
                RUNTIME.block_on(async move {
                    let bucket = build_s3_bucket(&config);
                    check_s3_connectivity(&bucket).await;
                });
            } else {
                error!("S3 storage backend selected, but no S3 config provided!");
                panic!("S3 config required for S3 backend");
            }
        } else {
            info!("Using Filesystem storage backend");
        }
        AsyncTask::new(StoreSCPServer {
          verbose: self.verbose,
          calling_ae_title: self.calling_ae_title.clone(),
          strict: self.strict,
          uncompressed_only: self.uncompressed_only,
          promiscuous: self.promiscuous,
          max_pdu_length: self.max_pdu_length,
          port: self.port,
          out_dir: self.out_dir.clone().unwrap_or_else(|| ".".to_string()),
          study_timeout: self.study_timeout,
          storage_backend: self.storage_backend.clone(),
          s3_config: self.s3_config.clone(),
          store_with_file_meta: self.store_with_file_meta,
        })
    }

    #[napi]
    pub async fn close(&self) {
        info!("Initiating shutdown...");
        SHUTDOWN_NOTIFY.notify_waiters();
        //RUNTIME.shutdown_timeout(std::time::Duration::from_secs(5));
    }

    #[napi]
    pub fn add_event_listener(&self, event: Event, handler: ThreadsafeFunction<EventData>) {
        info!("Adding event listener for {:?}", event);
        let mut receiver = EVENT_CHANNEL.0.subscribe();
        let shutdown_notify = SHUTDOWN_NOTIFY.clone();
        RUNTIME.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_notify.notified() => {
                        info!("Shutting down event listener task...");
                        break;
                    }
                    result = receiver.recv() => {
                        if let Ok((evt, data)) = result {
                            if evt == event {
                                info!("Event received: {:?}", evt);
                                handler.call(Ok(data), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }
                    }
                }
            }
        });
    }

    fn emit_event(event: Event, data: EventData) {
        let _ = EVENT_CHANNEL.0.send((event, data));
    }
}

pub(crate) fn create_cstore_response(
    message_id: u16,
    sop_class_uid: &str,
    sop_instance_uid: &str,
) -> InMemDicomObject<StandardDataDictionary> {
    InMemDicomObject::command_from_element_iter([
        DataElement::new(
            tags::AFFECTED_SOP_CLASS_UID,
            VR::UI,
            dicom_value!(Str, sop_class_uid),
        ),
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x8001])),
        DataElement::new(
            tags::MESSAGE_ID_BEING_RESPONDED_TO,
            VR::US,
            dicom_value!(U16, [message_id]),
        ),
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0101]),
        ),
        DataElement::new(tags::STATUS, VR::US, dicom_value!(U16, [0x0000])),
        DataElement::new(
            tags::AFFECTED_SOP_INSTANCE_UID,
            VR::UI,
            dicom_value!(Str, sop_instance_uid),
        ),
    ])
}

pub(crate) fn create_cecho_response(message_id: u16) -> InMemDicomObject<StandardDataDictionary> {
    InMemDicomObject::command_from_element_iter([
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x8030])),
        DataElement::new(
            tags::MESSAGE_ID_BEING_RESPONDED_TO,
            VR::US,
            dicom_value!(U16, [message_id]),
        ),
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0101]),
        ),
        DataElement::new(tags::STATUS, VR::US, dicom_value!(U16, [0x0000])),
    ])
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.unwrap() };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}