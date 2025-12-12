use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::cell::RefCell;
use dicom_dictionary_std::tags;
use dicom_core::header::Tag;
use dicom_object::{ open_file, DefaultDicomObject};
use snafu::prelude::*;
use napi::JsError;
use s3::Bucket;

#[cfg(feature = "transcode")]
use dicom_pixeldata::{DecodedPixelData, PixelDecoder};

use crate::utils::{extract_tags, CustomTag, GroupingStrategy, S3Config, build_s3_bucket, s3_get_object, s3_put_object};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(whatever, display("{}", message))]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + 'static>, Some)))]
        source: Option<Box<dyn std::error::Error + 'static>>,
    },
}

/// Storage backend type
#[napi(string_enum)]
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    /// Local filesystem storage
    Filesystem,
    /// S3-compatible object storage
    S3,
}

/// Storage configuration for DicomFile
#[derive(Debug, Clone)]
#[napi(object)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    /// Root directory for filesystem storage (relative or absolute path)
    pub root_dir: Option<String>,
    /// S3 configuration (required if backend is S3)
    pub s3_config: Option<S3Config>,
}

#[napi(object)]
pub struct DicomFileMeta {
    /// Storage SOP Class UID
    pub sop_class_uid: String,
    /// Storage SOP Instance UID
    pub sop_instance_uid: String,
}

/// Options for pixel data processing
#[napi(object)]
pub struct PixelDataOptions {
    /// Output file path
    pub output_path: String,
    
    /// Output format
    pub format: Option<PixelDataFormat>,
    
    /// Decode/decompress pixel data (requires transcode feature)
    pub decode: Option<bool>,
    
    /// Convert to 8-bit grayscale (requires decode=true)
    pub convert_to_8bit: Option<bool>,
    
    /// Apply VOI LUT (Value of Interest Lookup Table) for windowing
    pub apply_voi_lut: Option<bool>,
    
    /// Window center for manual windowing (overrides VOI LUT from file)
    pub window_center: Option<f64>,
    
    /// Window width for manual windowing (overrides VOI LUT from file)
    pub window_width: Option<f64>,
    
    /// Frame number to extract (0-based, for multi-frame images)
    pub frame_number: Option<u32>,
    
    /// Extract all frames as separate files (output_path will be used as template: path_{frame}.ext)
    pub extract_all_frames: Option<bool>,
}

/// Output format for pixel data
#[napi(string_enum)]
pub enum PixelDataFormat {
    /// Raw binary data (no processing)
    Raw,
    /// PNG image (requires decode=true)
    Png,
    /// JPEG image (requires decode=true)
    Jpeg,
    /// JSON metadata about pixel data
    Json,
}

/// Pixel data information
#[napi(object)]
pub struct PixelDataInfo {
    /// Width in pixels
    pub width: u32,
    
    /// Height in pixels
    pub height: u32,
    
    /// Number of frames
    pub frames: u32,
    
    /// Bits allocated per pixel
    pub bits_allocated: u16,
    
    /// Bits stored per pixel
    pub bits_stored: u16,
    
    /// High bit
    pub high_bit: u16,
    
    /// Pixel representation (0=unsigned, 1=signed)
    pub pixel_representation: u16,
    
    /// Samples per pixel (1=grayscale, 3=RGB)
    pub samples_per_pixel: u16,
    
    /// Photometric interpretation
    pub photometric_interpretation: String,
    
    /// Transfer syntax UID
    pub transfer_syntax_uid: String,
    
    /// Whether pixel data is compressed
    pub is_compressed: bool,
    
    /// Total pixel data size in bytes
    pub data_size: u32,
    
    /// Rescale intercept (for Hounsfield units in CT)
    pub rescale_intercept: Option<f64>,
    
    /// Rescale slope
    pub rescale_slope: Option<f64>,
    
    /// Window center from file
    pub window_center: Option<f64>,
    
    /// Window width from file
    pub window_width: Option<f64>,
}



#[napi]
pub struct DicomFile{
    /// DICOM object (wrapped in RefCell for interior mutability in async methods)
    dicom_file: RefCell<Option<DefaultDicomObject>>,
    /// Storage configuration
    storage_config: StorageConfig,
    /// S3 bucket instance (if using S3)
    s3_bucket: Option<Bucket>,
}

#[napi]
impl DicomFile {

    /**
     * Create a new DicomFile instance.
     * 
     * The instance is initially empty. Call `open()` to load a DICOM file.
     * 
     * @param storageConfig - Optional storage configuration for S3 or filesystem with root directory
     * 
     * @example
     * ```typescript
     * // Default filesystem storage (current directory)
     * const file1 = new DicomFile();
     * 
     * // Filesystem with root directory
     * const file2 = new DicomFile({
     *   backend: 'Filesystem',
     *   rootDir: '/data/dicom'
     * });
     * 
     * // S3 storage
     * const file3 = new DicomFile({
     *   backend: 'S3',
     *   s3Config: {
     *     bucket: 'my-dicom-bucket',
     *     accessKey: 'ACCESS_KEY',
     *     secretKey: 'SECRET_KEY',
     *     endpoint: 'http://localhost:9000'
     *   }
     * });
     * ```
     */
    #[napi(constructor)]
    pub fn new(storage_config: Option<StorageConfig>) -> Result<Self, JsError> {
        let config = storage_config.unwrap_or(StorageConfig {
            backend: StorageBackend::Filesystem,
            root_dir: None,
            s3_config: None,
        });
        
        // Validate and build S3 bucket if S3 backend is used
        let s3_bucket = if config.backend == StorageBackend::S3 {
            let s3_cfg = config.s3_config.as_ref()
                .ok_or_else(|| JsError::from(napi::Error::from_reason(
                    "S3 backend requires s3_config to be provided".to_string()
                )))?;
            
            Some(build_s3_bucket(s3_cfg))
        } else {
            None
        };
        
        Ok(DicomFile {
            dicom_file: RefCell::new(None),
            storage_config: config,
            s3_bucket,
        })
    }

    // Helper method to resolve file path with root_dir
    fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(root_dir) = &self.storage_config.root_dir {
            let root = PathBuf::from(root_dir);
            root.join(path)
        } else {
            PathBuf::from(path)
        }
    }

    // Helper method to read from S3
    async fn read_from_s3(&self, path: &str) -> Result<Vec<u8>, napi::Error> {
        let bucket = self.s3_bucket.as_ref()
            .ok_or_else(|| napi::Error::from_reason("S3 bucket not initialized".to_string()))?;
        
        let result = s3_get_object(bucket, path).await;
        match result {
            Ok(data) => Ok(data),
            Err(_e) => Err(napi::Error::from_reason(format!("Failed to read from S3: {}", path)))
        }
    }

    // Helper method to write to S3
    async fn write_to_s3(&self, path: &str, data: &[u8]) -> Result<(), napi::Error> {
        let bucket = self.s3_bucket.as_ref()
            .ok_or_else(|| napi::Error::from_reason("S3 bucket not initialized".to_string()))?;
        
        let result = s3_put_object(bucket, path, data).await;
        match result {
            Ok(()) => Ok(()),
            Err(_e) => Err(napi::Error::from_reason(format!("Failed to write to S3: {}", path)))
        }
    }

    /**
     * Check if a file is a valid DICOM file and extract its metadata.
     * 
     * This is a lightweight operation that only reads the file meta information
     * without loading the entire dataset. Useful for quickly validating files
     * or extracting SOPInstanceUID without a full file open.
     * 
     * @param path - Absolute or relative path to the DICOM file
     * @returns DicomFileMeta containing SOP Class UID and SOP Instance UID
     * @throws Error if the file is not a valid DICOM file or DICOMDIR
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * const meta = file.check('/path/to/file.dcm');
     * console.log(meta.sopInstanceUid);
     * ```
     */
    #[napi]
    pub fn check(&self, path: String) -> Result<DicomFileMeta, JsError> {
        let file = PathBuf::from(path);
        Self::check_file(file.as_path())
            .map_err(|e| JsError::from(napi::Error::from_reason(e.to_string())))
    }

    /**
     * Open and load a DICOM file into memory.
     * 
     * Reads the entire DICOM dataset and makes it available for operations like
     * `extract()`, `saveRawPixelData()`, and `dump()`. Any previously opened file
     * is automatically closed. Automatically uses S3 or filesystem based on configuration.
     * 
     * @param path - Path to the DICOM file (filesystem path or S3 key)
     * @returns Success message if the file was opened successfully
     * @throws Error if the file cannot be opened or is not a valid DICOM file
     * 
     * @example
     * ```typescript
     * // Filesystem
     * const file1 = new DicomFile();
     * file1.open('/path/to/file.dcm');
     * 
     * // S3
     * const file2 = new DicomFile({ backend: 'S3', s3Config: {...} });
     * file2.open('folder/file.dcm'); // Reads from S3 bucket
     * ```
     */
    #[napi]
    pub fn open(&self, path: String) -> napi::Result<String> {
        // Create a Tokio runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(async {
            match self.storage_config.backend {
                StorageBackend::S3 => {
                    // Read from S3
                    let data = self.read_from_s3(&path).await?;
                    
                    // Parse DICOM from bytes
                    let dicom_file = dicom_object::from_reader(&data[..])
                        .map_err(|e| napi::Error::from_reason(format!("Failed to parse DICOM from S3: {}", e)))?;
                    
                    *self.dicom_file.borrow_mut() = Some(dicom_file);
                    Ok(format!("File opened successfully from S3: {}", path))
                },
                StorageBackend::Filesystem => {
                    // Read from filesystem
                    let resolved_path = self.resolve_path(&path);
                    let dicom_file = open_file(&resolved_path)
                        .map_err(|e| napi::Error::from_reason(format!("Failed to open DICOM file: {}", e)))?;
                    
                    *self.dicom_file.borrow_mut() = Some(dicom_file);
                    Ok(format!("File opened successfully: {}", resolved_path.display()))
                }
            }
        })
    }

    /**
     * Open and load a DICOM JSON file into memory.
     * 
     * Reads a DICOM file in JSON format (as specified by DICOM Part 18) and converts it
     * to an internal DICOM object representation. After opening, all standard operations
     * like `extract()`, `dump()`, and `saveAsDicom()` are available. Automatically uses S3 or filesystem.
     * 
     * @param path - Path to the DICOM JSON file (filesystem path or S3 key)
     * @returns Success message if the file was opened successfully
     * @throws Error if the file cannot be opened or is not valid DICOM JSON
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * file.openJson('/path/to/file.json');
     * const data = file.extract(['PatientName', 'StudyDate'], undefined, 'Flat');
     * ```
     */
    #[napi]
    pub fn open_json(&self, path: String) -> napi::Result<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(async {
            // Read JSON content from S3 or filesystem
            let json_content = match self.storage_config.backend {
                StorageBackend::S3 => {
                    let data = self.read_from_s3(&path).await?;
                    String::from_utf8(data)
                        .map_err(|e| napi::Error::from_reason(format!("Invalid UTF-8 in JSON file: {}", e)))?
                },
                StorageBackend::Filesystem => {
                    let resolved_path = self.resolve_path(&path);
                    std::fs::read_to_string(&resolved_path)
                        .map_err(|e| napi::Error::from_reason(format!("Failed to read JSON file: {}", e)))?
                }
            };
            
            self.parse_and_set_json(json_content, &path)
        })
    }
    
    fn parse_and_set_json(&self, json_content: String, path: &str) -> napi::Result<String> {
        let json_content_ref = &json_content;
        
        // dicom_json::from_str returns InMemDicomObject directly
        let mem_obj = dicom_json::from_str::<dicom_object::InMemDicomObject>(json_content_ref)
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse DICOM JSON: {}", e)))?;
        
        // Create file meta information for DefaultDicomObject
        use dicom_object::FileMetaTable;
        use dicom_dictionary_std::uids;
        
        // Extract or create necessary meta information
        let sop_class_uid = mem_obj.element(tags::SOP_CLASS_UID)
            .ok()
            .and_then(|e| e.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uids::SECONDARY_CAPTURE_IMAGE_STORAGE.to_string());
        
        let sop_instance_uid = mem_obj.element(tags::SOP_INSTANCE_UID)
            .ok()
            .and_then(|e| e.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "1.2.3.4.5.6.7.8.9".to_string());
        
        // Create file meta table
        let meta = FileMetaTable {
            information_group_length: 0, // Will be calculated on write
            information_version: [0, 1],
            media_storage_sop_class_uid: sop_class_uid,
            media_storage_sop_instance_uid: sop_instance_uid,
            transfer_syntax: uids::EXPLICIT_VR_LITTLE_ENDIAN.to_string(),
            implementation_class_uid: "1.2.826.0.1.3680043.9.7433.1.1".to_string(),
            implementation_version_name: Some("node-dicom-rs".to_string()),
            source_application_entity_title: None,
            sending_application_entity_title: None,
            receiving_application_entity_title: None,
            private_information_creator_uid: None,
            private_information: None,
        };
        
        // Create FileDicomObject with meta and copy data from mem_obj
        use dicom_object::FileDicomObject;
        let mut dicom_obj = FileDicomObject::new_empty_with_dict_and_meta(
            dicom_object::StandardDataDictionary,
            meta
        );
        
        // Copy all elements from mem_obj to dicom_obj
        for elem in mem_obj.into_iter() {
            let _ = dicom_obj.put(elem);
        }
        
        *self.dicom_file.borrow_mut() = Some(dicom_obj);
        Ok(format!("DICOM JSON file opened successfully: {}", path))
    }

    /**
     * Print a detailed dump of the DICOM file structure to stdout.
     * 
     * Displays all DICOM elements with their tags, VRs, and values in a human-readable format.
     * Useful for debugging and inspecting DICOM file contents.
     * 
     * @throws Error if no file is currently opened
     */
    #[napi]
    pub fn dump(&self) -> Result<(), JsError> {
        if self.dicom_file.borrow().is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened. Call open() first.".to_string())));
        }
        let dicom_ref = self.dicom_file.borrow();
        dicom_dump::dump_file(dicom_ref.as_ref().unwrap())
            .map_err(|e| JsError::from(napi::Error::from_reason(e.to_string())))
    }

    /**
     * Extract and save raw pixel data to a file.
     * 
     * Extracts the raw pixel data bytes from the DICOM file's PixelData element (7FE0,0010)
     * and writes them directly to a binary file. The data is saved as-is without any
     * decompression or conversion. Useful for extracting raw image data for custom processing.
     * 
     * Note: This does not decode or decompress the pixel data. For compressed transfer syntaxes
     * (e.g., JPEG, JPEG 2000), the output will be the compressed bitstream.
     * 
     * @param path - Output path where the raw pixel data will be saved
     * @returns Success message with the number of bytes written
     * @throws Error if no file is opened, pixel data is missing, or file write fails
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * file.open('image.dcm');
     * file.saveRawPixelData('output.raw');
     * ```
     */
    #[napi]
    pub fn save_raw_pixel_data(&self, path: String) -> Result<String, JsError> {
        if self.dicom_file.borrow().is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened. Call open() first.".to_string())));
        }
        
        let dicom_ref = self.dicom_file.borrow();
        let obj = dicom_ref.as_ref().unwrap();
        let pixel_data = obj.element(tags::PIXEL_DATA)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Pixel data not found: {}", e))))?;
        
        let data = pixel_data.to_bytes()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read pixel data: {}", e))))?;
        
        std::fs::write(&path, &data)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to write file: {}", e))))?;
        
        Ok(format!("Pixel data saved successfully ({} bytes)", data.len()))
    }

    /**
     * Extract DICOM tags with flexible grouping strategies.
     * 
     * Extracts specified DICOM tags and returns them as a JSON string, organized according
     * to the chosen grouping strategy. Supports any tag name from the DICOM standard or
     * hex format (e.g., "00100010").
     * 
     * ## Grouping Strategies
     * 
     * - **"ByScope"** (default): Groups tags by DICOM hierarchy levels (Patient, Study, Series, Instance, Equipment)
     * - **"Flat"**: Returns all tags in a flat key-value structure
     * - **"StudyLevel"**: Groups into studyLevel (Patient+Study) and instanceLevel (Series+Instance+Equipment)
     * - **"Custom"**: Reserved for user-defined grouping rules (currently behaves like ByScope)
     * 
     * ## Tag Name Formats
     * 
     * Tags can be specified in multiple formats:
     * - Standard name: "PatientName", "StudyDate", "Modality"
     * - Hex format: "00100010", "00080020", "00080060"
     * - Any valid DICOM tag from StandardDataDictionary
     * 
     * ## Custom Tags
     * 
     * Custom tags allow extraction of private or vendor-specific tags with user-defined names:
     * ```typescript
     * import { createCustomTag } from '@nuxthealth/node-dicom';
     * 
     * file.extract(
     *   ['PatientName'],
     *   [createCustomTag('00091001', 'VendorPrivateTag')],
     *   'ByScope'
     * );
     * ```
     * 
     * @param tagNames - Array of DICOM tag names or hex values to extract. Supports 300+ autocomplete suggestions.
     * @param customTags - Optional array of custom tag specifications for private/vendor tags
     * @param strategy - Grouping strategy: "ByScope" | "Flat" | "StudyLevel" | "Custom" (default: "ByScope")
     * @returns JSON string containing extracted tags, structure depends on grouping strategy
     * @throws Error if no file is opened or JSON serialization fails
     * 
     * @example
     * ```typescript
     * // Scoped grouping (default)
     * const json = file.extract(['PatientName', 'StudyDate', 'Modality'], undefined, 'ByScope');
     * const data = JSON.parse(json);
     * // { patient: { PatientName: "..." }, study: { StudyDate: "..." }, series: { Modality: "..." } }
     * 
     * // Flat structure
     * const flatJson = file.extract(['PatientName', 'StudyDate'], undefined, 'Flat');
     * // { "PatientName": "...", "StudyDate": "..." }
     * 
     * // Use predefined tag sets
     * import { getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';
     * const tags = getCommonTagSets();
     * const allTags = combineTags([tags.patientBasic, tags.studyBasic, tags.ct]);
     * const extracted = file.extract(allTags, undefined, 'StudyLevel');
     * ```
     */
    #[napi(
        ts_args_type = "tagNames: Array<'AccessionNumber' | 'AcquisitionDate' | 'AcquisitionDateTime' | 'AcquisitionNumber' | 'AcquisitionTime' | 'ActualCardiacTriggerTimePriorToRPeak' | 'ActualFrameDuration' | 'AdditionalPatientHistory' | 'AdmissionID' | 'AdmittingDiagnosesDescription' | 'AnatomicalOrientationType' | 'AnatomicRegionSequence' | 'AnodeTargetMaterial' | 'BeamLimitingDeviceAngle' | 'BitsAllocated' | 'BitsStored' | 'BluePaletteColorLookupTableDescriptor' | 'BodyPartExamined' | 'BodyPartThickness' | 'BranchOfService' | 'BurnedInAnnotation' | 'ChannelSensitivity' | 'CineRate' | 'CollimatorType' | 'Columns' | 'CompressionForce' | 'ContentDate' | 'ContentTime' | 'ContrastBolusAgent' | 'ContrastBolusIngredient' | 'ContrastBolusIngredientConcentration' | 'ContrastBolusRoute' | 'ContrastBolusStartTime' | 'ContrastBolusStopTime' | 'ContrastBolusTotalDose' | 'ContrastBolusVolume' | 'ContrastFlowDuration' | 'ContrastFlowRate' | 'ConvolutionKernel' | 'CorrectedImage' | 'CountsSource' | 'DataCollectionDiameter' | 'DecayCorrection' | 'DeidentificationMethod' | 'DerivationDescription' | 'DetectorTemperature' | 'DeviceSerialNumber' | 'DistanceSourceToDetector' | 'DistanceSourceToPatient' | 'EchoTime' | 'EthnicGroup' | 'Exposure' | 'ExposureInMicroAmpereSeconds' | 'ExposureTime' | 'FilterType' | 'FlipAngle' | 'FocalSpots' | 'FrameDelay' | 'FrameIncrementPointer' | 'FrameOfReferenceUID' | 'FrameTime' | 'GantryAngle' | 'GeneratorPower' | 'GraphicAnnotationSequence' | 'GreenPaletteColorLookupTableDescriptor' | 'HeartRate' | 'HighBit' | 'ImageComments' | 'ImageLaterality' | 'ImageOrientationPatient' | 'ImagePositionPatient' | 'ImagerPixelSpacing' | 'ImageTriggerDelay' | 'ImageType' | 'ImagingFrequency' | 'ImplementationClassUID' | 'ImplementationVersionName' | 'InstanceCreationDate' | 'InstanceCreationTime' | 'InstanceNumber' | 'InstitutionName' | 'IntensifierSize' | 'IssuerOfAdmissionID' | 'KVP' | 'LargestImagePixelValue' | 'LargestPixelValueInSeries' | 'Laterality' | 'LossyImageCompression' | 'LossyImageCompressionMethod' | 'LossyImageCompressionRatio' | 'MagneticFieldStrength' | 'Manufacturer' | 'ManufacturerModelName' | 'MedicalRecordLocator' | 'MilitaryRank' | 'Modality' | 'MultiplexGroupTimeOffset' | 'NameOfPhysiciansReadingStudy' | 'NominalCardiacTriggerDelayTime' | 'NominalInterval' | 'NumberOfFrames' | 'NumberOfSlices' | 'NumberOfTemporalPositions' | 'NumberOfWaveformChannels' | 'NumberOfWaveformSamples' | 'Occupation' | 'OperatorsName' | 'OtherPatientIDs' | 'OtherPatientNames' | 'OverlayBitPosition' | 'OverlayBitsAllocated' | 'OverlayColumns' | 'OverlayData' | 'OverlayOrigin' | 'OverlayRows' | 'OverlayType' | 'PaddleDescription' | 'PatientAge' | 'PatientBirthDate' | 'PatientBreedDescription' | 'PatientComments' | 'PatientID' | 'PatientIdentityRemoved' | 'PatientName' | 'PatientPosition' | 'PatientSex' | 'PatientSize' | 'PatientSpeciesDescription' | 'PatientSupportAngle' | 'PatientTelephoneNumbers' | 'PatientWeight' | 'PerformedProcedureStepDescription' | 'PerformedProcedureStepID' | 'PerformedProcedureStepStartDate' | 'PerformedProcedureStepStartTime' | 'PerformedProtocolCodeSequence' | 'PerformingPhysicianName' | 'PhotometricInterpretation' | 'PhysiciansOfRecord' | 'PixelAspectRatio' | 'PixelPaddingRangeLimit' | 'PixelPaddingValue' | 'PixelRepresentation' | 'PixelSpacing' | 'PlanarConfiguration' | 'PositionerPrimaryAngle' | 'PositionerSecondaryAngle' | 'PositionReferenceIndicator' | 'PreferredPlaybackSequencing' | 'PresentationIntentType' | 'PresentationLUTShape' | 'PrimaryAnatomicStructureSequence' | 'PrivateInformationCreatorUID' | 'ProtocolName' | 'QualityControlImage' | 'RadiationMachineName' | 'RadiationSetting' | 'RadionuclideTotalDose' | 'RadiopharmaceuticalInformationSequence' | 'RadiopharmaceuticalStartDateTime' | 'RadiopharmaceuticalStartTime' | 'RadiopharmaceuticalVolume' | 'ReasonForTheRequestedProcedure' | 'ReceivingApplicationEntityTitle' | 'RecognizableVisualFeatures' | 'RecommendedDisplayFrameRate' | 'ReconstructionDiameter' | 'ReconstructionTargetCenterPatient' | 'RedPaletteColorLookupTableDescriptor' | 'ReferencedBeamNumber' | 'ReferencedImageSequence' | 'ReferencedPatientPhotoSequence' | 'ReferencedPerformedProcedureStepSequence' | 'ReferencedRTPlanSequence' | 'ReferencedSOPClassUID' | 'ReferencedSOPInstanceUID' | 'ReferencedStudySequence' | 'ReferringPhysicianName' | 'RepetitionTime' | 'RequestAttributesSequence' | 'RequestedContrastAgent' | 'RequestedProcedureDescription' | 'RequestedProcedureID' | 'RequestingPhysician' | 'RescaleIntercept' | 'RescaleSlope' | 'RescaleType' | 'ResponsibleOrganization' | 'ResponsiblePerson' | 'ResponsiblePersonRole' | 'Rows' | 'RTImageDescription' | 'RTImageLabel' | 'SamplesPerPixel' | 'SamplingFrequency' | 'ScanningSequence' | 'SendingApplicationEntityTitle' | 'SeriesDate' | 'SeriesDescription' | 'SeriesInstanceUID' | 'SeriesNumber' | 'SeriesTime' | 'SeriesType' | 'SliceLocation' | 'SliceThickness' | 'SmallestImagePixelValue' | 'SmallestPixelValueInSeries' | 'SoftwareVersions' | 'SOPClassUID' | 'SOPInstanceUID' | 'SoundPathLength' | 'SourceApplicationEntityTitle' | 'SourceImageSequence' | 'SpacingBetweenSlices' | 'SpecificCharacterSet' | 'StationName' | 'StudyComments' | 'StudyDate' | 'StudyDescription' | 'StudyID' | 'StudyInstanceUID' | 'StudyTime' | 'TableHeight' | 'TableTopLateralPosition' | 'TableTopLongitudinalPosition' | 'TableTopVerticalPosition' | 'TableType' | 'TemporalPositionIdentifier' | 'TemporalResolution' | 'TextObjectSequence' | 'TimezoneOffsetFromUTC' | 'TransducerFrequency' | 'TransducerType' | 'TransferSyntaxUID' | 'TriggerTime' | 'TriggerTimeOffset' | 'UltrasoundColorDataPresent' | 'Units' | 'VOILUTFunction' | 'WaveformOriginality' | 'WaveformSequence' | 'WindowCenter' | 'WindowCenterWidthExplanation' | 'WindowWidth' | 'XRayTubeCurrent' | (string & {})>, customTags?: Array<CustomTag>, strategy?: 'ByScope' | 'Flat' | 'StudyLevel' | 'Custom'"
    )]
    pub fn extract(
        &self, 
        tag_names: Vec<String>, 
        custom_tags: Option<Vec<CustomTag>>,
        strategy: Option<String>
    ) -> Result<String, JsError> {
        if self.dicom_file.borrow().is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened. Call open() first.".to_string())));
        }
        
        let dicom_ref = self.dicom_file.borrow();
        let obj = dicom_ref.as_ref().unwrap();
        let custom = custom_tags.unwrap_or_default();
        let grouping = match strategy.as_deref() {
            Some("Flat") => GroupingStrategy::Flat,
            Some("StudyLevel") => GroupingStrategy::StudyLevel,
            Some("Custom") => GroupingStrategy::Custom,
            _ => GroupingStrategy::ByScope,
        };
        
        let result = extract_tags(obj, &tag_names, &custom, grouping);
        
        serde_json::to_string(&result)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to serialize result: {}", e))))
    }

    /**
     * Get comprehensive information about pixel data in the DICOM file.
     * 
     * Extracts metadata about the image dimensions, bit depth, photometric interpretation,
     * compression status, and windowing parameters without decoding the actual pixel data.
     * 
     * @returns PixelDataInfo object with detailed pixel data metadata
     * @throws Error if no file is opened or pixel data is missing
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * file.open('ct_scan.dcm');
     * const info = file.getPixelDataInfo();
     * console.log(`${info.width}x${info.height}, ${info.frames} frames`);
     * console.log(`Bits: ${info.bitsStored}, Compressed: ${info.isCompressed}`);
     * ```
     */
    #[napi]
    pub fn get_pixel_data_info(&self) -> Result<PixelDataInfo, JsError> {
        if self.dicom_file.borrow().is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened. Call open() first.".to_string())));
        }
        
        let dicom_ref = self.dicom_file.borrow();
        let obj = dicom_ref.as_ref().unwrap();
        
        // Get required attributes
        let rows = obj.element(tags::ROWS)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read Rows: {}", e))))?  
            .to_int::<u32>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert Rows: {}", e))))?;
        
        let columns = obj.element(tags::COLUMNS)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read Columns: {}", e))))?  
            .to_int::<u32>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert Columns: {}", e))))?;
        
        let bits_allocated = obj.element(tags::BITS_ALLOCATED)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read BitsAllocated: {}", e))))?  
            .to_int::<u16>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert BitsAllocated: {}", e))))?;
        
        let bits_stored = obj.element(tags::BITS_STORED)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read BitsStored: {}", e))))?  
            .to_int::<u16>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert BitsStored: {}", e))))?;
        
        let high_bit = obj.element(tags::HIGH_BIT)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read HighBit: {}", e))))?  
            .to_int::<u16>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert HighBit: {}", e))))?;
        
        let pixel_representation = obj.element(tags::PIXEL_REPRESENTATION)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read PixelRepresentation: {}", e))))?  
            .to_int::<u16>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert PixelRepresentation: {}", e))))?;
        
        let samples_per_pixel = obj.element(tags::SAMPLES_PER_PIXEL)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read SamplesPerPixel: {}", e))))?  
            .to_int::<u16>()
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert SamplesPerPixel: {}", e))))?;
        
        let photometric_interpretation = obj.element(tags::PHOTOMETRIC_INTERPRETATION)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read PhotometricInterpretation: {}", e))))?  
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert PhotometricInterpretation: {}", e))))?;
        
        // Optional attributes
        let frames = obj.element(tags::NUMBER_OF_FRAMES)
            .ok()
            .and_then(|e| e.to_int::<u32>().ok())
            .unwrap_or(1);
        
        let transfer_syntax_uid = obj.meta().transfer_syntax.trim_end_matches('\0').to_string();
        
        // Check if compressed (common compressed transfer syntaxes)
        let is_compressed = !transfer_syntax_uid.starts_with("1.2.840.10008.1.2.1") // Explicit VR Little Endian
            && !transfer_syntax_uid.starts_with("1.2.840.10008.1.2.2") // Explicit VR Big Endian
            && transfer_syntax_uid != "1.2.840.10008.1.2"; // Implicit VR Little Endian
        
        // Get pixel data size
        let pixel_data = obj.element(tags::PIXEL_DATA)
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Pixel data not found: {}", e))))?;
        
        let data_size = pixel_data.to_bytes()
            .map(|b| b.len() as u32)
            .unwrap_or(0);
        
        // Optional windowing parameters
        let rescale_intercept = obj.element(tags::RESCALE_INTERCEPT)
            .ok()
            .and_then(|e| e.to_float64().ok());
        
        let rescale_slope = obj.element(tags::RESCALE_SLOPE)
            .ok()
            .and_then(|e| e.to_float64().ok());
        
        let window_center = obj.element(tags::WINDOW_CENTER)
            .ok()
            .and_then(|e| e.to_float64().ok());
        
        let window_width = obj.element(tags::WINDOW_WIDTH)
            .ok()
            .and_then(|e| e.to_float64().ok());
        
        Ok(PixelDataInfo {
            width: columns,
            height: rows,
            frames,
            bits_allocated,
            bits_stored,
            high_bit,
            pixel_representation,
            samples_per_pixel,
            photometric_interpretation,
            transfer_syntax_uid,
            is_compressed,
            data_size,
            rescale_intercept,
            rescale_slope,
            window_center,
            window_width,
        })
    }

    /**
     * Process and extract pixel data with flexible options.
     * 
     * Advanced pixel data processing supporting:
     * - Raw extraction or decoded/decompressed output
     * - Multiple output formats (Raw, PNG, JPEG, JSON)
     * - Frame extraction (single or all frames)
     * - Windowing and 8-bit conversion
     * - VOI LUT application
     * 
     * @param options - Processing options
     * @returns Success message with processing details
     * @throws Error if processing fails or required features are not available
     * 
     * @example
     * ```typescript
     * // Extract raw pixel data
     * file.processPixelData({
     *   outputPath: 'output.raw',
     *   format: 'Raw'
     * });
     * 
     * // Decode and save as PNG with windowing
     * file.processPixelData({
     *   outputPath: 'output.png',
     *   format: 'Png',
     *   decode: true,
     *   applyVoiLut: true,
     *   convertTo8bit: true
     * });
     * 
     * // Extract specific frame
     * file.processPixelData({
     *   outputPath: 'frame_5.raw',
     *   format: 'Raw',
     *   frameNumber: 5
     * });
     * 
     * // Get metadata as JSON
     * file.processPixelData({
     *   outputPath: 'info.json',
     *   format: 'Json'
     * });
     * ```
     */
    #[napi]
    pub fn process_pixel_data(&self, options: PixelDataOptions) -> Result<String, JsError> {
        if self.dicom_file.borrow().is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened. Call open() first.".to_string())));
        }
        
        let dicom_ref = self.dicom_file.borrow();
        let obj = dicom_ref.as_ref().unwrap();
        let format = options.format.unwrap_or(PixelDataFormat::Raw);
        let decode = options.decode.unwrap_or(false);
        
        // Handle JSON format - just return metadata
        if matches!(format, PixelDataFormat::Json) {
            let info = self.get_pixel_data_info()?;
            let json = serde_json::to_string_pretty(&serde_json::json!({
                "width": info.width,
                "height": info.height,
                "frames": info.frames,
                "bitsAllocated": info.bits_allocated,
                "bitsStored": info.bits_stored,
                "highBit": info.high_bit,
                "pixelRepresentation": info.pixel_representation,
                "samplesPerPixel": info.samples_per_pixel,
                "photometricInterpretation": info.photometric_interpretation,
                "transferSyntaxUid": info.transfer_syntax_uid,
                "isCompressed": info.is_compressed,
                "dataSizeBytes": info.data_size,
                "rescaleIntercept": info.rescale_intercept,
                "rescaleSlope": info.rescale_slope,
                "windowCenter": info.window_center,
                "windowWidth": info.window_width,
            }))
            .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to serialize JSON: {}", e))))?;
            
            std::fs::write(&options.output_path, json)
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to write JSON file: {}", e))))?;
            
            return Ok(format!("Pixel data metadata saved to {}", options.output_path));
        }
        
        // Handle raw format without decoding
        if matches!(format, PixelDataFormat::Raw) && !decode {
            let pixel_data = obj.element(tags::PIXEL_DATA)
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Pixel data not found: {}", e))))?;
            
            let data = pixel_data.to_bytes()
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read pixel data: {}", e))))?;
            
            std::fs::write(&options.output_path, &data)
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to write file: {}", e))))?;
            
            return Ok(format!("Raw pixel data saved to {} ({} bytes)", options.output_path, data.len()));
        }
        
        // Decoding required beyond this point
        #[cfg(not(feature = "transcode"))]
        {
            return Err(JsError::from(napi::Error::from_reason(
                "Pixel data decoding requires the 'transcode' feature. Rebuild with --features transcode".to_string()
            )));
        }
        
        #[cfg(feature = "transcode")]
        {
            // Decode pixel data
            let decoded = obj.decode_pixel_data()
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to decode pixel data: {}", e))))?;
            
            // Get pixel data info
            let info = self.get_pixel_data_info()?;
            
            // Convert decoded data to bytes
            // The DecodedPixelData provides methods to access pixel values
            // For simplicity, we'll convert to a raw byte representation
            let bytes = decoded.to_vec()
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to convert pixel data: {}", e))))?;
            
            // Handle frame extraction if requested
            if let Some(frame_num) = options.frame_number {
                if frame_num >= info.frames {
                    return Err(JsError::from(napi::Error::from_reason(
                        format!("Frame number {} out of range (0-{})", frame_num, info.frames - 1)
                    )));
                }
                // TODO: Extract specific frame
                return Err(JsError::from(napi::Error::from_reason(
                    "Frame extraction not yet implemented. Use decode=false for raw extraction.".to_string()
                )));
            }
            
            // Save decoded data
            std::fs::write(&options.output_path, &bytes)
                .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to write file: {}", e))))?;
            
            Ok(format!(
                "Decoded pixel data saved to {} ({} bytes, {}x{}, {} frames)",
                options.output_path,
                bytes.len(),
                info.width,
                info.height,
                info.frames
            ))
        }
    }

    /**
     * Save the currently opened DICOM file as JSON format.
     * 
     * Converts the DICOM object to JSON representation according to DICOM Part 18
     * standard and saves it to the specified path. Automatically uses S3 or filesystem.
     * 
     * @param path - Output path for the JSON file (filesystem path or S3 key)
     * @param pretty - Pretty print the JSON (default: true)
     * @returns Success message with file size
     * @throws Error if no file is opened or JSON conversion fails
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * file.open('image.dcm');
     * file.saveAsJson('output.json', true);
     * ```
     */
    #[napi]
    pub fn save_as_json(&self, path: String, pretty: Option<bool>) -> napi::Result<String> {
        if self.dicom_file.borrow().is_none() {
            return Err(napi::Error::from_reason("File not opened. Call open() first.".to_string()));
        }
        
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(async {
            let dicom_ref = self.dicom_file.borrow();
            let obj = dicom_ref.as_ref().unwrap();
            let pretty_print = pretty.unwrap_or(true);
            
            let json_string = if pretty_print {
                dicom_json::to_string_pretty(obj)
                    .map_err(|e| napi::Error::from_reason(format!("Failed to convert to JSON: {}", e)))?
            } else {
                dicom_json::to_string(obj)
                    .map_err(|e| napi::Error::from_reason(format!("Failed to convert to JSON: {}", e)))?
            };
            
            match self.storage_config.backend {
                StorageBackend::S3 => {
                    self.write_to_s3(&path, json_string.as_bytes()).await?;
                    Ok(format!("DICOM saved as JSON to S3: {} ({} bytes)", path, json_string.len()))
                },
                StorageBackend::Filesystem => {
                    let resolved_path = self.resolve_path(&path);
                    std::fs::write(&resolved_path, &json_string)
                        .map_err(|e| napi::Error::from_reason(format!("Failed to write JSON file: {}", e)))?;
                    Ok(format!("DICOM saved as JSON to {} ({} bytes)", resolved_path.display(), json_string.len()))
                }
            }
        })
    }    /**
     * Save the currently opened DICOM file (regardless of original format) as standard DICOM.
     * 
     * Writes the DICOM object as a standard .dcm file with proper file meta information.
     * Useful for converting DICOM JSON back to binary DICOM format. Automatically uses S3 or filesystem.
     * 
     * @param path - Output path for the DICOM file (filesystem path or S3 key)
     * @returns Success message
     * @throws Error if no file is opened or write fails
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * file.openJson('input.json');
     * file.saveAsDicom('output.dcm');
     * ```
     */
    #[napi]
    pub fn save_as_dicom(&self, path: String) -> napi::Result<String> {
        if self.dicom_file.borrow().is_none() {
            return Err(napi::Error::from_reason("File not opened. Call open() first.".to_string()));
        }
        
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(async {
            let dicom_ref = self.dicom_file.borrow();
            let obj = dicom_ref.as_ref().unwrap();
            
            match self.storage_config.backend {
                StorageBackend::S3 => {
                    // Write to buffer first
                    let mut buffer = Vec::new();
                    obj.write_all(&mut buffer)
                        .map_err(|e| napi::Error::from_reason(format!("Failed to write DICOM to buffer: {}", e)))?;
                    
                    // Upload to S3
                    self.write_to_s3(&path, &buffer).await?;
                    Ok(format!("DICOM file saved to S3: {} ({} bytes)", path, buffer.len()))
                },
                StorageBackend::Filesystem => {
                    let resolved_path = self.resolve_path(&path);
                    obj.write_to_file(&resolved_path)
                        .map_err(|e| napi::Error::from_reason(format!("Failed to write DICOM file: {}", e)))?;
                    Ok(format!("DICOM file saved to {}", resolved_path.display()))
                }
            }
        })
    }

    /**
     * Close the currently opened DICOM file and free memory.
     * 
     * Releases the DICOM dataset from memory. After closing, you must call `open()`
     * again before performing any operations that require file data. It's good practice
     * to close files when done to free resources, though the file will be automatically
     * closed when the instance is dropped.
     * 
     * @example
     * ```typescript
     * const file = new DicomFile();
     * file.open('file1.dcm');
     * // ... work with file1
     * file.close();
     * 
     * file.open('file2.dcm');  // Can reuse same instance
     * // ... work with file2
     * file.close();
     * ```
     */
    #[napi]
    pub fn close(&self) {
        *self.dicom_file.borrow_mut() = None;
    }

    fn check_file(file: &Path) -> Result<DicomFileMeta, Error> {
        // Ignore DICOMDIR files until better support is added
        let _ = (file.file_name() != Some(OsStr::new("DICOMDIR")))
            .then_some(false)
            .whatever_context("DICOMDIR file not supported")?;
        let dicom_file = dicom_object::OpenFileOptions::new()
            .read_until(Tag(0x0001, 0x000))
            .open_file(file)
            .with_whatever_context(|_| format!("Could not open DICOM file {}", file.display()))?;

        let meta = dicom_file.meta();

        let storage_sop_class_uid = &meta.media_storage_sop_class_uid;
        let storage_sop_instance_uid = &meta.media_storage_sop_instance_uid;

        Ok(DicomFileMeta {
            sop_class_uid: storage_sop_class_uid.to_string(),
            sop_instance_uid: storage_sop_instance_uid.to_string(),
        })
    }
}

/**
 * Standalone utility to extract raw pixel data from a DICOM file.
 * 
 * This is a convenience function that opens a DICOM file, extracts the raw pixel data,
 * and saves it to a file in a single operation. For repeated operations on the same file,
 * prefer using the DicomFile class with `open()` and `saveRawPixelData()`.
 * 
 * @param filePath - Path to the source DICOM file
 * @param outPath - Path where the raw pixel data will be saved
 * @returns Success message with the number of bytes written
 * @throws Error if the file cannot be opened, pixel data is missing, or write fails
 * 
 * @example
 * ```typescript
 * import { saveRawPixelData } from '@nuxthealth/node-dicom';
 * 
 * saveRawPixelData('/path/to/image.dcm', '/path/to/output.raw');
 * ```
 */
#[napi]
pub fn save_raw_pixel_data(file_path: String, out_path: String) -> Result<String, JsError> {
    let file = PathBuf::from(file_path);
    let dicom_file = open_file(file)
        .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to open DICOM file: {}", e))))?;

    let pixel_data = dicom_file.element(tags::PIXEL_DATA)
        .map_err(|e| JsError::from(napi::Error::from_reason(format!("Pixel data not found: {}", e))))?;

    let data = pixel_data.to_bytes()
        .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to read pixel data: {}", e))))?;

    std::fs::write(&out_path, &data)
        .map_err(|e| JsError::from(napi::Error::from_reason(format!("Failed to write file: {}", e))))?;

    Ok(format!("Pixel data saved successfully ({} bytes)", data.len()))
}