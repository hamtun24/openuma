use axum::{extract::Json, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};
use benchmark::{BenchmarkRunner, LlamaCliRunner, BenchmarkConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn health() -> ResponseJson<HealthResponse> {
    ResponseJson(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
    })
}

pub async fn probe() -> ResponseJson<serde_json::Value> {
    let profile = hw_probe::probe_all().unwrap_or_else(|_e| {
        hw_probe::HardwareProfile {
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
        }
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

pub async fn list_profiles() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "profiles": []
    }))
}
