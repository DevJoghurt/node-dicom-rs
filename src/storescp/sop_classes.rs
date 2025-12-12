//! SOP Class UID mappings for abstract syntax configuration

use dicom_dictionary_std::uids::*;

/// Map friendly names to SOP Class UIDs
pub fn map_sop_class_name(name: &str) -> Option<&'static str> {
    match name {
        "CTImageStorage" => Some(CT_IMAGE_STORAGE),
        "EnhancedCTImageStorage" => Some(ENHANCED_CT_IMAGE_STORAGE),
        "MRImageStorage" => Some(MR_IMAGE_STORAGE),
        "EnhancedMRImageStorage" => Some(ENHANCED_MR_IMAGE_STORAGE),
        "UltrasoundImageStorage" => Some(ULTRASOUND_MULTI_FRAME_IMAGE_STORAGE),
        "UltrasoundMultiFrameImageStorage" => Some(ULTRASOUND_MULTI_FRAME_IMAGE_STORAGE),
        "SecondaryCaptureImageStorage" => Some(SECONDARY_CAPTURE_IMAGE_STORAGE),
        "MultiFrameGrayscaleByteSecondaryCaptureImageStorage" => Some(MULTI_FRAME_GRAYSCALE_BYTE_SECONDARY_CAPTURE_IMAGE_STORAGE),
        "MultiFrameGrayscaleWordSecondaryCaptureImageStorage" => Some(MULTI_FRAME_GRAYSCALE_WORD_SECONDARY_CAPTURE_IMAGE_STORAGE),
        "MultiFrameTrueColorSecondaryCaptureImageStorage" => Some(MULTI_FRAME_TRUE_COLOR_SECONDARY_CAPTURE_IMAGE_STORAGE),
        "ComputedRadiographyImageStorage" => Some(COMPUTED_RADIOGRAPHY_IMAGE_STORAGE),
        "DigitalXRayImageStorageForPresentation" => Some(DIGITAL_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION),
        "DigitalXRayImageStorageForProcessing" => Some(DIGITAL_X_RAY_IMAGE_STORAGE_FOR_PROCESSING),
        "DigitalMammographyXRayImageStorageForPresentation" => Some(DIGITAL_MAMMOGRAPHY_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION),
        "DigitalMammographyXRayImageStorageForProcessing" => Some(DIGITAL_MAMMOGRAPHY_X_RAY_IMAGE_STORAGE_FOR_PROCESSING),
        "BreastTomosynthesisImageStorage" => Some(BREAST_TOMOSYNTHESIS_IMAGE_STORAGE),
        "BreastProjectionXRayImageStorageForPresentation" => Some(BREAST_PROJECTION_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION),
        "BreastProjectionXRayImageStorageForProcessing" => Some(BREAST_PROJECTION_X_RAY_IMAGE_STORAGE_FOR_PROCESSING),
        "PositronEmissionTomographyImageStorage" => Some(POSITRON_EMISSION_TOMOGRAPHY_IMAGE_STORAGE),
        "EnhancedPETImageStorage" => Some(ENHANCED_PET_IMAGE_STORAGE),
        "NuclearMedicineImageStorage" => Some(NUCLEAR_MEDICINE_IMAGE_STORAGE),
        "RTImageStorage" => Some(RT_IMAGE_STORAGE),
        "RTDoseStorage" => Some(RT_DOSE_STORAGE),
        "RTStructureSetStorage" => Some(RT_STRUCTURE_SET_STORAGE),
        "RTPlanStorage" => Some(RT_PLAN_STORAGE),
        "EncapsulatedPDFStorage" => Some(ENCAPSULATED_PDF_STORAGE),
        "EncapsulatedCDAStorage" => Some(ENCAPSULATED_CDA_STORAGE),
        "EncapsulatedSTLStorage" => Some(ENCAPSULATED_STL_STORAGE),
        "GrayscaleSoftcopyPresentationStateStorage" => Some(GRAYSCALE_SOFTCOPY_PRESENTATION_STATE_STORAGE),
        "BasicTextSRStorage" => Some(BASIC_TEXT_SR_STORAGE),
        "EnhancedSRStorage" => Some(ENHANCED_SR_STORAGE),
        "ComprehensiveSRStorage" => Some(COMPREHENSIVE_SR_STORAGE),
        "Verification" => Some(VERIFICATION),
        // If not a friendly name, assume it's already a UID
        _ => {
            // Check if it looks like a UID (starts with digits and contains dots)
            if name.chars().next().map_or(false, |c| c.is_ascii_digit()) && name.contains('.') {
                None // Return None to signal it should be used as-is
            } else {
                None
            }
        }
    }
}

/// Map friendly transfer syntax names to UIDs
pub fn map_transfer_syntax_name(name: &str) -> Option<&'static str> {
    match name {
        "ImplicitVRLittleEndian" => Some("1.2.840.10008.1.2"),
        "ExplicitVRLittleEndian" => Some("1.2.840.10008.1.2.1"),
        "ExplicitVRBigEndian" => Some("1.2.840.10008.1.2.2"),
        "DeflatedExplicitVRLittleEndian" => Some("1.2.840.10008.1.2.1.99"),
        "JPEGBaseline" => Some("1.2.840.10008.1.2.4.50"),
        "JPEGExtended" => Some("1.2.840.10008.1.2.4.51"),
        "JPEGLossless" => Some("1.2.840.10008.1.2.4.57"),
        "JPEGLosslessNonHierarchical" => Some("1.2.840.10008.1.2.4.70"),
        "JPEGLSLossless" => Some("1.2.840.10008.1.2.4.80"),
        "JPEGLSLossy" => Some("1.2.840.10008.1.2.4.81"),
        "JPEG2000Lossless" => Some("1.2.840.10008.1.2.4.90"),
        "JPEG2000" => Some("1.2.840.10008.1.2.4.91"),
        "RLELossless" => Some("1.2.840.10008.1.2.5"),
        "MPEG2MainProfile" => Some("1.2.840.10008.1.2.4.100"),
        "MPEG2MainProfileHighLevel" => Some("1.2.840.10008.1.2.4.101"),
        "MPEG4AVCH264HighProfile" => Some("1.2.840.10008.1.2.4.102"),
        "MPEG4AVCH264BDCompatibleHighProfile" => Some("1.2.840.10008.1.2.4.103"),
        _ => {
            // Check if it looks like a UID
            if name.chars().next().map_or(false, |c| c.is_ascii_digit()) && name.contains('.') {
                None // Return None to signal it should be used as-is
            } else {
                None
            }
        }
    }
}
