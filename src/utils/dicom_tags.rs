use dicom_core::header::Tag;
use dicom_core::DataDictionary;
use dicom_dictionary_std::StandardDataDictionary;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Tag scope classification based on DICOM hierarchy
#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagScope {
    Patient,
    Study,
    Series,
    Instance,
    Equipment,
}

/// Determine scope from tag group/element based on DICOM standard
pub fn get_tag_scope(tag: Tag) -> TagScope {
    let group = tag.group();
    let element = tag.element();
    
    match (group, element) {
        // Patient Module (0010,xxxx)
        (0x0010, _) => TagScope::Patient,
        
        // Study Module
        (0x0008, 0x0020) | (0x0008, 0x0030) | (0x0008, 0x0050) | (0x0008, 0x0090) | 
        (0x0008, 0x1030) | (0x0008, 0x1048) | (0x0020, 0x000D) | (0x0020, 0x0010) |
        (0x0032, _) => TagScope::Study,
        
        // Series Module
        (0x0008, 0x0021) | (0x0008, 0x0031) | (0x0008, 0x0060) | (0x0008, 0x0070) |
        (0x0008, 0x0080) | (0x0008, 0x0081) | (0x0008, 0x1010) | (0x0008, 0x103E) |
        (0x0008, 0x1050) | (0x0008, 0x1070) | (0x0018, 0x0015) | (0x0018, 0x1030) |
        (0x0020, 0x000E) | (0x0020, 0x0011) | (0x0020, 0x0060) | (0x0020, 0x1002) => TagScope::Series,
        
        // Equipment Module (0018,1xxx)
        (0x0018, 0x1000..=0x1fff) => TagScope::Equipment,
        
        // Instance Level - everything else including image-specific tags
        _ => TagScope::Instance,
    }
}

/// Parse tag from string (name, hex, or (GGGG,EEEE) format)
/// Uses dicom-rs StandardDataDictionary for comprehensive tag support
pub fn parse_tag(tag_str: &str) -> Result<Tag, String> {
    // Try parsing with StandardDataDictionary first
    if let Some(tag) = StandardDataDictionary.parse_tag(tag_str) {
        return Ok(tag);
    }
    
    // Fallback: try hex format without parentheses
    if tag_str.len() == 8 {
        if let (Ok(group), Ok(element)) = (
            u16::from_str_radix(&tag_str[0..4], 16),
            u16::from_str_radix(&tag_str[4..8], 16)
        ) {
            return Ok(Tag(group, element));
        }
    }
    
    Err(format!("Invalid tag format: {}", tag_str))
}
