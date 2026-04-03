use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub model_path: String,
    pub n_threads: Option<u32>,
    pub n_gpu_layers: Option<u32>,
    pub batch_size: Option<u32>,
    pub ctx_size: Option<u32>,
    pub prompt: Option<String>,
    pub generation_tokens: Option<u32>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            n_threads: Some(4),
            n_gpu_layers: Some(0),
            batch_size: Some(512),
            ctx_size: Some(2048),
            prompt: Some("The quick brown fox jumps over the lazy dog".to_string()),
            generation_tokens: Some(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub model: String,
    pub total_tokens: u32,
    pub prompt_eval_time_ms: u64,
    pub eval_time_ms: u64,
    pub tokens_per_second: f64,
    pub peak_memory_mb: u64,
}

impl BenchmarkResult {
    pub fn from_output(output: &str) -> Option<Self> {
        let mut result = BenchmarkResult {
            model: String::new(),
            total_tokens: 0,
            prompt_eval_time_ms: 0,
            eval_time_ms: 0,
            tokens_per_second: 0.0,
            peak_memory_mb: 0,
        };

        for line in output.lines() {
            if line.contains("total tokens") {
                result.total_tokens = line.split(':').nth(1)?.trim().parse().ok()?;
            } else if line.contains("prompt eval time") {
                result.prompt_eval_time_ms = line
                    .split(':')
                    .nth(1)?
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()?;
            } else if line.contains("eval time") {
                result.eval_time_ms = line
                    .split(':')
                    .nth(1)?
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()?;
            } else if line.contains("tokens per second") || line.contains("t/s") {
                result.tokens_per_second = line
                    .split(':')
                    .nth(1)
                    .or(Some(line))
                    .unwrap_or("0")
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()?;
            }
        }

        Some(result)
    }
}
