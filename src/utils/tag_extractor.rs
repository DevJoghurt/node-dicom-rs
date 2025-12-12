use dicom_object::DefaultDicomObject;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use napi_derive::napi;

use crate::dicom_tags::{parse_tag, get_tag_scope, TagScope};

/**
 * Grouping strategy for organizing extracted DICOM tag data.
 * 
 * Determines how extracted tags are structured in the output JSON.
 * Different strategies are useful for different workflows and data processing needs.
 * 
 * ## Strategy Types
 * 
 * ### ByScope
 * Groups tags by DICOM hierarchy level:
 * - `patient`: Patient demographics (PatientName, PatientID, etc.)
 * - `study`: Study-level metadata (StudyInstanceUID, StudyDate, etc.)
 * - `series`: Series-level metadata (SeriesInstanceUID, Modality, etc.)
 * - `instance`: Instance-level data (SOPInstanceUID, InstanceNumber, etc.)
 * - `equipment`: Device information (Manufacturer, StationName, etc.)
 * - `custom`: User-defined custom tags
 * 
 * Best for: DICOM-aware applications, PACS systems, hierarchical data processing
 * 
 * ### Flat
 * All tags at the root level without grouping.
 * 
 * Best for: Simple key-value access, database insertion, quick lookups
 * 
 * ### StudyLevel
 * Groups tags into two levels:
 * - `studyLevel`: Patient + Study tags (persists across all instances)
 * - `instanceLevel`: Series + Instance + Equipment tags (varies per file)
 * 
 * Best for: Study-based processing, deduplication, study aggregation
 * 
 * ### Custom
 * User-defined grouping rules (currently defaults to ByScope).
 * 
 * Best for: Future extensibility with custom grouping logic
 * 
 * @example
 * ```typescript
 * import { DicomFile } from '@nuxthealth/node-dicom';
 * 
 * const dicom = new DicomFile();
 * dicom.open('scan.dcm');
 * 
 * // ByScope strategy (hierarchical)
 * const scoped = dicom.extract(
 *   ['PatientName', 'StudyDate', 'Modality', 'SOPInstanceUID'],
 *   undefined,
 *   'ByScope'
 * );
 * console.log(JSON.parse(scoped));
 * // {
 * //   patient: { PatientName: 'DOE^JOHN' },
 * //   study: { StudyDate: '20240101' },
 * //   series: { Modality: 'CT' },
 * //   instance: { SOPInstanceUID: '1.2.3.4...' }
 * // }
 * 
 * // Flat strategy (simple)
 * const flat = dicom.extract(
 *   ['PatientName', 'StudyDate', 'Modality'],
 *   undefined,
 *   'Flat'
 * );
 * console.log(JSON.parse(flat));
 * // {
 * //   PatientName: 'DOE^JOHN',
 * //   StudyDate: '20240101',
 * //   Modality: 'CT'
 * // }
 * 
 * // StudyLevel strategy (two-tier)
 * const studyLevel = dicom.extract(
 *   ['PatientName', 'StudyDate', 'SeriesNumber', 'InstanceNumber'],
 *   undefined,
 *   'StudyLevel'
 * );
 * console.log(JSON.parse(studyLevel));
 * // {
 * //   studyLevel: { PatientName: 'DOE^JOHN', StudyDate: '20240101' },
 * //   instanceLevel: { SeriesNumber: '1', InstanceNumber: '1' }
 * // }
 * ```
 * 
 * @example
 * ```typescript
 * // Practical use cases
 * 
 * // PACS integration (ByScope)
 * const pacsData = dicom.extract(
 *   ['PatientID', 'StudyInstanceUID', 'SeriesInstanceUID', 'SOPInstanceUID'],
 *   undefined,
 *   'ByScope'
 * );
 * 
 * // Database insertion (Flat)
 * const dbData = dicom.extract(
 *   ['PatientID', 'StudyDate', 'Modality', 'FilePath'],
 *   undefined,
 *   'Flat'
 * );
 * // Can directly map to database columns
 * 
 * // Study deduplication (StudyLevel)
 * const dedupeData = dicom.extract(
 *   ['PatientID', 'StudyInstanceUID', 'SeriesInstanceUID', 'SOPInstanceUID'],
 *   undefined,
 *   'StudyLevel'
 * );
 * // studyLevel contains study key, instanceLevel contains file-specific data
 * ```
 */
#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupingStrategy {
    /// Group by DICOM hierarchy scope (Patient, Study, Series, Instance, Equipment)
    ByScope,
    /// Flat structure with all tags at root level
    Flat,
    /// Group into studyLevel (Patient+Study) and instanceLevel (Series+Instance+Equipment)
    StudyLevel,
    /// Custom grouping rules (user provides mapping)
    Custom,
}

/**
 * Custom tag specification for extracting non-standard or private DICOM tags.
 * 
 * Allows extraction of tags that aren't in the standard dictionary by specifying
 * the tag number directly. The extracted value will appear in the output using
 * your custom name instead of the raw tag number.
 * 
 * ## Tag Format
 * Tags can be specified in multiple formats:
 * - Hex with parentheses: `(0010,0010)`
 * - Hex without parentheses: `00100010`
 * - Standard tag name: `PatientName` (if in dictionary)
 * 
 * ## Output Location
 * Custom tags are grouped in a separate `custom` section in the output,
 * regardless of the grouping strategy used.
 * 
 * @example
 * ```typescript
 * import { DicomFile, CustomTag } from '@nuxthealth/node-dicom';
 * 
 * const dicom = new DicomFile();
 * dicom.open('scan-with-private-tags.dcm');
 * 
 * // Define custom tags
 * const customTags: CustomTag[] = [
 *   { tag: '(0009,1001)', name: 'VendorID' },
 *   { tag: '(0019,100A)', name: 'ScannerMode' },
 *   { tag: '00091010', name: 'PrivateField' } // Without parentheses
 * ];
 * 
 * // Extract with custom tags
 * const data = dicom.extract(
 *   ['PatientName', 'StudyDate'],
 *   customTags,
 *   'ByScope'
 * );
 * 
 * console.log(JSON.parse(data));
 * // {
 * //   patient: { PatientName: 'DOE^JOHN' },
 * //   study: { StudyDate: '20240101' },
 * //   custom: {
 * //     VendorID: 'GE_MED',
 * //     ScannerMode: 'HELICAL',
 * //     PrivateField: 'value'
 * //   }
 * // }
 * ```
 * 
 * @example
 * ```typescript
 * // Vendor-specific private tags
 * 
 * // GE Medical Systems
 * const geTags: CustomTag[] = [
 *   { tag: '(0009,1001)', name: 'GE_PrivateCreator' },
 *   { tag: '(0043,1010)', name: 'GE_ImageType' }
 * ];
 * 
 * // Siemens
 * const siemensTags: CustomTag[] = [
 *   { tag: '(0019,1008)', name: 'Siemens_CSAImageHeader' },
 *   { tag: '(0029,1010)', name: 'Siemens_CSASeriesHeader' }
 * ];
 * 
 * // Philips
 * const philipsTags: CustomTag[] = [
 *   { tag: '(2001,1001)', name: 'Philips_ImageType' },
 *   { tag: '(2005,1080)', name: 'Philips_ReconstructionNumber' }
 * ];
 * ```
 * 
 * @example
 * ```typescript
 * // Extract both standard and custom tags
 * import { getCommonTagSets } from '@nuxthealth/node-dicom';
 * 
 * const tagSets = getCommonTagSets();
 * const dicom = new DicomFile();
 * dicom.open('scan.dcm');
 * 
 * const customTags: CustomTag[] = [
 *   { tag: '(0009,1001)', name: 'PrivateTag1' },
 *   { tag: '(0019,100A)', name: 'PrivateTag2' }
 * ];
 * 
 * // Combine standard and custom extraction
 * const data = dicom.extract(
 *   tagSets.default, // All standard tags
 *   customTags,      // Plus custom private tags
 *   'ByScope'
 * );
 * ```
 */
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTag {
    /// The tag in hex format (e.g., "(0010,0010)" or "00100010") or tag name
    pub tag: String,
    /// User-defined name for this tag in the output (appears in 'custom' section)
    pub name: String,
}

/**
 * DICOM data extracted and grouped by hierarchy scope.
 * 
 * Result structure when using `GroupingStrategy.ByScope`.
 * Tags are organized into sections based on their DICOM hierarchy level.
 * Empty sections are omitted from the output.
 * 
 * ## Hierarchy Levels
 * - **patient**: Patient demographics and identification
 * - **study**: Study-level metadata and workflow info
 * - **series**: Series-level imaging parameters
 * - **instance**: Instance-specific data (per image/file)
 * - **equipment**: Device and acquisition equipment info
 * - **custom**: User-defined custom/private tags
 * 
 * @example
 * ```typescript
 * import { DicomFile, ScopedDicomData } from '@nuxthealth/node-dicom';
 * 
 * const dicom = new DicomFile();
 * dicom.open('ct-scan.dcm');
 * 
 * const json = dicom.extract(
 *   ['PatientName', 'PatientID', 'StudyDate', 'Modality', 'SOPInstanceUID'],
 *   undefined,
 *   'ByScope'
 * );
 * 
 * const data: ScopedDicomData = JSON.parse(json);
 * 
 * // Access by hierarchy level
 * if (data.patient) {
 *   console.log('Patient:', data.patient.PatientName);
 *   console.log('ID:', data.patient.PatientID);
 * }
 * 
 * if (data.study) {
 *   console.log('Study Date:', data.study.StudyDate);
 * }
 * 
 * if (data.series) {
 *   console.log('Modality:', data.series.Modality);
 * }
 * 
 * if (data.instance) {
 *   console.log('SOP UID:', data.instance.SOPInstanceUID);
 * }
 * ```
 * 
 * @example
 * ```typescript
 * // Type-safe processing
 * interface PatientInfo {
 *   PatientName?: string;
 *   PatientID?: string;
 *   PatientBirthDate?: string;
 * }
 * 
 * interface StudyInfo {
 *   StudyInstanceUID?: string;
 *   StudyDate?: string;
 *   StudyDescription?: string;
 * }
 * 
 * const data: ScopedDicomData = JSON.parse(json);
 * 
 * const patient = data.patient as PatientInfo | undefined;
 * const study = data.study as StudyInfo | undefined;
 * 
 * if (patient?.PatientID && study?.StudyInstanceUID) {
 *   // Process study for this patient
 *   console.log(`Patient ${patient.PatientID}: Study ${study.StudyInstanceUID}`);
 * }
 * ```
 */
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScopedDicomData {
    /// Patient-level tags (demographics, identification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient: Option<HashMap<String, String>>,
    
    /// Study-level tags (study metadata, workflow)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study: Option<HashMap<String, String>>,
    
    /// Series-level tags (series metadata, imaging parameters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<HashMap<String, String>>,
    
    /// Instance-level tags (per-image/file data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<HashMap<String, String>>,
    
    /// Equipment-level tags (device, manufacturer info)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<HashMap<String, String>>,
    
    /// Custom/private tags with user-defined names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

/**
 * DICOM data extracted and grouped by study level.
 * 
 * Result structure when using `GroupingStrategy.StudyLevel`.
 * Separates data that persists across all instances in a study (study-level)
 * from data that varies per file (instance-level).
 * 
 * ## Two-Tier Structure
 * - **studyLevel**: Patient + Study tags (same for all instances in a study)
 * - **instanceLevel**: Series + Instance + Equipment tags (varies per file)
 * - **custom**: User-defined custom/private tags
 * 
 * This grouping is particularly useful for:
 * - Study deduplication (use studyLevel as the key)
 * - Batch processing (group files by studyLevel)
 * - Database optimization (separate study and instance tables)
 * - Study aggregation (combine multiple instances using studyLevel)
 * 
 * @example
 * ```typescript
 * import { DicomFile, StudyLevelData } from '@nuxthealth/node-dicom';
 * 
 * const dicom = new DicomFile();
 * dicom.open('series1-image1.dcm');
 * 
 * const json = dicom.extract(
 *   [
 *     'PatientID', 'PatientName',           // Study-level
 *     'StudyInstanceUID', 'StudyDate',      // Study-level
 *     'SeriesNumber', 'InstanceNumber',     // Instance-level
 *     'SOPInstanceUID', 'Modality'          // Instance-level
 *   ],
 *   undefined,
 *   'StudyLevel'
 * );
 * 
 * const data: StudyLevelData = JSON.parse(json);
 * 
 * // Study-level data (same for all instances in this study)
 * console.log('Study Key:');
 * console.log(data.studyLevel);
 * // {
 * //   PatientID: '12345',
 * //   PatientName: 'DOE^JOHN',
 * //   StudyInstanceUID: '1.2.3.4.5...',
 * //   StudyDate: '20240101'
 * // }
 * 
 * // Instance-level data (unique per file)
 * console.log('Instance Data:');
 * console.log(data.instanceLevel);
 * // {
 * //   SeriesNumber: '1',
 * //   InstanceNumber: '1',
 * //   SOPInstanceUID: '1.2.3.4.5.6...',
 * //   Modality: 'CT'
 * // }
 * ```
 * 
 * @example
 * ```typescript
 * // Study deduplication workflow
 * interface StudyKey {
 *   PatientID: string;
 *   StudyInstanceUID: string;
 * }
 * 
 * const studyMap = new Map<string, StudyKey>();
 * const files = ['img1.dcm', 'img2.dcm', 'img3.dcm'];
 * 
 * for (const file of files) {
 *   const dicom = new DicomFile();
 *   dicom.open(file);
 *   
 *   const json = dicom.extract(
 *     ['PatientID', 'StudyInstanceUID', 'SOPInstanceUID'],
 *     undefined,
 *     'StudyLevel'
 *   );
 *   
 *   const data: StudyLevelData = JSON.parse(json);
 *   
 *   // Use studyLevel as deduplication key
 *   const studyKey = JSON.stringify(data.studyLevel);
 *   if (!studyMap.has(studyKey)) {
 *     studyMap.set(studyKey, data.studyLevel as any);
 *     console.log('New study:', studyKey);
 *   }
 *   
 *   // Process instance-specific data
 *   console.log('Instance:', data.instanceLevel?.SOPInstanceUID);
 * }
 * 
 * console.log(`Total unique studies: ${studyMap.size}`);
 * ```
 * 
 * @example
 * ```typescript
 * // Database insertion with normalized schema
 * import { DicomFile, StudyLevelData } from '@nuxthealth/node-dicom';
 * 
 * async function importDicomToDb(filePath: string, db: Database) {
 *   const dicom = new DicomFile();
 *   dicom.open(filePath);
 *   
 *   const json = dicom.extract(
 *     ['PatientID', 'StudyInstanceUID', 'SeriesInstanceUID', 'SOPInstanceUID'],
 *     undefined,
 *     'StudyLevel'
 *   );
 *   
 *   const data: StudyLevelData = JSON.parse(json);
 *   
 *   // Insert or update study table (study-level data)
 *   await db.upsert('studies', data.studyLevel);
 *   
 *   // Insert instance table (instance-level data)
 *   await db.insert('instances', {
 *     ...data.instanceLevel,
 *     filePath: filePath
 *   });
 * }
 * ```
 */
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StudyLevelData {
    /// Study-level data: Patient + Study tags (persists across all instances)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_level: Option<HashMap<String, String>>,
    
    /// Instance-level data: Series + Instance + Equipment tags (varies per file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_level: Option<HashMap<String, String>>,
    
    /// Custom/private tags with user-defined names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

/// Main extraction result that can be serialized to different formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtractionResult {
    Scoped(ScopedDicomData),
    Flat(HashMap<String, String>),
    StudyLevel(StudyLevelData),
}

/// Extract DICOM tags with specified grouping strategy
pub fn extract_tags(
    obj: &DefaultDicomObject,
    tag_names: &[String],
    custom_tags: &[CustomTag],
    strategy: GroupingStrategy,
) -> ExtractionResult {
    match strategy {
        GroupingStrategy::ByScope => ExtractionResult::Scoped(extract_by_scope(obj, tag_names, custom_tags)),
        GroupingStrategy::Flat => ExtractionResult::Flat(extract_flat(obj, tag_names, custom_tags)),
        GroupingStrategy::StudyLevel => ExtractionResult::StudyLevel(extract_study_level(obj, tag_names, custom_tags)),
        GroupingStrategy::Custom => ExtractionResult::Scoped(extract_by_scope(obj, tag_names, custom_tags)), // Default to scope for now
    }
}

/// Extract tags grouped by DICOM scope
fn extract_by_scope(
    obj: &DefaultDicomObject,
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
    
    // Extract standard tags
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
    
    // Extract custom tags
    for custom_tag in custom_tags {
        if let Ok(tag) = parse_tag(&custom_tag.tag) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    custom.insert(custom_tag.name.clone(), value_str.to_string());
                }
            }
        }
    }
    
    // Only include non-empty maps
    if !patient.is_empty() { result.patient = Some(patient); }
    if !study.is_empty() { result.study = Some(study); }
    if !series.is_empty() { result.series = Some(series); }
    if !instance.is_empty() { result.instance = Some(instance); }
    if !equipment.is_empty() { result.equipment = Some(equipment); }
    if !custom.is_empty() { result.custom = Some(custom); }
    
    result
}

/// Extract tags into flat structure
fn extract_flat(
    obj: &DefaultDicomObject,
    tag_names: &[String],
    custom_tags: &[CustomTag],
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    // Extract standard tags
    for tag_name in tag_names {
        if let Ok(tag) = parse_tag(tag_name) {
            if let Ok(elem) = obj.element(tag) {
                if let Ok(value_str) = elem.to_str() {
                    result.insert(tag_name.clone(), value_str.to_string());
                }
            }
        }
    }
    
    // Extract custom tags
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

/// Extract tags grouped by study level (study-level vs instance-level)
fn extract_study_level(
    obj: &DefaultDicomObject,
    tag_names: &[String],
    custom_tags: &[CustomTag],
) -> StudyLevelData {
    let mut result = StudyLevelData::default();
    
    let mut study_level = HashMap::new();
    let mut instance_level = HashMap::new();
    let mut custom = HashMap::new();
    
    // Extract standard tags
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
    
    // Extract custom tags
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

/// Get tag value as string from DICOM object
pub fn get_tag_value(obj: &DefaultDicomObject, tag_str: &str) -> Option<String> {
    let tag = parse_tag(tag_str).ok()?;
    let elem = obj.element(tag).ok()?;
    elem.to_str().ok().map(|s| s.to_string())
}
