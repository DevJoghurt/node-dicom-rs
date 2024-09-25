use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use dicom_dictionary_std::tags;
use dicom_core::header::Tag;
use dicom_object::{ open_file, DefaultDicomObject};
use snafu::prelude::*;
use napi::JsError;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(whatever, display("{}", message))]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + 'static>, Some)))]
        source: Option<Box<dyn std::error::Error + 'static>>,
    },
}

#[napi]
pub struct DicomFile{
    /// DICOM object
    dicom_file: Option<DefaultDicomObject>,
}

#[napi(object)]
pub struct DicomFileMeta {
    /// Storage SOP Class UID
    pub sop_class_uid: String,
    /// Storage SOP Instance UID
    pub sop_instance_uid: String,
}

#[napi(object)]
pub struct DicomElements {
    /// Storage SOP Class UID
    pub sop_class_uid: Option<String>,
    /// Storage SOP Instance UID
    pub sop_instance_uid: Option<String>,
    /// Instance Creation Date
    pub instance_creation_date: Option<String>,
    /// Instance Creation Time
    pub instance_creation_time: Option<String>,
    /// Study Id
    pub study_id: Option<String>,
    /// Study Date
    pub study_date: Option<String>,
    /// Study Time
    pub study_time: Option<String>,
    /// Acquisition DateTime
    pub acquisition_date_time: Option<String>,
    /// Modality
    pub modality: Option<String>,
    /// Manufacturer
    pub manufacturer: Option<String>,
    /// Manufacturer Model Name
    pub manufacturer_model_name: Option<String>,
    /// Study Description
    pub study_description: Option<String>,
    /// Series Description
    pub series_description: Option<String>,
    /// Patient Name
    pub patient_name: Option<String>,
    /// Patient ID
    pub patient_id: Option<String>,
    /// Patient Birth Date
    pub patient_birth_date: Option<String>,
    /// Patient Sex
    pub patient_sex: Option<String>,
    /// Image Comments
    pub image_comments: Option<String>,
    /// Series Number
    pub series_number: Option<String>,
    /// Instance Number
    pub instance_number: Option<String>,
}

#[napi]
impl DicomFile {

    #[napi(constructor)]
    pub fn new() -> Result<Self, JsError> {
        Ok(DicomFile {
            dicom_file: None,
        })
    }

    #[napi]
    pub fn check(&self, path: String) -> Result<DicomFileMeta, JsError> {
        let file = PathBuf::from(path);
        let obj = check_file(file.as_path());

        match obj {
            Ok(obj) => Ok(obj),
            Err(e) => Err(JsError::from(napi::Error::from_reason(e.to_string()))),
        }
    }

    #[napi]
    pub fn open(&mut self, path: String) -> Option<String> {
        let file = PathBuf::from(path);
        let dicom_file = open_file(file).unwrap();
        self.dicom_file = Some(dicom_file);
        Some("File opened".to_string())
    }

    #[napi]
    pub fn dump(&self) {
        let _ = dicom_dump::dump_file(self.dicom_file.as_ref().unwrap());
    }

    #[napi]
    pub fn get_patient_name(&self) -> Result<Option<String>, JsError> {
        Self::get_element_value(self, tags::PATIENT_NAME)
    }

    #[napi]
    pub fn get_elements(&self) -> Result<DicomElements, JsError> {

        let sop_class_uid = Self::get_element_value(self, tags::MEDIA_STORAGE_SOP_CLASS_UID);
        let sop_instance_uid = Self::get_element_value(self, tags::MEDIA_STORAGE_SOP_INSTANCE_UID);
        let instance_creation_date = Self::get_element_value(self, tags::INSTANCE_CREATION_DATE);
        let instance_creation_time = Self::get_element_value(self, tags::INSTANCE_CREATION_TIME);
        let study_date = Self::get_element_value(self, tags::STUDY_DATE);
        let study_time = Self::get_element_value(self, tags::STUDY_TIME);
        let acquisition_date_time = Self::get_element_value(self, tags::ACQUISITION_DATE_TIME);
        let modality = Self::get_element_value(self, tags::MODALITY);
        let manufacturer = Self::get_element_value(self, tags::MANUFACTURER);
        let manufacturer_model_name = Self::get_element_value(self, tags::MANUFACTURER_MODEL_NAME);
        let study_description = Self::get_element_value(self, tags::STUDY_DESCRIPTION);
        let series_description = Self::get_element_value(self, tags::SERIES_DESCRIPTION);
        let patient_name = Self::get_element_value(self, tags::PATIENT_NAME);
        let patient_id = Self::get_element_value(self, tags::PATIENT_ID);
        let patient_birth_date = Self::get_element_value(self, tags::PATIENT_BIRTH_DATE);
        let patient_sex = Self::get_element_value(self, tags::PATIENT_SEX);
        let image_comments = Self::get_element_value(self, tags::IMAGE_COMMENTS);
        let series_number = Self::get_element_value(self, tags::SERIES_NUMBER);
        let instance_number = Self::get_element_value(self, tags::INSTANCE_NUMBER);
        let study_id = Self::get_element_value(self, tags::STUDY_ID);

        Ok(DicomElements {
            sop_class_uid: sop_class_uid.unwrap_or(None),
            sop_instance_uid: sop_instance_uid.unwrap_or(None),
            instance_creation_date: instance_creation_date.unwrap_or(None),
            instance_creation_time: instance_creation_time.unwrap_or(None),
            study_date: study_date.unwrap_or(None),
            study_time: study_time.unwrap_or(None),
            acquisition_date_time: acquisition_date_time.unwrap_or(None),
            modality: modality.unwrap_or(None),
            manufacturer: manufacturer.unwrap_or(None),
            manufacturer_model_name: manufacturer_model_name.unwrap_or(None),
            study_description: study_description.unwrap_or(None),
            series_description: series_description.unwrap_or(None),
            patient_name: patient_name.unwrap_or(None),
            patient_id: patient_id.unwrap_or(None),
            patient_birth_date: patient_birth_date.unwrap_or(None),
            patient_sex: patient_sex.unwrap_or(None),
            image_comments: image_comments.unwrap_or(None),
            series_number: series_number.unwrap_or(None),
            instance_number: instance_number.unwrap_or(None),
            study_id: study_id.unwrap_or(None),
        })
    }

    #[napi]
    pub fn save_raw_pixel_data(&self, path: String) -> Result<String, JsError> {
        if self.dicom_file.is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened".to_string())));
        }
        let obj = self.dicom_file.as_ref().unwrap();
        let pixel_data = obj.element(tags::PIXEL_DATA);

        match pixel_data {
            Ok(p) => {
                let data = p.to_bytes().map_err(|e| JsError::from(napi::Error::from_reason(e.to_string())));
                match data {
                    Ok(d) => {
                        let _ = std::fs::write(path, d).map_err(|e| JsError::from(napi::Error::from_reason(e.to_string())));
                        Ok("Pixel data saved".to_string())
                    },
                    Err(e) => return Err(e),
                }
            },
            Err(e) => Err(JsError::from(napi::Error::from_reason(e.to_string()))),
        }
    }

    /**
     * Close the DICOM file to free resources
     */
    #[napi]
    pub fn close(&mut self) {
        self.dicom_file = None;
    }

    fn get_element_value(&self, tag: Tag) -> Result<Option<String>, JsError> {
        if self.dicom_file.is_none() {
            return Err(JsError::from(napi::Error::from_reason("File not opened".to_string())));
        }
        let obj = self.dicom_file.as_ref().unwrap();

        let element = obj.element(tag);

        match element {
            Ok(p) => p.to_str().map(|s| Some(s.to_string())).map_err(|e| JsError::from(napi::Error::from_reason(e.to_string()))),
            Err(e) => Err(JsError::from(napi::Error::from_reason(e.to_string()))),
        }
    }
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