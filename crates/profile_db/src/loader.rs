use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub cpu_model: Option<String>,
    pub max_memory_mb: Option<u64>,
    pub compute_units: Option<u32>,
    pub llama_config: LlamaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaConfig {
    pub n_threads: Option<u32>,
    pub n_gpu_layers: Option<u32>,
    pub flash_attn: Option<bool>,
}

pub fn load_profile(path: &Path) -> Result<Profile, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let profile: Profile = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(profile)
}

pub fn load_profiles_from_dir(dir: &Path) -> Result<Vec<Profile>, String> {
    let mut profiles = Vec::new();

    if !dir.exists() {
        return Ok(profiles);
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            match load_profile(&path) {
                Ok(profile) => profiles.push(profile),
                Err(e) => eprintln!("Failed to load profile {}: {}", path.display(), e),
            }
        }
    }

    Ok(profiles)
}
