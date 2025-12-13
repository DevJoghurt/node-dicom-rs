pub mod s3;
pub mod dicom_tags;
pub mod helpers;

use std::collections::HashMap;
use dicom_object::DefaultDicomObject;
use napi_derive::napi;
use serde::{Serialize, Deserialize};

// Re-export commonly used items
pub use s3::{S3Config, build_s3_bucket, check_s3_connectivity, s3_get_object, s3_put_object, s3_list_objects};
pub use dicom_tags::*;
pub use helpers::*;

/**
 * Custom tag specification for extracting non-standard or private DICOM tags.
 */
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTag {
    /// The tag in hex format (e.g., "(0010,0010)" or "00100010") or tag name
    pub tag: String,
    /// User-defined name for this tag in the output
    pub name: String,
}

/// Extract DICOM tags as flat HashMap (simple key-value pairs)
pub fn extract_tags_flat(
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
