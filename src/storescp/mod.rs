use napi::bindgen_prelude::AsyncTask;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use dicom_core::{dicom_value, DataElement, VR};
use dicom_dictionary_std::tags;
use dicom_object::{InMemDicomObject, StandardDataDictionary};
use snafu::Report;
use tracing::{error, info, Level};

use tokio::sync::{broadcast, Notify, watch};
use tokio::runtime::Runtime;

mod transfer;
mod store_async;
use store_async::run_store_async;

type EventSender = broadcast::Sender<(Event, EventData)>;
type EventReceiver = broadcast::Receiver<(Event, EventData)>;

lazy_static::lazy_static! {
    static ref EVENT_CHANNEL: (EventSender, EventReceiver) = broadcast::channel(100);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref SHUTDOWN_NOTIFY: Arc<Notify> = Arc::new(Notify::new());
    static ref SHUTDOWN_SIGNAL: (watch::Sender<bool>, watch::Receiver<bool>) = watch::channel(false);
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
    /// Output directory for incoming objects
    // short = 'o', default_value = "."
    out_dir: String,
    /// Which port to listen on
    // short, default_value = "11111"
    port: u16,
}


#[napi(string_enum)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    OnServerStarted,
    OnError,
    OnConnection,
    OnFileStored
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
  port: u16
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
      out_dir: self.out_dir.clone()
    };

    RUNTIME.block_on(async move {
      let server_task = RUNTIME.spawn(async move {
        run(args).await.unwrap_or_else(|e| {
          error!("{:?}", e);
          std::process::exit(-2);
        });
      });
      // shutdown on ctrl-c
      tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down...");
            SHUTDOWN_NOTIFY.notify_waiters();
            SHUTDOWN_SIGNAL.0.send(true).unwrap();
        }
        _ = SHUTDOWN_NOTIFY.notified() => {
            info!("Received shutdown signal...");
            SHUTDOWN_SIGNAL.0.send(true).unwrap();
        }
      }

      server_task.await.unwrap();
      SHUTDOWN_NOTIFY.notify_waiters();
    });

    Ok(())
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::bindgen_prelude::Result<Self::JsValue> {
    Ok(output)
  }
}

async fn run(args: StoreSCP) -> Result<(), Box<dyn std::error::Error>> {

  std::fs::create_dir_all(&args.out_dir).unwrap_or_else(|e| {
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

  let mut shutdown_signal = SHUTDOWN_SIGNAL.1.clone();

  loop {
      tokio::select! {
          _ = shutdown_signal.changed() => {
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
              };

              let mut shutdown_signal = SHUTDOWN_SIGNAL.1.clone();
              RUNTIME.spawn(async move {
                  tokio::select! {
                      _ = shutdown_signal.changed() => {
                          info!("Shutting down connection task...");
                      }
                      result = run_store_async(socket, &args) => {
                          if let Err(e) = result {
                              StoreSCP::emit_event(Event::OnError, EventData {
                                  message: "Error storing file".to_string(),
                                  data: Some(e.to_string()),
                              });
                              error!("{}", Report::from_error(e));
                          } else {
                              let (sop_class_uid, sop_instance_uid, transfer_syntax_uid, study_instance_uid, series_instance_uid, file_path, additional_metadata) = result.unwrap();
                              let json_data = serde_json::json!({
                                  "sop_class_uid": sop_class_uid,
                                  "sop_instance_uid": sop_instance_uid,
                                  "study_instance_uid": study_instance_uid,
                                  "series_instance_uid": series_instance_uid,
                                  "transfer_syntax_uid": transfer_syntax_uid,
                                  "file_path": file_path,
                                  "metadata": additional_metadata
                              });
                              StoreSCP::emit_event(Event::OnFileStored, EventData {
                                  message: "File stored successfully".to_string(),
                                  data: Some(json_data.to_string()),
                              });
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
    /// Output directory for incoming objects
    // short = 'o', default_value = "."
    pub out_dir: String,
    /// Which port to listen on
    // short, default_value = "11111"
    pub port: u16,
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
        StoreSCP {
            verbose: verbose,
            calling_ae_title: calling_ae_title,
            strict: strict,
            uncompressed_only: uncompressed_only,
            promiscuous: promiscuous,
            max_pdu_length: max_pdu_length,
            port: options.port,
            out_dir: options.out_dir,
        }
    }

    #[napi]
    pub fn listen(&self) -> AsyncTask<StoreSCPServer> {
        info!("Starting server...");
        AsyncTask::new(StoreSCPServer {
          verbose: self.verbose,
          calling_ae_title: self.calling_ae_title.clone(),
          strict: self.strict,
          uncompressed_only: self.uncompressed_only,
          promiscuous: self.promiscuous,
          max_pdu_length: self.max_pdu_length,
          port: self.port,
          out_dir: self.out_dir.clone()
        })
    }

    #[napi]
    pub fn close(&self) {
        info!("Initiating shutdown...");
        SHUTDOWN_NOTIFY.notify_waiters();
        SHUTDOWN_SIGNAL.0.send(true).unwrap();
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
                            info!("Event received: {:?}", evt);
                            if evt == event {
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


fn create_cstore_response(
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

fn create_cecho_response(message_id: u16) -> InMemDicomObject<StandardDataDictionary> {
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