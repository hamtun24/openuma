use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu: CpuProfile,
    pub igpu: Option<IgpuProfile>,
    pub ram: RamProfile,
    pub dgpu: Option<String>,
    pub platform: PlatformProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuProfile {
    pub model: String,
    pub vendor: String,
    pub cores: u32,
    pub threads: u32,
    pub frequency_mhz: u32,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgpuProfile {
    pub name: String,
    pub vendor: String,
    pub driver: String,
    pub memory_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamProfile {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_free_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformProfile {
    pub os: String,
    pub os_version: String,
    pub kernel: String,
    pub compute_backend: String,
}
