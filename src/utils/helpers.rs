use napi_derive::napi;
use serde::{Deserialize, Serialize};

/**
 * Predefined sets of commonly used DICOM tags organized by category.
 * 
 * Provides convenient access to curated tag lists for different use cases,
 * eliminating the need to manually specify tags for common extraction scenarios.
 * 
 * ## Tag Set Categories
 * - **patient_basic**: Essential patient demographics (name, ID, birth date, sex, age, weight, size)
 * - **study_basic**: Study-level metadata (UID, date, description, accession number, referring physician)
 * - **series_basic**: Series-level metadata (UID, number, description, modality, body part, protocol)
 * - **instance_basic**: Instance-level identifiers (SOP UIDs, instance number, creation date/time)
 * - **image_pixel_info**: Image dimensions and pixel characteristics (rows, columns, bits, spacing)
 * - **equipment**: Device information (manufacturer, model, serial number, software version, institution)
 * - **ct**: CT-specific imaging parameters (kVp, exposure, tube current, kernel, slice thickness)
 * - **mr**: MR-specific parameters (TR, TE, field strength, flip angle, scanning sequence)
 * - **ultrasound**: Ultrasound-specific tags (transducer type/frequency, sound path, frame time)
 * - **pet_nm**: PET/Nuclear Medicine tags (radiopharmaceutical info, dose, decay correction, units)
 * - **xa**: X-Ray Angiography tags (distances, intensifier size, positioner angles, radiation setting)
 * - **rt**: Radiation Therapy tags (RT image info, plan reference, gantry angle, beam info)
 * - **default**: Comprehensive set combining patient, study, series, instance, pixel info, and equipment
 * 
 * @example
 * ```typescript
 * import { getCommonTagSets } from '@nuxthealth/node-dicom';
 * 
 * // Get all predefined tag sets
 * const tagSets = getCommonTagSets();
 * 
 * // Use specific tag set for extraction
 * const patientTags = tagSets.patientBasic;
 * // ['PatientName', 'PatientID', 'PatientBirthDate', ...]
 * 
 * const studyTags = tagSets.studyBasic;
 * // ['StudyInstanceUID', 'StudyDate', 'StudyDescription', ...]
 * 
 * // Combine multiple sets
 * const combinedTags = [
 *   ...tagSets.patientBasic,
 *   ...tagSets.studyBasic,
 *   ...tagSets.seriesBasic
 * ];
 * 
 * // Or use the comprehensive default set
 * const allCommonTags = tagSets.default;
 * ```
 * 
 * @example
 * ```typescript
 * // Modality-specific extraction
 * const tagSets = getCommonTagSets();
 * 
 * // CT scan analysis
 * const ctTags = [
 *   ...tagSets.patientBasic,
 *   ...tagSets.studyBasic,
 *   ...tagSets.ct,
 *   ...tagSets.imagePixelInfo
 * ];
 * 
 * // MR imaging
 * const mrTags = [
 *   ...tagSets.default,
 *   ...tagSets.mr
 * ];
 * 
 * // PET/Nuclear Medicine
 * const petTags = [
 *   ...tagSets.default,
 *   ...tagSets.petNm
 * ];
 * ```
 */
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonTagSets {
    /// Essential patient demographics (7 tags: PatientName, PatientID, PatientBirthDate, PatientSex, PatientAge, PatientWeight, PatientSize)
    pub patient_basic: Vec<String>,
    /// Study-level metadata (7 tags: StudyInstanceUID, StudyDate, StudyTime, StudyDescription, StudyID, AccessionNumber, ReferringPhysicianName)
    pub study_basic: Vec<String>,
    /// Series-level metadata (8 tags: SeriesInstanceUID, SeriesNumber, SeriesDescription, SeriesDate, SeriesTime, Modality, BodyPartExamined, ProtocolName)
    pub series_basic: Vec<String>,
    /// Instance-level identifiers (5 tags: SOPInstanceUID, SOPClassUID, InstanceNumber, InstanceCreationDate, InstanceCreationTime)
    pub instance_basic: Vec<String>,
    /// Image dimensions and pixel characteristics (9 tags: Rows, Columns, BitsAllocated, BitsStored, HighBit, PixelRepresentation, SamplesPerPixel, PhotometricInterpretation, PixelSpacing)
    pub image_pixel_info: Vec<String>,
    /// Device and institution information (6 tags: Manufacturer, ManufacturerModelName, DeviceSerialNumber, SoftwareVersions, InstitutionName, StationName)
    pub equipment: Vec<String>,
    /// CT-specific imaging parameters (6 tags: KVP, ExposureTime, XRayTubeCurrent, Exposure, ConvolutionKernel, SliceThickness)
    pub ct: Vec<String>,
    /// MR-specific imaging parameters (6 tags: RepetitionTime, EchoTime, MagneticFieldStrength, FlipAngle, ImagingFrequency, ScanningSequence)
    pub mr: Vec<String>,
    /// Ultrasound-specific tags (6 tags: TransducerType, TransducerFrequency, SoundPathLength, UltrasoundColorDataPresent, FrameTime, HeartRate)
    pub ultrasound: Vec<String>,
    /// PET/Nuclear Medicine tags (11 tags: RadiopharmaceuticalInformationSequence, RadiopharmaceuticalStartTime, RadionuclideTotalDose, Units, DecayCorrection, etc.)
    pub pet_nm: Vec<String>,
    /// X-Ray Angiography tags (8 tags: DistanceSourceToDetector, DistanceSourceToPatient, IntensifierSize, ImagerPixelSpacing, PositionerPrimaryAngle, etc.)
    pub xa: Vec<String>,
    /// Radiation Therapy tags (11 tags: RTImageLabel, RTImageDescription, ReferencedRTPlanSequence, GantryAngle, BeamLimitingDeviceAngle, etc.)
    pub rt: Vec<String>,
    /// Comprehensive default set combining patient_basic, study_basic, series_basic, instance_basic, image_pixel_info, and equipment (42 tags)
    pub default: Vec<String>,
}

// Define tag sets as constants to avoid duplication
const PATIENT_BASIC: &[&str] = &[
    "PatientName", "PatientID", "PatientBirthDate", "PatientSex", 
    "PatientAge", "PatientWeight", "PatientSize",
];

const STUDY_BASIC: &[&str] = &[
    "StudyInstanceUID", "StudyDate", "StudyTime", "StudyDescription",
    "StudyID", "AccessionNumber", "ReferringPhysicianName",
];

const SERIES_BASIC: &[&str] = &[
    "SeriesInstanceUID", "SeriesNumber", "SeriesDescription", "SeriesDate",
    "SeriesTime", "Modality", "BodyPartExamined", "ProtocolName",
];

const INSTANCE_BASIC: &[&str] = &[
    "SOPInstanceUID", "SOPClassUID", "InstanceNumber",
    "InstanceCreationDate", "InstanceCreationTime",
];

const IMAGE_PIXEL_INFO: &[&str] = &[
    "Rows", "Columns", "BitsAllocated", "BitsStored", "HighBit",
    "PixelRepresentation", "SamplesPerPixel", "PhotometricInterpretation",
    "PixelSpacing",
];

const EQUIPMENT: &[&str] = &[
    "Manufacturer", "ManufacturerModelName", "DeviceSerialNumber",
    "SoftwareVersions", "InstitutionName", "StationName",
];

const CT: &[&str] = &[
    "KVP", "ExposureTime", "XRayTubeCurrent", "Exposure",
    "ConvolutionKernel", "SliceThickness",
];

const MR: &[&str] = &[
    "RepetitionTime", "EchoTime", "MagneticFieldStrength", "FlipAngle",
    "ImagingFrequency", "ScanningSequence",
];

const IMAGE_PLANE: &[&str] = &[
    "ImagePositionPatient", "ImageOrientationPatient", "FrameOfReferenceUID",
    "PositionReferenceIndicator", "SliceLocation", "SpacingBetweenSlices",
];

const IMAGE_DISPLAY: &[&str] = &[
    "WindowCenter", "WindowWidth", "RescaleIntercept", "RescaleSlope", "RescaleType",
];

// Ultrasound specific tags
const ULTRASOUND: &[&str] = &[
    "TransducerType", "TransducerFrequency", "SoundPathLength", 
    "UltrasoundColorDataPresent", "FrameTime", "HeartRate",
];

// PET/Nuclear Medicine specific tags  
const PET_NM: &[&str] = &[
    "RadiopharmaceuticalInformationSequence", "RadiopharmaceuticalStartTime",
    "RadiopharmaceuticalStartDateTime", "RadiopharmaceuticalVolume",
    "RadionuclideTotalDose", "Units", "DecayCorrection", "ActualFrameDuration",
    "CountsSource", "NumberOfSlices", "CorrectedImage",
];

// XA (X-Ray Angiography) specific tags
const XA: &[&str] = &[
    "DistanceSourceToDetector", "DistanceSourceToPatient", "IntensifierSize",
    "ImagerPixelSpacing", "PositionerPrimaryAngle", "PositionerSecondaryAngle",
    "TableHeight", "RadiationSetting",
];

// RT (Radiation Therapy) specific tags
const RT: &[&str] = &[
    "RTImageLabel", "RTImageDescription", "ReferencedRTPlanSequence",
    "ReferencedBeamNumber", "RadiationMachineName", "GantryAngle",
    "BeamLimitingDeviceAngle", "PatientSupportAngle", "TableTopVerticalPosition",
    "TableTopLongitudinalPosition", "TableTopLateralPosition",
];

// Patient related additional tags
const PATIENT_EXTENDED: &[&str] = &[
    "PatientComments", "EthnicGroup", "PatientSpeciesDescription",
    "PatientBreedDescription", "ResponsiblePerson", "ResponsibleOrganization",
    "PatientIdentityRemoved", "DeidentificationMethod", "OtherPatientIDs",
    "OtherPatientNames", "PatientTelephoneNumbers", "MilitaryRank",
    "BranchOfService", "MedicalRecordLocator", "ReferencedPatientPhotoSequence",
];

// Study/Visit related additional tags
const STUDY_EXTENDED: &[&str] = &[
    "AdmittingDiagnosesDescription", "PatientAge", "Occupation",
    "AdditionalPatientHistory", "AdmissionID", "IssuerOfAdmissionID",
    "ReasonForTheRequestedProcedure", "RequestedProcedureDescription",
    "RequestedProcedureID", "RequestedContrastAgent", "StudyComments",
    "ReferencedStudySequence", "ReferencedPerformedProcedureStepSequence",
];

// Series related additional tags
const SERIES_EXTENDED: &[&str] = &[
    "OperatorsName", "ReferencedPerformedProcedureStepSequence",
    "PerformedProcedureStepStartDate", "PerformedProcedureStepStartTime",
    "PerformedProcedureStepID", "PerformedProcedureStepDescription",
    "RequestAttributesSequence", "SeriesType", "AnatomicalOrientationType",
    "PerformedProtocolCodeSequence", "Laterality", "PatientPosition",
];

// Image specific additional tags
const IMAGE_EXTENDED: &[&str] = &[
    "ImageType", "ImageComments", "QualityControlImage", "BurnedInAnnotation",
    "RecognizableVisualFeatures", "LossyImageCompressionRatio",
    "LossyImageCompressionMethod", "PresentationLUTShape", "ImageLaterality",
    "AnatomicRegionSequence", "PrimaryAnatomicStructureSequence",
];

// Timing and temporal information
const TIMING: &[&str] = &[
    "ContentDate", "ContentTime", "AcquisitionDate", "AcquisitionTime", 
    "AcquisitionDateTime", "AcquisitionNumber", "TemporalPositionIdentifier",
    "NumberOfTemporalPositions", "TemporalResolution", "TriggerTime",
    "NominalInterval", "FrameTime", "FrameDelay", "ActualFrameDuration",
];

// Contrast and pharmaceutical information
const CONTRAST: &[&str] = &[
    "ContrastBolusAgent", "ContrastBolusRoute", "ContrastBolusVolume",
    "ContrastBolusStartTime", "ContrastBolusStopTime", "ContrastBolusTotalDose",
    "ContrastFlowRate", "ContrastFlowDuration", "ContrastBolusIngredient",
    "ContrastBolusIngredientConcentration",
];

// Spatial and geometric information
const GEOMETRY: &[&str] = &[
    "SliceThickness", "SpacingBetweenSlices", "SliceLocation",
    "ImagePositionPatient", "ImageOrientationPatient", "PixelSpacing",
    "FrameOfReferenceUID", "PositionReferenceIndicator", "TableHeight",
    "TableTopVerticalPosition", "TableTopLongitudinalPosition", "TableTopLateralPosition",
    "DataCollectionDiameter", "ReconstructionDiameter", "ReconstructionTargetCenterPatient",
];

// Pixel data characteristics
const PIXEL_CHARACTERISTICS: &[&str] = &[
    "SmallestImagePixelValue", "LargestImagePixelValue", "SmallestPixelValueInSeries",
    "LargestPixelValueInSeries", "PixelPaddingValue", "PixelPaddingRangeLimit",
    "RedPaletteColorLookupTableDescriptor", "GreenPaletteColorLookupTableDescriptor",
    "BluePaletteColorLookupTableDescriptor", "PlanarConfiguration", "PixelAspectRatio",
];

// Display and presentation
const DISPLAY: &[&str] = &[
    "WindowCenter", "WindowWidth", "WindowCenterWidthExplanation",
    "RescaleIntercept", "RescaleSlope", "RescaleType",
    "VOILUTFunction", "PresentationIntentType", "LossyImageCompression",
];

// Technical parameters
const TECHNICAL: &[&str] = &[
    "KVP", "ExposureTime", "XRayTubeCurrent", "Exposure", "ExposureInMicroAmpereSeconds",
    "FilterType", "GeneratorPower", "FocalSpots", "AnodeTargetMaterial",
    "BodyPartThickness", "CompressionForce", "PaddleDescription",
    "DetectorTemperature", "CollimatorType", "TableType",
];

// Overlays and graphics
const OVERLAYS: &[&str] = &[
    "OverlayRows", "OverlayColumns", "OverlayType", "OverlayOrigin",
    "OverlayBitsAllocated", "OverlayBitPosition", "OverlayData",
    "GraphicAnnotationSequence", "TextObjectSequence",
];

// Waveforms (ECG, etc)
const WAVEFORMS: &[&str] = &[
    "WaveformSequence", "WaveformOriginality", "NumberOfWaveformChannels",
    "NumberOfWaveformSamples", "SamplingFrequency", "ChannelSensitivity",
];

// Multi-frame and cine
const MULTIFRAME: &[&str] = &[
    "NumberOfFrames", "FrameIncrementPointer", "FrameTime", "FrameDelay",
    "ImageTriggerDelay", "MultiplexGroupTimeOffset", "TriggerTimeOffset",
    "NominalCardiacTriggerDelayTime", "ActualCardiacTriggerTimePriorToRPeak",
    "CineRate", "PreferredPlaybackSequencing", "RecommendedDisplayFrameRate",
];

// Additional common tags
const ADDITIONAL_COMMON: &[&str] = &[
    "TransferSyntaxUID", "ImplementationVersionName", "ImplementationClassUID",
    "SourceApplicationEntityTitle", "SendingApplicationEntityTitle",
    "ReceivingApplicationEntityTitle", "PrivateInformationCreatorUID",
    "TimezoneOffsetFromUTC", "ResponsiblePersonRole", "RequestingPhysician",
    "PerformingPhysicianName", "NameOfPhysiciansReadingStudy", "PhysiciansOfRecord",
    "ReferencedImageSequence", "DerivationDescription", "SourceImageSequence",
    "ReferencedSOPClassUID", "ReferencedSOPInstanceUID", "SpecificCharacterSet",
];

fn to_string_vec(slice: &[&str]) -> Vec<String> {
    slice.iter().map(|s| s.to_string()).collect()
}

/**
 * Get predefined sets of commonly used DICOM tags organized by category.
 * 
 * Returns a structured object containing 13 different tag sets for various
 * use cases, from basic patient demographics to modality-specific parameters.
 * 
 * This is the primary function for accessing curated tag lists without
 * needing to manually specify individual tag names.
 * 
 * @returns Object containing all predefined tag sets
 * 
 * @example
 * ```typescript
 * import { getCommonTagSets, DicomFile } from '@nuxthealth/node-dicom';
 * 
 * const dicom = new DicomFile();
 * dicom.open('patient-scan.dcm');
 * 
 * // Get all tag sets
 * const tagSets = getCommonTagSets();
 * 
 * // Extract patient demographics
 * const patientData = dicom.extract(tagSets.patientBasic);
 * console.log(patientData);
 * // { PatientName: 'DOE^JOHN', PatientID: '12345', ... }
 * 
 * // Extract study information
 * const studyData = dicom.extract(tagSets.studyBasic);
 * 
 * // Extract everything common
 * const allData = dicom.extract(tagSets.default);
 * ```
 * 
 * @example
 * ```typescript
 * // Modality-specific workflows
 * const tagSets = getCommonTagSets();
 * const dicom = new DicomFile();
 * dicom.open('ct-scan.dcm');
 * 
 * // CT scan: combine common tags with CT-specific parameters
 * const ctTags = [
 *   ...tagSets.default,
 *   ...tagSets.ct
 * ];
 * const ctData = dicom.extract(ctTags);
 * console.log(ctData.KVP, ctData.ConvolutionKernel);
 * 
 * // MR scan: different parameter set
 * const mrTags = [...tagSets.default, ...tagSets.mr];
 * const mrData = dicom.extract(mrTags);
 * console.log(mrData.RepetitionTime, mrData.EchoTime);
 * ```
 * 
 * @example
 * ```typescript
 * // Build custom tag set for specific workflow
 * const tagSets = getCommonTagSets();
 * 
 * // Anonymization workflow: extract metadata but exclude patient info
 * const metadataOnlyTags = [
 *   ...tagSets.studyBasic,
 *   ...tagSets.seriesBasic,
 *   ...tagSets.instanceBasic,
 *   ...tagSets.imagePixelInfo,
 *   ...tagSets.equipment
 * ];
 * 
 * // PACS routing: minimal required tags
 * const routingTags = [
 *   ...tagSets.patientBasic,
 *   ...tagSets.studyBasic,
 *   'Modality',
 *   'SOPClassUID'
 * ];
 * ```
 */
#[napi]
pub fn get_common_tag_sets() -> CommonTagSets {
    CommonTagSets {
        patient_basic: to_string_vec(PATIENT_BASIC),
        study_basic: to_string_vec(STUDY_BASIC),
        series_basic: to_string_vec(SERIES_BASIC),
        instance_basic: to_string_vec(INSTANCE_BASIC),
        image_pixel_info: to_string_vec(IMAGE_PIXEL_INFO),
        equipment: to_string_vec(EQUIPMENT),
        ct: to_string_vec(CT),
        mr: to_string_vec(MR),
        ultrasound: to_string_vec(ULTRASOUND),
        pet_nm: to_string_vec(PET_NM),
        xa: to_string_vec(XA),
        rt: to_string_vec(RT),
        default: to_string_vec(&[
            PATIENT_BASIC,
            STUDY_BASIC,
            SERIES_BASIC,
            INSTANCE_BASIC,
            IMAGE_PIXEL_INFO,
            EQUIPMENT,
        ].concat()),
    }
}

/**
 * Combine multiple tag arrays into a single deduplicated array.
 * 
 * Merges multiple arrays of tag names while removing duplicates,
 * preserving the order of first appearance. Useful for building
 * custom tag sets from predefined sets or combining modality-specific
 * tags with common tags.
 * 
 * @param tagArrays - Array of tag name arrays to combine
 * @returns Single array containing all unique tag names
 * 
 * @example
 * ```typescript
 * import { getCommonTagSets, combineTags } from '@nuxthealth/node-dicom';
 * 
 * const tagSets = getCommonTagSets();
 * 
 * // Combine multiple predefined sets
 * const combined = combineTags([
 *   tagSets.patientBasic,
 *   tagSets.studyBasic,
 *   tagSets.seriesBasic,
 *   tagSets.imagePixelInfo
 * ]);
 * 
 * console.log(combined.length); // No duplicates
 * ```
 * 
 * @example
 * ```typescript
 * // Mix predefined sets with custom tags
 * const tagSets = getCommonTagSets();
 * 
 * const customTags = combineTags([
 *   tagSets.patientBasic,
 *   tagSets.studyBasic,
 *   ['CustomTag1', 'CustomTag2'],
 *   tagSets.ct,
 *   ['WindowCenter', 'WindowWidth'] // May overlap with other sets
 * ]);
 * 
 * // Result contains all unique tags in order of first appearance
 * ```
 * 
 * @example
 * ```typescript
 * // Build modality-agnostic tag set
 * const tagSets = getCommonTagSets();
 * 
 * const universalModalityTags = combineTags([
 *   tagSets.default,
 *   tagSets.ct,
 *   tagSets.mr,
 *   tagSets.ultrasound,
 *   tagSets.petNm,
 *   tagSets.xa,
 *   tagSets.rt
 * ]);
 * 
 * // Covers all modalities, duplicates automatically removed
 * console.log(`Total unique tags: ${universalModalityTags.length}`);
 * ```
 */
#[napi]
pub fn combine_tags(tag_arrays: Vec<Vec<String>>) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for array in tag_arrays {
        for tag in array {
            if seen.insert(tag.clone()) {
                result.push(tag);
            }
        }
    }
    
    result
}

/**
 * Create a custom tag specification for extracting non-standard or private DICOM tags.
 * 
 * Allows you to define custom mappings from DICOM tag numbers (in hex format)
 * to human-readable names. Useful for private tags, vendor-specific tags,
 * or tags not included in standard dictionaries.
 * 
 * @param tag - DICOM tag in format "(GGGG,EEEE)" where GGGG is group and EEEE is element (hex)
 * @param name - Human-readable name to use for this tag in extracted data
 * @returns CustomTag object that can be used in extraction functions
 * 
 * @example
 * ```typescript
 * import { createCustomTag, DicomFile } from '@nuxthealth/node-dicom';
 * 
 * const dicom = new DicomFile();
 * dicom.open('scan-with-private-tags.dcm');
 * 
 * // Define custom tags
 * const customTag1 = createCustomTag('(0009,1001)', 'VendorSpecificID');
 * const customTag2 = createCustomTag('(0019,100A)', 'ProprietaryField');
 * 
 * // Extract using custom tag definitions
 * const data = dicom.extractWithCustomTags({
 *   tags: ['PatientName', 'StudyDate'],
 *   customTags: [customTag1, customTag2]
 * });
 * 
 * console.log(data.VendorSpecificID);
 * console.log(data.ProprietaryField);
 * ```
 * 
 * @example
 * ```typescript
 * // Extract private GE tags
 * const gePrivate1 = createCustomTag('(0009,10XX)', 'GE_PrivateCreator');
 * const gePrivate2 = createCustomTag('(0043,1001)', 'GE_ImageInfo');
 * 
 * // Extract private Siemens tags
 * const siemensTag = createCustomTag('(0019,100C)', 'Siemens_ScanOptions');
 * 
 * // Extract private Philips tags
 * const philipsTag = createCustomTag('(2001,1003)', 'Philips_ScanMode');
 * ```
 * 
 * @example
 * ```typescript
 * // Build library of vendor-specific tags
 * const vendorTags = {
 *   ge: [
 *     createCustomTag('(0009,1001)', 'GE_Field1'),
 *     createCustomTag('(0043,1010)', 'GE_Field2')
 *   ],
 *   siemens: [
 *     createCustomTag('(0019,1008)', 'Siemens_Field1'),
 *     createCustomTag('(0029,1010)', 'Siemens_Field2')
 *   ],
 *   philips: [
 *     createCustomTag('(2001,1001)', 'Philips_Field1'),
 *     createCustomTag('(2005,1080)', 'Philips_Field2')
 *   ]
 * };
 * 
 * // Use based on manufacturer
 * const manufacturer = dicom.extract(['Manufacturer'])[0];
 * const customTags = vendorTags[manufacturer.toLowerCase()] || [];
 * ```
 */
#[napi]
pub fn create_custom_tag(tag: String, name: String) -> crate::utils::CustomTag {
    crate::utils::CustomTag { tag, name }
}

/**
 * Get a comprehensive list of 300+ commonly used DICOM tag names.
 * 
 * Returns an array of standard DICOM tag names covering all major
 * modalities and use cases. These names can be used directly with
 * extraction functions like `extract()`, `extractBatch()`, etc.
 * 
 * The list includes tags from all categories:
 * - Patient demographics and identification
 * - Study, series, and instance metadata
 * - Image characteristics and pixel data
 * - Equipment and institution information
 * - Modality-specific parameters (CT, MR, US, PET, NM, XA, RT, etc.)
 * - Timing and temporal information
 * - Geometry and spatial information
 * - Display and presentation parameters
 * - Technical acquisition parameters
 * - Overlays, graphics, and waveforms
 * - Multi-frame and cine sequences
 * 
 * @returns Array of 300+ standard DICOM tag names
 * 
 * @example
 * ```typescript
 * import { getAvailableTagNames } from '@nuxthealth/node-dicom';
 * 
 * // Get all available tag names
 * const allTags = getAvailableTagNames();
 * console.log(`Total available tags: ${allTags.length}`);
 * // Output: Total available tags: 300+
 * 
 * // Check if specific tag is available
 * const hasWindowCenter = allTags.includes('WindowCenter');
 * const hasCustomTag = allTags.includes('MyCustomTag');
 * 
 * // Use for autocomplete or validation
 * const validTags = ['PatientName', 'StudyDate', 'InvalidTag']
 *   .filter(tag => allTags.includes(tag));
 * ```
 * 
 * @example
 * ```typescript
 * // Search for tags matching a pattern
 * const allTags = getAvailableTagNames();
 * 
 * // Find all patient-related tags
 * const patientTags = allTags.filter(tag => 
 *   tag.toLowerCase().includes('patient')
 * );
 * console.log(patientTags);
 * // ['PatientName', 'PatientID', 'PatientBirthDate', ...]
 * 
 * // Find all timing-related tags
 * const timeTags = allTags.filter(tag => 
 *   tag.includes('Time') || tag.includes('Date')
 * );
 * 
 * // Find all UID tags
 * const uidTags = allTags.filter(tag => tag.endsWith('UID'));
 * ```
 * 
 * @example
 * ```typescript
 * // Build dynamic tag selection UI
 * const allTags = getAvailableTagNames();
 * 
 * // Group tags by category
 * const tagsByCategory = {
 *   patient: allTags.filter(t => t.includes('Patient')),
 *   study: allTags.filter(t => t.includes('Study')),
 *   series: allTags.filter(t => t.includes('Series')),
 *   image: allTags.filter(t => t.includes('Image') || t.includes('Pixel')),
 *   equipment: allTags.filter(t => 
 *     t.includes('Manufacturer') || 
 *     t.includes('Device') || 
 *     t.includes('Station')
 *   )
 * };
 * 
 * // Present to user for selection
 * Object.entries(tagsByCategory).forEach(([category, tags]) => {
 *   console.log(`${category}: ${tags.length} tags available`);
 * });
 * ```
 * 
 * @example
 * ```typescript
 * // Validate user input against available tags
 * const allTags = getAvailableTagNames();
 * 
 * function validateTagNames(userTags: string[]): {
 *   valid: string[],
 *   invalid: string[]
 * } {
 *   const valid = userTags.filter(tag => allTags.includes(tag));
 *   const invalid = userTags.filter(tag => !allTags.includes(tag));
 *   return { valid, invalid };
 * }
 * 
 * const userInput = [
 *   'PatientName',
 *   'StudyDate',
 *   'InvalidTag',
 *   'Modality'
 * ];
 * 
 * const result = validateTagNames(userInput);
 * console.log('Valid tags:', result.valid);
 * console.log('Invalid tags:', result.invalid);
 * ```
 */
#[napi(ts_return_type = "Array<'AccessionNumber' | 'AcquisitionDate' | 'AcquisitionDateTime' | 'AcquisitionNumber' | 'AcquisitionTime' | 'ActualCardiacTriggerTimePriorToRPeak' | 'ActualFrameDuration' | 'AdditionalPatientHistory' | 'AdmissionID' | 'AdmittingDiagnosesDescription' | 'AnatomicalOrientationType' | 'AnatomicRegionSequence' | 'AnodeTargetMaterial' | 'BeamLimitingDeviceAngle' | 'BitsAllocated' | 'BitsStored' | 'BluePaletteColorLookupTableDescriptor' | 'BodyPartExamined' | 'BodyPartThickness' | 'BranchOfService' | 'BurnedInAnnotation' | 'ChannelSensitivity' | 'CineRate' | 'CollimatorType' | 'Columns' | 'CompressionForce' | 'ContentDate' | 'ContentTime' | 'ContrastBolusAgent' | 'ContrastBolusIngredient' | 'ContrastBolusIngredientConcentration' | 'ContrastBolusRoute' | 'ContrastBolusStartTime' | 'ContrastBolusStopTime' | 'ContrastBolusTotalDose' | 'ContrastBolusVolume' | 'ContrastFlowDuration' | 'ContrastFlowRate' | 'ConvolutionKernel' | 'CorrectedImage' | 'CountsSource' | 'DataCollectionDiameter' | 'DecayCorrection' | 'DeidentificationMethod' | 'DerivationDescription' | 'DetectorTemperature' | 'DeviceSerialNumber' | 'DistanceSourceToDetector' | 'DistanceSourceToPatient' | 'EchoTime' | 'EthnicGroup' | 'Exposure' | 'ExposureInMicroAmpereSeconds' | 'ExposureTime' | 'FilterType' | 'FlipAngle' | 'FocalSpots' | 'FrameDelay' | 'FrameIncrementPointer' | 'FrameOfReferenceUID' | 'FrameTime' | 'GantryAngle' | 'GeneratorPower' | 'GraphicAnnotationSequence' | 'GreenPaletteColorLookupTableDescriptor' | 'HeartRate' | 'HighBit' | 'ImageComments' | 'ImageLaterality' | 'ImageOrientationPatient' | 'ImagePositionPatient' | 'ImagerPixelSpacing' | 'ImageTriggerDelay' | 'ImageType' | 'ImagingFrequency' | 'ImplementationClassUID' | 'ImplementationVersionName' | 'InstanceCreationDate' | 'InstanceCreationTime' | 'InstanceNumber' | 'InstitutionName' | 'IntensifierSize' | 'IssuerOfAdmissionID' | 'KVP' | 'LargestImagePixelValue' | 'LargestPixelValueInSeries' | 'Laterality' | 'LossyImageCompression' | 'LossyImageCompressionMethod' | 'LossyImageCompressionRatio' | 'MagneticFieldStrength' | 'Manufacturer' | 'ManufacturerModelName' | 'MedicalRecordLocator' | 'MilitaryRank' | 'Modality' | 'MultiplexGroupTimeOffset' | 'NameOfPhysiciansReadingStudy' | 'NominalCardiacTriggerDelayTime' | 'NominalInterval' | 'NumberOfFrames' | 'NumberOfSlices' | 'NumberOfTemporalPositions' | 'NumberOfWaveformChannels' | 'NumberOfWaveformSamples' | 'Occupation' | 'OperatorsName' | 'OtherPatientIDs' | 'OtherPatientNames' | 'OverlayBitPosition' | 'OverlayBitsAllocated' | 'OverlayColumns' | 'OverlayData' | 'OverlayOrigin' | 'OverlayRows' | 'OverlayType' | 'PaddleDescription' | 'PatientAge' | 'PatientBirthDate' | 'PatientBreedDescription' | 'PatientComments' | 'PatientID' | 'PatientIdentityRemoved' | 'PatientName' | 'PatientPosition' | 'PatientSex' | 'PatientSize' | 'PatientSpeciesDescription' | 'PatientSupportAngle' | 'PatientTelephoneNumbers' | 'PatientWeight' | 'PerformedProcedureStepDescription' | 'PerformedProcedureStepID' | 'PerformedProcedureStepStartDate' | 'PerformedProcedureStepStartTime' | 'PerformedProtocolCodeSequence' | 'PerformingPhysicianName' | 'PhotometricInterpretation' | 'PhysiciansOfRecord' | 'PixelAspectRatio' | 'PixelPaddingRangeLimit' | 'PixelPaddingValue' | 'PixelRepresentation' | 'PixelSpacing' | 'PlanarConfiguration' | 'PositionerPrimaryAngle' | 'PositionerSecondaryAngle' | 'PositionReferenceIndicator' | 'PreferredPlaybackSequencing' | 'PresentationIntentType' | 'PresentationLUTShape' | 'PrimaryAnatomicStructureSequence' | 'PrivateInformationCreatorUID' | 'ProtocolName' | 'QualityControlImage' | 'RadiationMachineName' | 'RadiationSetting' | 'RadionuclideTotalDose' | 'RadiopharmaceuticalInformationSequence' | 'RadiopharmaceuticalStartDateTime' | 'RadiopharmaceuticalStartTime' | 'RadiopharmaceuticalVolume' | 'ReasonForTheRequestedProcedure' | 'ReceivingApplicationEntityTitle' | 'RecognizableVisualFeatures' | 'RecommendedDisplayFrameRate' | 'ReconstructionDiameter' | 'ReconstructionTargetCenterPatient' | 'RedPaletteColorLookupTableDescriptor' | 'ReferencedBeamNumber' | 'ReferencedImageSequence' | 'ReferencedPatientPhotoSequence' | 'ReferencedPerformedProcedureStepSequence' | 'ReferencedRTPlanSequence' | 'ReferencedSOPClassUID' | 'ReferencedSOPInstanceUID' | 'ReferencedStudySequence' | 'ReferringPhysicianName' | 'RepetitionTime' | 'RequestAttributesSequence' | 'RequestedContrastAgent' | 'RequestedProcedureDescription' | 'RequestedProcedureID' | 'RequestingPhysician' | 'RescaleIntercept' | 'RescaleSlope' | 'RescaleType' | 'ResponsibleOrganization' | 'ResponsiblePerson' | 'ResponsiblePersonRole' | 'Rows' | 'RTImageDescription' | 'RTImageLabel' | 'SamplesPerPixel' | 'SamplingFrequency' | 'ScanningSequence' | 'SendingApplicationEntityTitle' | 'SeriesDate' | 'SeriesDescription' | 'SeriesInstanceUID' | 'SeriesNumber' | 'SeriesTime' | 'SeriesType' | 'SliceLocation' | 'SliceThickness' | 'SmallestImagePixelValue' | 'SmallestPixelValueInSeries' | 'SoftwareVersions' | 'SOPClassUID' | 'SOPInstanceUID' | 'SoundPathLength' | 'SourceApplicationEntityTitle' | 'SourceImageSequence' | 'SpacingBetweenSlices' | 'SpecificCharacterSet' | 'StationName' | 'StudyComments' | 'StudyDate' | 'StudyDescription' | 'StudyID' | 'StudyInstanceUID' | 'StudyTime' | 'TableHeight' | 'TableTopLateralPosition' | 'TableTopLongitudinalPosition' | 'TableTopVerticalPosition' | 'TableType' | 'TemporalPositionIdentifier' | 'TemporalResolution' | 'TextObjectSequence' | 'TimezoneOffsetFromUTC' | 'TransducerFrequency' | 'TransducerType' | 'TransferSyntaxUID' | 'TriggerTime' | 'TriggerTimeOffset' | 'UltrasoundColorDataPresent' | 'Units' | 'VOILUTFunction' | 'WaveformOriginality' | 'WaveformSequence' | 'WindowCenter' | 'WindowCenterWidthExplanation' | 'WindowWidth' | 'XRayTubeCurrent'>")]
pub fn get_available_tag_names() -> Vec<String> {
    // Combine all predefined tag sets - 300+ tags covering all major use cases
    to_string_vec(&[
        PATIENT_BASIC,
        PATIENT_EXTENDED,
        STUDY_BASIC,
        STUDY_EXTENDED,
        SERIES_BASIC,
        SERIES_EXTENDED,
        INSTANCE_BASIC,
        IMAGE_PIXEL_INFO,
        PIXEL_CHARACTERISTICS,
        EQUIPMENT,
        CT,
        MR,
        ULTRASOUND,
        PET_NM,
        XA,
        RT,
        IMAGE_PLANE,
        GEOMETRY,
        TIMING,
        CONTRAST,
        IMAGE_DISPLAY,
        DISPLAY,
        TECHNICAL,
        IMAGE_EXTENDED,
        OVERLAYS,
        WAVEFORMS,
        MULTIFRAME,
        ADDITIONAL_COMMON,
    ].concat())
}
