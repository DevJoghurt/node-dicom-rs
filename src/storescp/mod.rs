use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    path::PathBuf,
};

use napi::{
    bindgen_prelude::Function,
    JsError
};

use dicom_core::{dicom_value, DataElement, VR};
use dicom_dictionary_std::tags;
use dicom_encoding::transfer_syntax::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject, StandardDataDictionary};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dicom_ul::{pdu::PDataValueType, Pdu};
use snafu::{OptionExt, Report, ResultExt, Whatever};
use tracing::{debug, error, info, warn, Level};

use crate::storescp::transfer::ABSTRACT_SYNTAXES;
pub mod transfer;


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

#[napi(string_enum)]
pub enum Event {
    OnServerStarted,
    OnError,
    OnConnection,
    OnFileStored
}

#[napi(object)]
pub struct EventData {
    pub message: String,
    pub data: Option<String>
}

#[napi]
impl StoreSCP {

    #[napi(constructor)]
    pub fn new(options: StoreSCPOptions) -> Self {
        let mut verbose: bool = false;
        if options.verbose.is_some() {  
            verbose = options.verbose.unwrap();
        }
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
    pub fn listen(&self, cb: Function<(Event, EventData), ()>) -> Result<(), JsError> {

        tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(if self.verbose {
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
    
        std::fs::create_dir_all(&self.out_dir).unwrap_or_else(|e| {
            error!("Could not create output directory: {}", e);
            std::process::exit(-2);
        });
    
        let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), self.port);
        let listener = TcpListener::bind(listen_addr);
        match listener {
            Ok(l) => {
                info!(
                    "{} listening on: tcp://{}",
                    &self.calling_ae_title, listen_addr
                );
                let _ = cb.call((Event::OnServerStarted, {
                    EventData {
                        message: "StoreSCP Server started successfully".to_string(),
                        data: Some(listen_addr.to_string())
                    }
                }));
                for stream in l.incoming() {
                    match stream {
                        Ok(scu_stream) => {
                            if let Err(e) = Self::run(scu_stream, &self, &cb) {
                                error!("{}", snafu::Report::from_error(e));
                            }
                        }
                        Err(e) => {
                            error!("{}", snafu::Report::from_error(e));
                        }
                    }
                }
            }
            Err(e) => {
                error!("{}", snafu::Report::from_error(e));
                std::process::exit(-1);
            }
        }
        Ok(())
    }

    fn run(scu_stream: TcpStream, args: &StoreSCP, cb: &Function<(Event, EventData), ()>) -> Result<(), Whatever> {
        let StoreSCP {
            verbose,
            calling_ae_title,
            strict,
            uncompressed_only,
            promiscuous,
            max_pdu_length,
            out_dir,
            port: _,
        } = args;
        let verbose = *verbose;
        let out_dir = PathBuf::from(out_dir);
    
        let mut buffer: Vec<u8> = Vec::with_capacity(*max_pdu_length as usize);
        let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
        let mut msgid = 1;
        let mut sop_class_uid = "".to_string();
        let mut sop_instance_uid = "".to_string();
    
        let mut options = dicom_ul::association::ServerAssociationOptions::new()
            .accept_any()
            .ae_title(calling_ae_title)
            .strict(*strict)
            .promiscuous(*promiscuous);
    
        if *uncompressed_only {
            options = options
                .with_transfer_syntax("1.2.840.10008.1.2")
                .with_transfer_syntax("1.2.840.10008.1.2.1");
        } else {
            for ts in TransferSyntaxRegistry.iter() {
                if !ts.is_unsupported() {
                    options = options.with_transfer_syntax(ts.uid());
                }
            }
        };
    
        for uid in ABSTRACT_SYNTAXES {
            options = options.with_abstract_syntax(*uid);
        }
    
        let mut association = options
            .establish(scu_stream)
            .whatever_context("could not establish association")?;
    
        info!("New association from {}", association.client_ae_title());
        debug!(
            "> Presentation contexts: {:?}",
            association.presentation_contexts()
        );
    
        loop {
            match association.receive() {
                Ok(mut pdu) => {
                    if verbose {
                        debug!("scu ----> scp: {}", pdu.short_description());
                    }
                    match pdu {
                        Pdu::PData { ref mut data } => {
                            if data.is_empty() {
                                debug!("Ignoring empty PData PDU");
                                continue;
                            }
    
                            for data_value in data {
                                if data_value.value_type == PDataValueType::Data && !data_value.is_last
                                {
                                    instance_buffer.append(&mut data_value.data);
                                } else if data_value.value_type == PDataValueType::Command
                                    && data_value.is_last
                                {
                                    // commands are always in implict VR LE
                                    let ts =
                                        dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                            .erased();
                                    let data_value = &data_value;
                                    let v = &data_value.data;
    
                                    let obj = InMemDicomObject::read_dataset_with_ts(v.as_slice(), &ts)
                                        .whatever_context("failed to read incoming DICOM command")?;
                                    let command_field = obj
                                        .element(tags::COMMAND_FIELD)
                                        .whatever_context("Missing Command Field")?
                                        .uint16()
                                        .whatever_context("Command Field is not an integer")?;
    
                                    if command_field == 0x0030 {
                                        // Handle C-ECHO-RQ
                                        let cecho_response = create_cecho_response(msgid);
                                        let mut cecho_data = Vec::new();
    
                                        cecho_response
                                            .write_dataset_with_ts(&mut cecho_data, &ts)
                                            .whatever_context(
                                                "could not write C-ECHO response object",
                                            )?;
    
                                        let pdu_response = Pdu::PData {
                                            data: vec![dicom_ul::pdu::PDataValue {
                                                presentation_context_id: data_value
                                                    .presentation_context_id,
                                                value_type: PDataValueType::Command,
                                                is_last: true,
                                                data: cecho_data,
                                            }],
                                        };
                                        association.send(&pdu_response).whatever_context(
                                            "failed to send C-ECHO response object to SCU",
                                        )?;
                                    } else {
                                        msgid = obj
                                            .element(tags::MESSAGE_ID)
                                            .whatever_context("Missing Message ID")?
                                            .to_int()
                                            .whatever_context("Message ID is not an integer")?;
                                        sop_class_uid = obj
                                            .element(tags::AFFECTED_SOP_CLASS_UID)
                                            .whatever_context("missing Affected SOP Class UID")?
                                            .to_str()
                                            .whatever_context(
                                                "could not retrieve Affected SOP Class UID",
                                            )?
                                            .to_string();
                                        sop_instance_uid = obj
                                            .element(tags::AFFECTED_SOP_INSTANCE_UID)
                                            .whatever_context("missing Affected SOP Instance UID")?
                                            .to_str()
                                            .whatever_context(
                                                "could not retrieve Affected SOP Instance UID",
                                            )?
                                            .to_string();
                                    }
                                    instance_buffer.clear();
                                } else if data_value.value_type == PDataValueType::Data
                                    && data_value.is_last
                                {
                                    instance_buffer.append(&mut data_value.data);
    
                                    let presentation_context = association
                                        .presentation_contexts()
                                        .iter()
                                        .find(|pc| pc.id == data_value.presentation_context_id)
                                        .whatever_context("missing presentation context")?;
                                    let ts = &presentation_context.transfer_syntax;
    
                                    let obj = InMemDicomObject::read_dataset_with_ts(
                                        instance_buffer.as_slice(),
                                        TransferSyntaxRegistry.get(ts).unwrap(),
                                    )
                                    .whatever_context("failed to read DICOM data object")?;
                                    let file_meta = FileMetaTableBuilder::new()
                                        .media_storage_sop_class_uid(
                                            obj.element(tags::SOP_CLASS_UID)
                                                .whatever_context("missing SOP Class UID")?
                                                .to_str()
                                                .whatever_context("could not retrieve SOP Class UID")?,
                                        )
                                        .media_storage_sop_instance_uid(
                                            obj.element(tags::SOP_INSTANCE_UID)
                                                .whatever_context("missing SOP Instance UID")?
                                                .to_str()
                                                .whatever_context("missing SOP Instance UID")?,
                                        )
                                        .transfer_syntax(ts)
                                        .build()
                                        .whatever_context(
                                            "failed to build DICOM meta file information",
                                        )?;
                                    let file_obj = obj.with_exact_meta(file_meta);
    
                                    // write the files to the current directory with their SOPInstanceUID as filenames
                                    let mut file_path = out_dir.clone();
                                    file_path.push(
                                        sop_instance_uid.trim_end_matches('\0').to_string() + ".dcm",
                                    );
                                    file_obj
                                        .write_to_file(&file_path)
                                        .whatever_context("could not save DICOM object to file")?;
                                    info!("Stored {}", file_path.display());

                                    let _ = cb.call((Event::OnFileStored, EventData {
                                        message: "Stored file".to_string(),
                                        data: Some(file_path.display().to_string())
                                    }));
    
                                    // send C-STORE-RSP object
                                    // commands are always in implict VR LE
                                    let ts =
                                        dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                            .erased();
    
                                    let obj = create_cstore_response(
                                        msgid,
                                        &sop_class_uid,
                                        &sop_instance_uid,
                                    );
    
                                    let mut obj_data = Vec::new();
    
                                    obj.write_dataset_with_ts(&mut obj_data, &ts)
                                        .whatever_context("could not write response object")?;
    
                                    let pdu_response = Pdu::PData {
                                        data: vec![dicom_ul::pdu::PDataValue {
                                            presentation_context_id: data_value.presentation_context_id,
                                            value_type: PDataValueType::Command,
                                            is_last: true,
                                            data: obj_data,
                                        }],
                                    };
                                    association
                                        .send(&pdu_response)
                                        .whatever_context("failed to send response object to SCU")?;
                                }
                            }
                        }
                        Pdu::ReleaseRQ => {
                            buffer.clear();
                            association.send(&Pdu::ReleaseRP).unwrap_or_else(|e| {
                                warn!(
                                    "Failed to send association release message to SCU: {}",
                                    snafu::Report::from_error(e)
                                );
                            });
                            info!(
                                "Released association with {}",
                                association.client_ae_title()
                            );
                            break;
                        }
                        Pdu::AbortRQ { source } => {
                            warn!("Aborted connection from: {:?}", source);
                            break;
                        }
                        _ => {}
                    }
                }
                Err(err @ dicom_ul::association::server::Error::Receive { .. }) => {
                    let _ = cb.call((Event::OnError, EventData {
                        message: "Error".to_string(),
                        data: Some(err.to_string())
                    }));
                    if verbose {
                        info!("{}", Report::from_error(err));
                    } else {
                        info!("{}", err);
                    }
                    break;
                }
                Err(err) => {
                    let _ = cb.call((Event::OnError, EventData {
                        message: "Error".to_string(),
                        data: Some(err.to_string()),
                    }));
                    warn!("Unexpected error: {}", Report::from_error(err));
                    break;
                }
            }
        }
    
        if let Ok(peer_addr) = association.inner_stream().peer_addr() {
            info!(
                "Dropping connection with {} ({})",
                association.client_ae_title(),
                peer_addr
            );
        } else {
            info!("Dropping connection with {}", association.client_ae_title());
        }
    
        Ok(())
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