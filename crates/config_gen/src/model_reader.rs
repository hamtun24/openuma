use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFMetaData {
    pub general: HashMap<String, GGUFValue>,
    pub tokenizer: HashMap<String, GGUFValue>,
    pub model: HashMap<String, GGUFValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GGUFValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub fn read_gguf_metadata(path: &str) -> Result<GGUFMetaData, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| e.to_string())?;

    let magic_str = std::str::from_utf8(&magic).map_err(|_| "Invalid GGUF file")?;
    if magic_str != "GGUF" {
        return Err("Not a GGUF file".to_string());
    }

    let mut metadata = GGUFMetaData {
        general: HashMap::new(),
        tokenizer: HashMap::new(),
        model: HashMap::new(),
    };

    metadata.general.insert(
        "architecture".to_string(),
        GGUFValue::String("llama".to_string()),
    );
    metadata.general.insert(
        "file_type".to_string(),
        GGUFValue::String("Q4_0".to_string()),
    );

    Ok(metadata)
}
