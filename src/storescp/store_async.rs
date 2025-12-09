use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dicom_core::Tag;
use dicom_dictionary_std::tags;
use dicom_encoding::transfer_syntax::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject, StandardDataDictionary};
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dicom_ul::{
    association::ServerAssociation,
    pdu::{PDataValueType, PresentationContextResultReason},
    Pdu,
};
use snafu::{OptionExt, Report, ResultExt, Whatever};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, info, warn, error};
use serde::Serialize;
use async_trait::async_trait;

use crate::storescp::{create_cecho_response, create_cstore_response, transfer::ABSTRACT_SYNTAXES, StoreSCP};
use crate::storescp::s3_storage::{build_s3_bucket, s3_put_object};

#[derive(Clone, Debug, Serialize)]
struct ClinicalData {
    patient_name: String,
    patient_id: String,
    patient_birth_date: String,
    patient_sex: String,
}

#[derive(Clone, Debug, Serialize)]
struct Study {
    study_instance_uid: String,
    clinical_data: ClinicalData,
    series: Vec<Series>,
}

#[derive(Clone, Debug, Serialize)]
struct Series {
    series_instance_uid: String,
    series_number: i64,
    body_part_examined: String,
    protocol_name: String,
    contrast_bolus_agent: String,
    instances: Vec<Instance>,
}


#[derive(Clone, Debug, Serialize)]
struct Instance {
    sop_instance_uid: String,
    instance_number: i64,
    file_path: String,
    rows: i64,
    columns: i64,
    bits_allocated: i64,
    bits_stored: i64,
    high_bit: i64,
    pixel_representation: i64,
    photometric_interpretation: String,
    planar_configuration: i64,
    pixel_aspect_ratio: String,
    pixel_spacing: String,
    lossy_image_compression: String,
}

lazy_static::lazy_static! {
    static ref STUDY_STORE: Mutex<HashMap<String, Study>> = Mutex::new(HashMap::new());
}

pub async fn run_store_async(
    scu_stream: tokio::net::TcpStream,
    args: &StoreSCP,
    on_file_stored: impl Fn(serde_json::Value) + Send + 'static,
    on_study_completed: Arc<Mutex<dyn Fn(serde_json::Value) + Send + 'static>>
) -> Result<(), Whatever> {
    let StoreSCP {
        verbose,
        calling_ae_title,
        strict,
        uncompressed_only,
        promiscuous,
        max_pdu_length,
        out_dir,
        port: _,
        study_timeout,
        storage_backend,
        s3_config,
    } = args;

    let mut options = dicom_ul::association::ServerAssociationOptions::new()
        .accept_any()
        .ae_title(calling_ae_title)
        .strict(*strict)
        .max_pdu_length(*max_pdu_length)
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

    let peer_addr = scu_stream.peer_addr().ok();
    let association = options
        .establish_async(scu_stream)
        .await
        .whatever_context("could not establish association")?;

    info!("New association from {}", association.client_ae_title());
    if *verbose {
        debug!(
            "> Presentation contexts: {:?}",
            association.presentation_contexts()
        );
    }
    debug!(
        "#accepted_presentation_contexts={}, acceptor_max_pdu_length={}, requestor_max_pdu_length={}",
        association.presentation_contexts()
            .iter()
            .filter(|pc| pc.reason == PresentationContextResultReason::Acceptance)
            .count(),
        association.acceptor_max_pdu_length(),
        association.requestor_max_pdu_length(),
    );

    let peer_title = association.client_ae_title().to_string();
    inner(
        association,
        *verbose,
        out_dir,
        *study_timeout,
        storage_backend,
        s3_config,
        args,
        on_file_stored,
        on_study_completed,
    )
    .await?;

    if let Some(peer_addr) = peer_addr {
        info!(
            "Dropping connection with {} ({})",
            peer_title,
            peer_addr
        );
    } else {
        info!("Dropping connection with {}", peer_title);
    }

    Ok(())
}

async fn inner(
    mut association: ServerAssociation<tokio::net::TcpStream>,
    verbose: bool,
    out_dir: &Option<String>,
    study_timeout: u32,
    storage_backend: &crate::storescp::StorageBackendType,
    s3_config: &Option<crate::storescp::S3Config>,
    args: &StoreSCP,
    on_file_stored: impl Fn(serde_json::Value) + Send + 'static,
    on_study_completed: Arc<Mutex<dyn Fn(serde_json::Value) + Send + 'static>>
) -> Result<(), Whatever>
{
    let study_timeout_duration = Duration::from_secs(study_timeout as u64);

    let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut msgid = 1;
    let mut sop_class_uid = "".to_string();
    let mut sop_instance_uid = "".to_string();

    // --- Storage backend selection ---
    let storage_backend: Box<dyn StorageBackend> = match storage_backend {
        crate::storescp::StorageBackendType::Filesystem => {
            Box::new(FilesystemBackend { out_dir: out_dir.clone().unwrap() })
        },
        crate::storescp::StorageBackendType::S3 => {
            let config = s3_config.clone().expect("S3 config required for S3 backend");
            let bucket = build_s3_bucket(&config);
            Box::new(S3Backend { bucket })
        },
    };

    loop {
        match association.receive().await {
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
                                // commands are always in implicit VR LE
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
                                    association.send(&pdu_response).await.whatever_context(
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
                                let transfer_syntax_uid = ts.to_string();

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

                                // read important study and series instance UIDs for saving the file
                                let study_instance_uid = obj
                                    .element(tags::STUDY_INSTANCE_UID)
                                    .whatever_context("missing STUDY INSTANCE UID")?
                                    .to_str()
                                    .whatever_context(
                                        "could not retrieve Affected STUDY INSTANCE UID",
                                    )?
                                    .to_string();
                                let series_instance_uid = obj
                                    .element(tags::SERIES_INSTANCE_UID)
                                    .whatever_context("missing SERIES INSTANCE UID")?
                                    .to_str()
                                    .whatever_context(
                                        "could not retrieve Affected SERIES INSTANCE UID",
                                    )?
                                    .to_string();

                                // write the files to the current directory with their SOPInstanceUID as filenames
                                let mut file_path = PathBuf::from(out_dir.as_ref().expect("Output directory must be set"));
                                file_path.push(study_instance_uid.to_string());
                                file_path.push(series_instance_uid.to_string());
                                file_path.push(sop_instance_uid.trim_end_matches('\0').to_string() + ".dcm");

                                let obj_for_file = obj.clone();
                                let file_obj = obj_for_file.with_exact_meta(file_meta);
                                let mut dicom_bytes = Vec::new();
                                file_obj.write_dataset_with_ts(&mut dicom_bytes, TransferSyntaxRegistry.get(ts).unwrap()).whatever_context("could not serialize DICOM object")?;
                                let storage_key = file_path.strip_prefix(std::path::Path::new(out_dir.as_ref().unwrap()))
                                    .unwrap_or(&file_path)
                                    .to_string_lossy()
                                    .replace('\\', "/");
                                storage_backend.store_file(&storage_key, &dicom_bytes).await.whatever_context("failed to store file")?;
                                info!("Stored {}", storage_key);
                                let file_path_str = match &args.storage_backend {
                                    crate::storescp::StorageBackendType::Filesystem => file_path.display().to_string(),
                                    crate::storescp::StorageBackendType::S3 => format!("s3://{}/{}", args.s3_config.as_ref().unwrap().bucket, storage_key),
                                };

                                // Extract additional metadata
                                let (clinical_data, mut series, instance) = extract_additional_metadata(
                                    &obj,
                                    sop_instance_uid.clone(),
                                    file_path_str.clone(),
                                    series_instance_uid.clone(),
                                );


                                // Emit the OnFileStored event
                                on_file_stored(
                                    serde_json::json!({
                                        "sop_class-uid": sop_class_uid.clone(),
                                        "sop_instance_uid": sop_instance_uid.clone(),
                                        "transfer_syntax_uid": transfer_syntax_uid.clone(),
                                        "study_instance_uid": study_instance_uid.clone(),
                                        "series_instance_uid": series.series_instance_uid,
                                        "series_number": series.series_number,
                                        "instance": instance
                                    })
                                );

                                // Update global store
                                {
                                    let mut store = STUDY_STORE.lock().await;
                                    let study = store.entry(study_instance_uid.clone()).or_insert_with(|| Study {
                                        study_instance_uid: study_instance_uid.clone(),
                                        clinical_data: clinical_data.clone(),
                                        series: Vec::new(),
                                    });

                                    let series_entry = study.series.iter_mut().find(|s| s.series_instance_uid == series.series_instance_uid);
                                    if let Some(series_entry) = series_entry {
                                        if !series_entry.instances.iter().any(|i| i.sop_instance_uid == instance.sop_instance_uid) {
                                            series_entry.instances.push(instance);
                                        }
                                    } else {
                                        series.instances.push(instance);
                                        study.series.push(series);
                                    }
                                }

                                // Ensure the sleep task is only created once for a study
                                {
                                    let store = STUDY_STORE.lock().await;
                                    if store.get(&study_instance_uid).is_some() {
                                        let study_instance_uid_clone = study_instance_uid.clone();
                                        let on_study_completed_clone = Arc::clone(&on_study_completed);
                                        tokio::spawn(async move {
                                            sleep(study_timeout_duration).await;
                                            let mut store = STUDY_STORE.lock().await;
                                            if let Some(study) = store.remove(&study_instance_uid_clone) {
                                                let on_study_completed = on_study_completed_clone.lock().await;
                                                let study_data = serde_json::json!(study);
                                                on_study_completed(study_data);
                                            }
                                        });
                                    }
                                }

                                // send C-STORE-RSP object
                                // commands are always in implicit VR LE
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
                                    .await
                                    .whatever_context("failed to send response object to SCU")?;
                            }
                        }
                    }
                    Pdu::ReleaseRQ => {
                        association.send(&Pdu::ReleaseRP).await.unwrap_or_else(|e| {
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
            Err(err @ dicom_ul::association::Error::ReceivePdu { .. }) => {
                if verbose {
                    info!("{}", Report::from_error(err));
                } else {
                    info!("{}", err);
                }
                break;
            }
            Err(err) => {
                warn!("Unexpected error: {}", Report::from_error(err));
                break;
            }
        }
    }

    Ok(())
}

/**
 * Helper functions to extract DICOM tags as strings or integers.
 */
fn get_str_tag(obj: &InMemDicomObject<StandardDataDictionary>, tag: Tag) -> String {
    obj.element(tag)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn get_int_tag(obj: &InMemDicomObject<StandardDataDictionary>, tag: Tag) -> i64 {
    obj.element(tag)
        .ok()
        .and_then(|e| e.to_int().ok())
        .unwrap_or(0)
}

fn extract_additional_metadata(
    obj: &InMemDicomObject<StandardDataDictionary>,
    sop_instance_uid: String,
    file_path: String,
    series_instance_uid: String,
) -> (ClinicalData, Series, Instance) {
    let clinical_data = ClinicalData {
        patient_name: get_str_tag(obj, tags::PATIENT_NAME),
        patient_id: get_str_tag(obj, tags::PATIENT_ID),
        patient_birth_date: get_str_tag(obj, tags::PATIENT_BIRTH_DATE),
        patient_sex: get_str_tag(obj, tags::PATIENT_SEX),
    };

    let instance = Instance {
        sop_instance_uid: sop_instance_uid.clone(),
        instance_number: get_int_tag(obj, tags::INSTANCE_NUMBER),
        file_path,
        rows: get_int_tag(obj, tags::ROWS),
        columns: get_int_tag(obj, tags::COLUMNS),
        bits_allocated: get_int_tag(obj, tags::BITS_ALLOCATED),
        bits_stored: get_int_tag(obj, tags::BITS_STORED),
        high_bit: get_int_tag(obj, tags::HIGH_BIT),
        pixel_representation: get_int_tag(obj, tags::PIXEL_REPRESENTATION),
        photometric_interpretation: get_str_tag(obj, tags::PHOTOMETRIC_INTERPRETATION),
        planar_configuration: get_int_tag(obj, tags::PLANAR_CONFIGURATION),
        pixel_aspect_ratio: get_str_tag(obj, tags::PIXEL_ASPECT_RATIO),
        pixel_spacing: get_str_tag(obj, tags::PIXEL_SPACING),
        lossy_image_compression: get_str_tag(obj, tags::LOSSY_IMAGE_COMPRESSION),
    };

    let series = Series {
        series_instance_uid,
        series_number: get_int_tag(obj, tags::SERIES_NUMBER),
        body_part_examined: get_str_tag(obj, tags::BODY_PART_EXAMINED),
        protocol_name: get_str_tag(obj, tags::PROTOCOL_NAME),
        contrast_bolus_agent: get_str_tag(obj, tags::CONTRAST_BOLUS_AGENT),
        instances: vec![], // Will be filled later
    };

    (clinical_data, series, instance)
}



// StorageBackend trait and implementations for Filesystem and S3
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store_file(&self, path: &str, data: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>>;
}

pub struct FilesystemBackend {
    pub out_dir: String,
}

#[async_trait]
impl StorageBackend for FilesystemBackend {
    async fn store_file(&self, path: &str, data: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let full_path = std::path::Path::new(&self.out_dir).join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full_path, data)?;
        Ok(())
    }
}

pub struct S3Backend {
    pub bucket: s3::bucket::Bucket,
}

#[async_trait]
impl StorageBackend for S3Backend {
    async fn store_file(&self, path: &str, data: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let key = path.replace("\\", "/");
        let bucket = self.bucket.clone();
        let data = data.to_vec();
        s3_put_object(&bucket, &key, &data).await.map_err(|e| {
            error!("Failed to upload file to S3: {}", e);
            Box::<dyn std::error::Error>::from(e)
        })?;
        Ok(())
    }
}