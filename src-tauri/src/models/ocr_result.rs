use serde::{Deserialize, Serialize};

/// OCR recognition result for level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelResult {
    pub level: u32,
    pub raw_text: String,
}

/// OCR recognition result for EXP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpResult {
    pub absolute: u64,
    pub percentage: f64,
    pub raw_text: String,
}

/// OCR recognition result for map
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapResult {
    pub map_name: String,
    pub raw_text: String,
}
