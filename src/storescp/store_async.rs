use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dicom_dictionary_std::tags;
use dicom_encoding::transfer_syntax::TransferSyntaxIndex;
use dicom_object::{FileMetaTableBuilder, InMemDicomObject};
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

use crate::storescp::{create_cecho_response, create_cstore_response, transfer::ABSTRACT_SYNTAXES, StoreScp, ScpEventDetails, StudyHierarchyData, SeriesHierarchyData, InstanceHierarchyData};
use crate::utils::{build_s3_bucket, s3_put_object};
use crate::tag_extractor::{CustomTag, GroupingStrategy, ExtractionResult, ScopedDicomData, StudyLevelData};
use crate::dicom_tags::{parse_tag, get_tag_scope, TagScope};

// New hierarchy for OnStudyCompleted event
#[derive(Clone, Debug, Serialize)]
struct StudyHierarchy {
    #[serde(rename = "studyInstanceUid")]
    study_instance_uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<ExtractionResult>,
    series: Vec<SeriesHierarchy>,
}

#[derive(Clone, Debug, Serialize)]
struct SeriesHierarchy {
    #[serde(rename = "seriesInstanceUid")]
    series_instance_uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<ExtractionResult>,
    instances: Vec<InstanceHierarchy>,
}

#[derive(Clone, Debug, Serialize)]
struct InstanceHierarchy {
    #[serde(rename = "sopInstanceUid")]
    sop_instance_uid: String,
    #[serde(rename = "sopClassUid")]
    sop_class_uid: String,
    #[serde(rename = "transferSyntaxUid")]
    transfer_syntax_uid: String,
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<ExtractionResult>,
}

lazy_static::lazy_static! {
    static ref STUDY_STORE: Mutex<HashMap<String, StudyHierarchy>> = Mutex::new(HashMap::new());
}

/// Extract tags from InMemDicomObject with grouping strategy
fn extract_tags_from_inmem(
    obj: &InMemDicomObject<dicom_dictionary_std::StandardDataDictionary>,
    tag_names: &[String],
    custom_tags: &[CustomTag],
    strategy: GroupingStrategy,
) -> ExtractionResult {
    match strategy {
        GroupingStrategy::ByScope => ExtractionResult::Scoped(extract_by_scope_inmem(obj, tag_names, custom_tags)),
        GroupingStrategy::Flat => ExtractionResult::Flat(extract_flat_inmem(obj, tag_names, custom_tags)),
        GroupingStrategy::StudyLevel => ExtractionResult::StudyLevel(extract_study_level_inmem(obj, tag_names, custom_tags)),
        GroupingStrategy::Custom => ExtractionResult::Scoped(extract_by_scope_inmem(obj, tag_names, custom_tags)),
    }
}

fn extract_by_scope_inmem(
    obj: &InMemDicomObject<dicom_dictionary_std::StandardDataDictionary>,
    tag_names: &[String],
    custom_tags: &[CustomTag],
) -> ScopedDicomData {
    let mut result = ScopedDicomData::default();
    let mut patient = HashMap::new();
    let mut study = HashMap::new();
    let mut series = HashMap::new();
    let mut instance = HashMap::new();
    let mut equipment = HashMap::new();
    let mut custom = HashMap::new();
    
    for tag_name in tag_names {
        if let Ok(tag) = parse_tag(tag_name) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    let scope = get_tag_scope(tag);
                    let value = value_str.to_string();
                    
                    match scope {
                        TagScope::Patient => { patient.insert(tag_name.clone(), value); },
                        TagScope::Study => { study.insert(tag_name.clone(), value); },
                        TagScope::Series => { series.insert(tag_name.clone(), value); },
                        TagScope::Instance => { instance.insert(tag_name.clone(), value); },
                        TagScope::Equipment => { equipment.insert(tag_name.clone(), value); },
                    }
                }
            }
        }
    }
    
    for custom_tag in custom_tags {
        if let Ok(tag) = parse_tag(&custom_tag.tag) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    custom.insert(custom_tag.name.clone(), value_str.to_string());
                }
            }
        }
    }
    
    if !patient.is_empty() { result.patient = Some(patient); }
    if !study.is_empty() { result.study = Some(study); }
    if !series.is_empty() { result.series = Some(series); }
    if !instance.is_empty() { result.instance = Some(instance); }
    if !equipment.is_empty() { result.equipment = Some(equipment); }
    if !custom.is_empty() { result.custom = Some(custom); }
    
    result
}

fn extract_flat_inmem(
    obj: &InMemDicomObject<dicom_dictionary_std::StandardDataDictionary>,
    tag_names: &[String],
    custom_tags: &[CustomTag],
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    for tag_name in tag_names {
        if let Ok(tag) = parse_tag(tag_name) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    result.insert(tag_name.clone(), value_str.to_string());
                }
            }
        }
    }
    
    for custom_tag in custom_tags {
        if let Ok(tag) = parse_tag(&custom_tag.tag) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    result.insert(custom_tag.name.clone(), value_str.to_string());
                }
            }
        }
    }
    
    result
}

fn extract_study_level_inmem(
    obj: &InMemDicomObject<dicom_dictionary_std::StandardDataDictionary>,
    tag_names: &[String],
    custom_tags: &[CustomTag],
) -> StudyLevelData {
    let mut result = StudyLevelData::default();
    let mut study_level = HashMap::new();
    let mut instance_level = HashMap::new();
    let mut custom = HashMap::new();
    
    for tag_name in tag_names {
        if let Ok(tag) = parse_tag(tag_name) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    let scope = get_tag_scope(tag);
                    let value = value_str.to_string();
                    
                    match scope {
                        TagScope::Patient | TagScope::Study => {
                            study_level.insert(tag_name.clone(), value);
                        },
                        TagScope::Series | TagScope::Instance | TagScope::Equipment => {
                            instance_level.insert(tag_name.clone(), value);
                        },
                    }
                }
            }
        }
    }
    
    for custom_tag in custom_tags {
        if let Ok(tag) = parse_tag(&custom_tag.tag) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    custom.insert(custom_tag.name.clone(), value_str.to_string());
                }
            }
        }
    }
    
    if !study_level.is_empty() { result.study_level = Some(study_level); }
    if !instance_level.is_empty() { result.instance_level = Some(instance_level); }
    if !custom.is_empty() { result.custom = Some(custom); }
    
    result
}

/// Extract tags at specific hierarchy level (study/series/instance) based on scope
fn extract_at_hierarchy_level(
    obj: &InMemDicomObject<dicom_dictionary_std::StandardDataDictionary>,
    tag_names: &[String],
    custom_tags: &[CustomTag],
    level: HierarchyLevel,
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    for tag_name in tag_names {
        if let Ok(tag) = parse_tag(tag_name) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    let scope = get_tag_scope(tag);
                    let include = match level {
                        HierarchyLevel::Study => matches!(scope, TagScope::Patient | TagScope::Study),
                        HierarchyLevel::Series => matches!(scope, TagScope::Series),
                        HierarchyLevel::Instance => matches!(scope, TagScope::Instance | TagScope::Equipment),
                    };
                    
                    if include {
                        result.insert(tag_name.clone(), value_str.to_string());
                    }
                }
            }
        }
    }
    
    // Custom tags always go to instance level
    if matches!(level, HierarchyLevel::Instance) {
        for custom_tag in custom_tags {
            if let Ok(tag) = parse_tag(&custom_tag.tag) {
                if let Ok(elem) = obj.element(tag) {
                    if let Ok(value_str) = elem.to_str() {
                        result.insert(custom_tag.name.clone(), value_str.to_string());
                    }
                }
            }
        }
    }
    
    result
}

#[derive(Debug, Clone, Copy)]
enum HierarchyLevel {
    Study,
    Series,
    Instance,
}

pub async fn run_store_async(
    scu_stream: tokio::net::TcpStream,
    args: &StoreScp,
    on_file_stored: impl Fn(ScpEventDetails) + Send + 'static,
    on_study_completed: Arc<Mutex<dyn Fn(StudyHierarchyData) + Send + 'static>>
) -> Result<(), Whatever> {
    let StoreScp {
        verbose,
        calling_ae_title,
        strict,
        max_pdu_length,
        out_dir,
        port: _,
        study_timeout,
        storage_backend,
        s3_config,
        store_with_file_meta,
        extract_tags,
        extract_custom_tags,
        grouping_strategy: _,
        abstract_syntax_mode,
        abstract_syntaxes,
        transfer_syntax_mode,
        transfer_syntaxes,
    } = args;

    let mut options = dicom_ul::association::ServerAssociationOptions::new()
        .accept_any()
        .ae_title(calling_ae_title)
        .strict(*strict)
        .max_pdu_length(*max_pdu_length);

    // Configure abstract syntaxes based on mode
    use crate::storescp::{AbstractSyntaxMode, TransferSyntaxMode};
    match abstract_syntax_mode {
        AbstractSyntaxMode::All => {
            options = options.promiscuous(true);
        },
        AbstractSyntaxMode::AllStorage => {
            // Use the default list of storage SOP classes
            for uid in ABSTRACT_SYNTAXES {
                options = options.with_abstract_syntax(*uid);
            }
        },
        AbstractSyntaxMode::Custom => {
            // Use user-provided list
            use crate::storescp::sop_classes::map_sop_class_name;
            for name_or_uid in abstract_syntaxes {
                let uid = map_sop_class_name(name_or_uid).unwrap_or(name_or_uid.as_str());
                options = options.with_abstract_syntax(uid);
            }
        }
    }

    // Configure transfer syntaxes based on mode
    match transfer_syntax_mode {
        TransferSyntaxMode::All => {
            for ts in TransferSyntaxRegistry.iter() {
                if !ts.is_unsupported() {
                    options = options.with_transfer_syntax(ts.uid());
                }
            }
        },
        TransferSyntaxMode::UncompressedOnly => {
            options = options
                .with_transfer_syntax("1.2.840.10008.1.2")      // Implicit VR Little Endian
                .with_transfer_syntax("1.2.840.10008.1.2.1");   // Explicit VR Little Endian
        },
        TransferSyntaxMode::Custom => {
            // Use user-provided list
            use crate::storescp::sop_classes::map_transfer_syntax_name;
            for name_or_uid in transfer_syntaxes {
                let uid = map_transfer_syntax_name(name_or_uid).unwrap_or(name_or_uid.as_str());
                options = options.with_transfer_syntax(uid);
            }
        }
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
        *store_with_file_meta,
        args,
        extract_tags,
        extract_custom_tags,
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
    store_with_file_meta: bool,
    args: &StoreScp,
    extract_tags: &[String],
    extract_custom_tags: &[CustomTag],
    on_file_stored: impl Fn(ScpEventDetails) + Send + 'static,
    on_study_completed: Arc<Mutex<dyn Fn(StudyHierarchyData) + Send + 'static>>
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
                                if store_with_file_meta {
                                    // Write complete DICOM file with file meta header
                                    file_obj.write_all(&mut dicom_bytes).whatever_context("could not serialize DICOM object")?;
                                } else {
                                    // Write dataset-only (more efficient, standard for PACS)
                                    file_obj.write_dataset_with_ts(&mut dicom_bytes, TransferSyntaxRegistry.get(ts).unwrap()).whatever_context("could not serialize DICOM object")?;
                                }
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

                                // Extract metadata using the tag extractor with configured strategy
                                let grouping = match &args.grouping_strategy {
                                    crate::storescp::GroupingStrategy::ByScope => GroupingStrategy::ByScope,
                                    crate::storescp::GroupingStrategy::Flat => GroupingStrategy::Flat,
                                    crate::storescp::GroupingStrategy::StudyLevel => GroupingStrategy::StudyLevel,
                                    crate::storescp::GroupingStrategy::Custom => GroupingStrategy::Custom,
                                };
                                
                                let extraction_result = extract_tags_from_inmem(
                                    &obj,
                                    extract_tags,
                                    extract_custom_tags,
                                    grouping,
                                );
                                
                                // Create event details with proper types
                                let (tags_scoped, tags_flat, tags_study_level) = match extraction_result {
                                    ExtractionResult::Scoped(scoped) => (Some(scoped), None, None),
                                    ExtractionResult::Flat(flat) => (None, Some(flat), None),
                                    ExtractionResult::StudyLevel(study_level) => (None, None, Some(study_level)),
                                };

                                // Emit the OnFileStored event with proper typed structs
                                on_file_stored(ScpEventDetails {
                                    file: Some(file_path_str.clone()),
                                    sop_instance_uid: Some(sop_instance_uid.clone()),
                                    sop_class_uid: Some(sop_class_uid.clone()),
                                    transfer_syntax_uid: Some(transfer_syntax_uid.clone()),
                                    study_instance_uid: Some(study_instance_uid.clone()),
                                    series_instance_uid: Some(series_instance_uid.clone()),
                                    tags_scoped,
                                    tags_flat,
                                    tags_study_level,
                                    error: None,
                                    study: None,
                                });

                                // Update global study store with hierarchy
                                // Extract data based on grouping strategy to avoid duplication
                                {
                                    let mut store = STUDY_STORE.lock().await;
                                    
                                    // Extract tags based on grouping strategy
                                    let (study_tags, series_tags, instance_tags) = match grouping {
                                        // ByScope: Return ScopedDicomData with proper hierarchy
                                        GroupingStrategy::ByScope | GroupingStrategy::Custom => {
                                            let study_result = ExtractionResult::Scoped(ScopedDicomData {
                                                patient: {
                                                    let data = extract_at_hierarchy_level(
                                                        &obj, extract_tags, extract_custom_tags, HierarchyLevel::Study
                                                    );
                                                    if data.is_empty() { None } else { Some(data) }
                                                },
                                                study: None,
                                                series: None,
                                                instance: None,
                                                equipment: None,
                                                custom: None,
                                            });
                                            let series_result = ExtractionResult::Scoped(ScopedDicomData {
                                                patient: None,
                                                study: None,
                                                series: {
                                                    let data = extract_at_hierarchy_level(
                                                        &obj, extract_tags, extract_custom_tags, HierarchyLevel::Series
                                                    );
                                                    if data.is_empty() { None } else { Some(data) }
                                                },
                                                instance: None,
                                                equipment: None,
                                                custom: None,
                                            });
                                            let instance_result = ExtractionResult::Scoped(ScopedDicomData {
                                                patient: None,
                                                study: None,
                                                series: None,
                                                instance: {
                                                    let data = extract_at_hierarchy_level(
                                                        &obj, extract_tags, extract_custom_tags, HierarchyLevel::Instance
                                                    );
                                                    if data.is_empty() { None } else { Some(data) }
                                                },
                                                equipment: None,
                                                custom: None,
                                            });
                                            (Some(study_result), Some(series_result), Some(instance_result))
                                        },
                                        // Flat: All data as flat HashMap
                                        GroupingStrategy::Flat => {
                                            let all_data = extract_flat_inmem(&obj, extract_tags, extract_custom_tags);
                                            let result = ExtractionResult::Flat(all_data);
                                            (None, None, Some(result))
                                        },
                                        // StudyLevel: Study+Patient at study level, rest at instance
                                        GroupingStrategy::StudyLevel => {
                                            let study_level_data = extract_at_hierarchy_level(
                                                &obj, extract_tags, extract_custom_tags, HierarchyLevel::Study
                                            );
                                            let mut instance_level_data = extract_at_hierarchy_level(
                                                &obj, extract_tags, extract_custom_tags, HierarchyLevel::Series
                                            );
                                            instance_level_data.extend(extract_at_hierarchy_level(
                                                &obj, extract_tags, extract_custom_tags, HierarchyLevel::Instance
                                            ));
                                            
                                            let study_result = ExtractionResult::StudyLevel(StudyLevelData {
                                                study_level: if study_level_data.is_empty() { None } else { Some(study_level_data) },
                                                instance_level: None,
                                                custom: None,
                                            });
                                            let instance_result = ExtractionResult::StudyLevel(StudyLevelData {
                                                study_level: None,
                                                instance_level: if instance_level_data.is_empty() { None } else { Some(instance_level_data) },
                                                custom: None,
                                            });
                                            (Some(study_result), None, Some(instance_result))
                                        },
                                    };
                                    
                                    let study = store.entry(study_instance_uid.clone()).or_insert_with(|| StudyHierarchy {
                                        study_instance_uid: study_instance_uid.clone(),
                                        tags: study_tags.clone(),
                                        series: Vec::new(),
                                    });

                                    let series_entry = study.series.iter_mut().find(|s| s.series_instance_uid == series_instance_uid);
                                    let instance_hierarchy = InstanceHierarchy {
                                        sop_instance_uid: sop_instance_uid.clone(),
                                        sop_class_uid: sop_class_uid.clone(),
                                        transfer_syntax_uid: transfer_syntax_uid.clone(),
                                        file: file_path_str.clone(),
                                        tags: instance_tags.clone(),
                                    };
                                    
                                    if let Some(series_entry) = series_entry {
                                        if !series_entry.instances.iter().any(|i| i.sop_instance_uid == sop_instance_uid) {
                                            series_entry.instances.push(instance_hierarchy);
                                        }
                                    } else {
                                        let mut new_series = SeriesHierarchy {
                                            series_instance_uid: series_instance_uid.clone(),
                                            tags: series_tags.clone(),
                                            instances: Vec::new(),
                                        };
                                        new_series.instances.push(instance_hierarchy);
                                        study.series.push(new_series);
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
                                                
                                                // Convert StudyHierarchy to StudyHierarchyData
                                                let (study_tags_scoped, study_tags_flat, study_tags_study_level) = if let Some(tags) = study.tags {
                                                    match tags {
                                                        ExtractionResult::Scoped(scoped) => (Some(scoped), None, None),
                                                        ExtractionResult::Flat(flat) => (None, Some(flat), None),
                                                        ExtractionResult::StudyLevel(study_level) => (None, None, Some(study_level)),
                                                    }
                                                } else {
                                                    (None, None, None)
                                                };
                                                
                                                let series_data: Vec<SeriesHierarchyData> = study.series.into_iter().map(|s| {
                                                    let (series_tags_scoped, series_tags_flat, series_tags_study_level) = if let Some(tags) = s.tags {
                                                        match tags {
                                                            ExtractionResult::Scoped(scoped) => (Some(scoped), None, None),
                                                            ExtractionResult::Flat(flat) => (None, Some(flat), None),
                                                            ExtractionResult::StudyLevel(study_level) => (None, None, Some(study_level)),
                                                        }
                                                    } else {
                                                        (None, None, None)
                                                    };
                                                    
                                                    let instances_data: Vec<InstanceHierarchyData> = s.instances.into_iter().map(|i| {
                                                        let (inst_tags_scoped, inst_tags_flat, inst_tags_study_level) = if let Some(tags) = i.tags {
                                                            match tags {
                                                                ExtractionResult::Scoped(scoped) => (Some(scoped), None, None),
                                                                ExtractionResult::Flat(flat) => (None, Some(flat), None),
                                                                ExtractionResult::StudyLevel(study_level) => (None, None, Some(study_level)),
                                                            }
                                                        } else {
                                                            (None, None, None)
                                                        };
                                                        
                                                        InstanceHierarchyData {
                                                            sop_instance_uid: i.sop_instance_uid,
                                                            sop_class_uid: i.sop_class_uid,
                                                            transfer_syntax_uid: i.transfer_syntax_uid,
                                                            file: i.file,
                                                            tags_scoped: inst_tags_scoped,
                                                            tags_flat: inst_tags_flat,
                                                            tags_study_level: inst_tags_study_level,
                                                        }
                                                    }).collect();
                                                    
                                                    SeriesHierarchyData {
                                                        series_instance_uid: s.series_instance_uid,
                                                        tags_scoped: series_tags_scoped,
                                                        tags_flat: series_tags_flat,
                                                        tags_study_level: series_tags_study_level,
                                                        instances: instances_data,
                                                    }
                                                }).collect();
                                                
                                                let study_hierarchy_data = StudyHierarchyData {
                                                    study_instance_uid: study.study_instance_uid,
                                                    tags_scoped: study_tags_scoped,
                                                    tags_flat: study_tags_flat,
                                                    tags_study_level: study_tags_study_level,
                                                    series: series_data,
                                                };
                                                
                                                on_study_completed(study_hierarchy_data);
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
        let s3_result = s3_put_object(&bucket, &key, &data).await;
        match s3_result {
            Ok(()) => Ok(()),
            Err(_e) => {
                error!("Failed to upload file to S3: {}", key);
                Err(format!("S3 upload failed: {}", key).into())
            }
        }
    }
}