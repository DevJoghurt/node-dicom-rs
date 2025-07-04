use dicom_dictionary_std::tags;
use dicom_core::{dicom_value, DataElement, VR, Tag};
use dicom_object::{InMemDicomObject, StandardDataDictionary, FileMetaTableBuilder};
use dicom_encoding::transfer_syntax::TransferSyntaxIndex;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dicom_ul::{pdu::PDataValueType, Pdu};
use snafu::{OptionExt, Report, ResultExt, Whatever};
use std::path::PathBuf;
use tracing::{debug, info, warn, error};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;
use serde::Serialize;

use crate::storescp::{transfer::ABSTRACT_SYNTAXES, StoreSCP};

#[derive(Clone, Debug, Serialize)]
struct ClinicalData {
    patient_name: String,
    patient_id: String,
    patient_birth_date: String,
    patient_sex: String,
}

#[derive(Clone, Debug, Serialize)]
struct SeriesData {
    series_number: i64,
    modality: String,
    body_part_examined: String,
    protocol_name: String,
    contrast_bolus_agent: String
}

#[derive(Clone, Debug, Serialize)]
struct InstanceData {
    instance_number: i64,
    instance_sop_class_uid: String,
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
        study_timeout: _
    } = args;
    let verbose = *verbose;

    let study_timeout_duration = Duration::from_secs(args.study_timeout as u64); // Configurable timeout

    let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut msgid = 1;
    let mut sop_class_uid = "".to_string();
    let mut sop_instance_uid = "".to_string();

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

    let mut association = options
        .establish_async(scu_stream)
        .await
        .whatever_context("could not establish association")?;

    info!("New association from {}", association.client_ae_title());
    debug!(
        "> Presentation contexts: {:?}",
        association.presentation_contexts()
    );

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

                                // Extract additional metadata
                                let (clinical_data, series_data, instance_data) = extract_additional_metadata(&obj);

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
                                let mut file_path = PathBuf::from(out_dir);
                                file_path.push(study_instance_uid.to_string());
                                file_path.push(series_instance_uid.to_string());

                                std::fs::create_dir_all(&file_path).unwrap_or_else(|e| {
                                    error!("Could not create directory: {}", e);
                                    std::process::exit(-2);
                                });

                                file_path.push(sop_instance_uid.trim_end_matches('\0').to_string() + ".dcm");
                                let file_obj = obj.with_exact_meta(file_meta);
                                file_obj
                                    .write_to_file(&file_path)
                                    .whatever_context("could not save DICOM object to file")?;
                                info!("Stored {}", file_path.display());
                                let file_path_str = file_path.display().to_string();

                                // Emit the OnFileStored event
                                on_file_stored(
                                    serde_json::json!({
                                        "sop_class-uid": sop_class_uid.clone(),
                                        "sop_instance_uid": sop_instance_uid.clone(),
                                        "transfer_syntax_uid": transfer_syntax_uid.clone(),
                                        "study_instance_uid": study_instance_uid.clone(),
                                        "series_instance_uid": series_instance_uid.clone(),
                                        "series_number": series_data.series_number,
                                        "instance": {
                                            "sop_instance_uid": sop_instance_uid.clone(),
                                            "instance_number": instance_data.instance_number,
                                            "file_path": file_path_str.clone(),
                                            "rows": instance_data.rows,
                                            "columns": instance_data.columns,
                                            "bits_allocated": instance_data.bits_allocated,
                                            "bits_stored": instance_data.bits_stored,
                                            "high_bit": instance_data.high_bit,
                                            "pixel_representation": instance_data.pixel_representation,
                                            "photometric_interpretation": instance_data.photometric_interpretation,
                                            "planar_configuration": instance_data.planar_configuration,
                                            "pixel_aspect_ratio": instance_data.pixel_aspect_ratio,
                                            "pixel_spacing": instance_data.pixel_spacing,
                                            "lossy_image_compression": instance_data.lossy_image_compression,
                                        }
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

                                    let series = study.series.iter_mut().find(|s| s.series_instance_uid == series_instance_uid);
                                    if let Some(series) = series {
                                        if !series.instances.iter().any(|i| i.sop_instance_uid == sop_instance_uid) {
                                            series.instances.push(Instance {
                                                sop_instance_uid: sop_instance_uid.clone(),
                                                instance_number: instance_data.instance_number,
                                                file_path: file_path_str.clone(),
                                                rows: instance_data.rows,
                                                columns: instance_data.columns,
                                                bits_allocated: instance_data.bits_allocated,
                                                bits_stored: instance_data.bits_stored,
                                                high_bit: instance_data.high_bit,
                                                pixel_representation: instance_data.pixel_representation,
                                                photometric_interpretation: instance_data.photometric_interpretation,
                                                planar_configuration: instance_data.planar_configuration,
                                                pixel_aspect_ratio: instance_data.pixel_aspect_ratio,
                                                pixel_spacing: instance_data.pixel_spacing,
                                                lossy_image_compression: instance_data.lossy_image_compression
                                            });
                                        }
                                    } else {
                                        study.series.push(Series {
                                            series_instance_uid: series_instance_uid.clone(),
                                            series_number: series_data.series_number,
                                            body_part_examined: series_data.body_part_examined,
                                            protocol_name: series_data.protocol_name,
                                            contrast_bolus_agent: series_data.contrast_bolus_agent,
                                            instances: vec![Instance {
                                                sop_instance_uid: sop_instance_uid.clone(),
                                                instance_number: instance_data.instance_number,
                                                file_path: file_path_str.clone(),
                                                rows: instance_data.rows,
                                                columns: instance_data.columns,
                                                bits_allocated: instance_data.bits_allocated,
                                                bits_stored: instance_data.bits_stored,
                                                high_bit: instance_data.high_bit,
                                                pixel_representation: instance_data.pixel_representation,
                                                photometric_interpretation: instance_data.photometric_interpretation,
                                                planar_configuration: instance_data.planar_configuration,
                                                pixel_aspect_ratio: instance_data.pixel_aspect_ratio,
                                                pixel_spacing: instance_data.pixel_spacing,
                                                lossy_image_compression: instance_data.lossy_image_compression
                                            }],
                                        });
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
            Err(err @ dicom_ul::association::server::Error::Receive { .. }) => {
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

fn extract_additional_metadata(obj: &InMemDicomObject<StandardDataDictionary>) -> (ClinicalData, SeriesData, InstanceData) {
    let clinical_data = ClinicalData {
        patient_name: get_str_tag(obj, tags::PATIENT_NAME),
        patient_id: get_str_tag(obj, tags::PATIENT_ID),
        patient_birth_date: get_str_tag(obj, tags::PATIENT_BIRTH_DATE),
        patient_sex: get_str_tag(obj, tags::PATIENT_SEX),
    };

    let instance_data = InstanceData {
        instance_number: get_int_tag(obj, tags::INSTANCE_NUMBER),
        instance_sop_class_uid: get_str_tag(obj, tags::SOP_CLASS_UID),
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

    let series_data = SeriesData {
        series_number: get_int_tag(obj, tags::SERIES_NUMBER),
        modality: get_str_tag(obj, tags::MODALITY),
        body_part_examined: get_str_tag(obj, tags::BODY_PART_EXAMINED),
        protocol_name: get_str_tag(obj, tags::PROTOCOL_NAME),
        contrast_bolus_agent: get_str_tag(obj, tags::CONTRAST_BOLUS_AGENT),
    };

    (clinical_data, series_data, instance_data)
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