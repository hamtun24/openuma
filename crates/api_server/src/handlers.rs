use axum::{extract::Json, response::Json as ResponseJson};
use benchmark::{BenchmarkConfig, BenchmarkRunner, LlamaCliRunner};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn health() -> ResponseJson<HealthResponse> {
    ResponseJson(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn probe() -> ResponseJson<serde_json::Value> {
    let profile = hw_probe::probe_all().unwrap_or_else(|_e| hw_probe::HardwareProfile {
        cpu: hw_probe::CpuProfile {
            model: "Unknown".to_string(),
            vendor: "Unknown".to_string(),
            cores: 0,
            threads: 0,
            frequency_mhz: 0,
            flags: vec![],
        },
        igpu: None,
        ram: hw_probe::RamProfile {
            total_bytes: 0,
            available_bytes: 0,
            swap_total_bytes: 0,
            swap_free_bytes: 0,
        },
        dgpu: None,
        platform: hw_probe::PlatformProfile {
            os: "Unknown".to_string(),
            os_version: "Unknown".to_string(),
            kernel: "Unknown".to_string(),
            compute_backend: "Unknown".to_string(),
        },
    });

    ResponseJson(serde_json::to_value(profile).unwrap())
}

#[derive(Debug, Deserialize)]
pub struct BenchmarkRequest {
    pub model_path: String,
    pub n_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResponse {
    pub success: bool,
    pub tokens_per_second: Option<f64>,
    pub error: Option<String>,
}

pub async fn run_benchmark(Json(req): Json<BenchmarkRequest>) -> ResponseJson<BenchmarkResponse> {
    let config = BenchmarkConfig {
        model_path: req.model_path,
        n_threads: Some(4),
        n_gpu_layers: Some(0),
        batch_size: Some(512),
        ctx_size: Some(2048),
        prompt: Some("The quick brown fox".to_string()),
        generation_tokens: req.n_tokens.or(Some(100)),
    };

    let runner = LlamaCliRunner;
    match runner.run(&config) {
        Ok(result) => ResponseJson(BenchmarkResponse {
            success: true,
            tokens_per_second: Some(result.tokens_per_second),
            error: None,
        }),
        Err(e) => ResponseJson(BenchmarkResponse {
            success: false,
            tokens_per_second: None,
            error: Some(e.to_string()),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigureRequest {
    pub model_path: Option<String>,
    pub engine: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigureResponse {
    pub success: bool,
    pub flags: Vec<String>,
    pub uma_pool: Option<UmaPool>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UmaPool {
    pub igpu_mb: u64,
    pub cpu_mb: u64,
}

pub async fn configure(Json(_req): Json<ConfigureRequest>) -> ResponseJson<ConfigureResponse> {
    let profile = match hw_probe::probe_all() {
        Ok(p) => p,
        Err(e) => return ResponseJson(ConfigureResponse {
            success: false,
            flags: vec![],
            uma_pool: None,
            error: Some(e.to_string()),
        }),
    };

    let total_mb = profile.ram.total_bytes / (1024 * 1024);
    let device_mb = profile.igpu.as_ref().and_then(|igpu| igpu.memory_mb).unwrap_or(0);
    
    let partition = mem_mgr::compute_partition(total_mb, device_mb, None);
    let has_vulkan = profile.platform.compute_backend.contains("vulkan");
    let cpu_cores = profile.cpu.threads;
    
    let model_size_mb = 4000;
    let llama_config = config_gen::LlamaCppConfig::generate(
        total_mb,
        device_mb,
        model_size_mb,
        cpu_cores,
        has_vulkan,
    );

    ResponseJson(ConfigureResponse {
        success: true,
        flags: llama_config.to_flags(),
        uma_pool: Some(UmaPool {
            igpu_mb: partition.device_memory_mb,
            cpu_mb: partition.host_memory_mb,
        }),
        error: None,
    })
}

#[derive(Debug, Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub cpu_model: String,
    pub igpu: Option<String>,
    pub ram_mb: u64,
}

#[derive(Debug, Serialize)]
pub struct ListProfilesResponse {
    pub success: bool,
    pub profiles: Vec<ProfileInfo>,
    pub error: Option<String>,
}

pub async fn list_profiles() -> ResponseJson<ListProfilesResponse> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openuma");
    
    std::fs::create_dir_all(&config_dir).ok();
    
    let db_path = config_dir.join("profiles.db");
    
    match profile_db::ProfileDatabase::new(&db_path) {
        Ok(db) => {
            if let Ok(profiles) = db.get_all_profiles() {
                if profiles.is_empty() {
                    let _ = db.seed_defaults();
                }
            }
            
            match db.get_all_profiles() {
                Ok(profiles) => ResponseJson(ListProfilesResponse {
                    success: true,
                    profiles: profiles.into_iter().map(|p| ProfileInfo {
                        name: p.name,
                        cpu_model: p.cpu_model,
                        igpu: p.igpu,
                        ram_mb: p.ram_mb,
                    }).collect(),
                    error: None,
                }),
                Err(e) => ResponseJson(ListProfilesResponse {
                    success: false,
                    profiles: vec![],
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => ResponseJson(ListProfilesResponse {
            success: false,
            profiles: vec![],
            error: Some(e.to_string()),
        }),
    }
}
