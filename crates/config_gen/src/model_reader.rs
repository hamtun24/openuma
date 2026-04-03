use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

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

fn read_string<R: Read>(reader: &mut R) -> Result<String, String> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let len = u32::from_le_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    String::from_utf8(buf).map_err(|e| e.to_string())
}

fn read_value<R: Read>(reader: &mut R, type_id: u32) -> Result<GGUFValue, String> {
    match type_id {
        0 => {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Bool(buf[0] != 0))
        }
        1 => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Int(i64::from_le_bytes(buf)))
        }
        2 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Float(f32::from_le_bytes(buf) as f64))
        }
        3 => read_string(reader).map(GGUFValue::String),
        4 => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Int(i64::from_le_bytes(buf)))
        }
        5 => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Bool(i64::from_le_bytes(buf) != 0))
        }
        6 => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Float(f64::from_le_bytes(buf)))
        }
        7 => read_string(reader).map(GGUFValue::String),
        8 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
            Ok(GGUFValue::Int(u32::from_le_bytes(buf) as i64))
        }
        _ => Err(format!("Unknown type id: {}", type_id)),
    }
}

pub fn read_gguf_metadata(path: &str) -> Result<GGUFMetaData, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic).map_err(|e| e.to_string())?;

    let magic_str = std::str::from_utf8(&magic).map_err(|_| "Invalid GGUF file")?;
    if magic_str != "GGUF" {
        return Err("Not a GGUF file".to_string());
    }

    let mut version_buf = [0u8; 4];
    reader
        .read_exact(&mut version_buf)
        .map_err(|e| e.to_string())?;
    let version = u32::from_le_bytes(version_buf);

    let mut tensor_count_buf = [0u8; 8];
    reader
        .read_exact(&mut tensor_count_buf)
        .map_err(|e| e.to_string())?;
    let _tensor_count = u64::from_le_bytes(tensor_count_buf);

    let mut metadata_count_buf = [0u8; 8];
    reader
        .read_exact(&mut metadata_count_buf)
        .map_err(|e| e.to_string())?;
    let metadata_count = u64::from_le_bytes(metadata_count_buf);

    let mut metadata = GGUFMetaData {
        general: HashMap::new(),
        tokenizer: HashMap::new(),
        model: HashMap::new(),
    };

    for _ in 0..metadata_count {
        let key = read_string(&mut reader)?;

        let mut type_buf = [0u8; 4];
        reader
            .read_exact(&mut type_buf)
            .map_err(|e| e.to_string())?;
        let type_id = u32::from_le_bytes(type_buf);

        let value = read_value(&mut reader, type_id)?;

        let (section, clean_key) = if key.starts_with("general.") {
            (
                &mut metadata.general,
                key.trim_start_matches("general.").to_string(),
            )
        } else if key.starts_with("tokenizer.") {
            (
                &mut metadata.tokenizer,
                key.trim_start_matches("tokenizer.").to_string(),
            )
        } else if key.starts_with("model.") || key.starts_with("llama.") {
            (&mut metadata.model, key)
        } else {
            continue;
        };

        section.insert(clean_key.to_string(), value);
    }

    if version >= 3 {
        if let Ok(pos) = reader.seek(SeekFrom::End(0)) {
            let _ = reader.seek(SeekFrom::Start(pos.saturating_sub(8)));
        }
    }

    if metadata.model.is_empty() {
        metadata.model.insert(
            "architecture".to_string(),
            GGUFValue::String("unknown".to_string()),
        );
    }
    if metadata.tokenizer.is_empty() {
        metadata.tokenizer.insert(
            "model".to_string(),
            GGUFValue::String("unknown".to_string()),
        );
    }

    Ok(metadata)
}

pub fn get_model_info(path: &str) -> Result<ModelInfo, String> {
    let metadata = read_gguf_metadata(path)?;

    let architecture = metadata
        .general
        .get("architecture")
        .or_else(|| metadata.model.get("architecture"))
        .and_then(|v| {
            if let GGUFValue::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let num_layers = metadata
        .model
        .get("layer_count")
        .or_else(|| metadata.model.get("n_layers"))
        .and_then(|v| {
            if let GGUFValue::Int(n) = v {
                Some(*n as u32)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let hidden_size = metadata
        .model
        .get("hidden_size")
        .or_else(|| metadata.model.get("embedding_length"))
        .and_then(|v| {
            if let GGUFValue::Int(n) = v {
                Some(*n as u32)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let vocab_size = metadata
        .tokenizer
        .get("vocab_size")
        .or_else(|| metadata.model.get("vocab_size"))
        .and_then(|v| {
            if let GGUFValue::Int(n) = v {
                Some(*n as u32)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let file_type = metadata
        .general
        .get("file_type")
        .and_then(|v| {
            if let GGUFValue::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ModelInfo {
        architecture,
        num_layers,
        hidden_size,
        vocab_size,
        file_type,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub architecture: String,
    pub num_layers: u32,
    pub hidden_size: u32,
    pub vocab_size: u32,
    pub file_type: String,
}

pub fn estimate_model_size_mb(path: &str) -> u64 {
    if let Ok(metadata) = read_gguf_metadata(path) {
        let layers = metadata
            .model
            .get("layer_count")
            .or_else(|| metadata.model.get("n_layers"))
            .and_then(|v| {
                if let GGUFValue::Int(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(32) as u64;

        let hidden = metadata
            .model
            .get("hidden_size")
            .or_else(|| metadata.model.get("embedding_length"))
            .and_then(|v| {
                if let GGUFValue::Int(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(4096) as u64;

        let vocab = metadata
            .tokenizer
            .get("vocab_size")
            .and_then(|v| {
                if let GGUFValue::Int(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(32000) as u64;

        let embedding_mb = (hidden * vocab * 2) / (1024 * 1024);
        let layers_mb = (layers * hidden * hidden * 4) / (1024 * 1024);
        let attention_mb = (layers * hidden * hidden * 4) / (1024 * 1024);

        (embedding_mb + layers_mb + attention_mb) / 6
    } else {
        4000
    }
}
