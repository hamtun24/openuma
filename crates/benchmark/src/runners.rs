use crate::types::{BenchmarkConfig, BenchmarkResult};
use anyhow::Result as AhResult;
use std::io::Read;
use std::process::{Command, Stdio};

pub trait BenchmarkRunner {
    fn run(&self, config: &BenchmarkConfig) -> AhResult<BenchmarkResult>;
}

pub struct LlamaBenchRunner;

impl BenchmarkRunner for LlamaBenchRunner {
    fn run(&self, config: &BenchmarkConfig) -> AhResult<BenchmarkResult> {
        let mut args = vec![
            "-m".to_string(),
            config.model_path.clone(),
            "-n".to_string(),
            config.generation_tokens.unwrap_or(100).to_string(),
            "-t".to_string(),
            config.n_threads.unwrap_or(4).to_string(),
        ];

        if config.n_gpu_layers.unwrap_or(0) > 0 {
            args.push("-ngl".to_string());
            args.push(config.n_gpu_layers.unwrap_or(0).to_string());
        }

        if let Some(ctx) = config.ctx_size {
            args.push("-c".to_string());
            args.push(ctx.to_string());
        }

        let mut child = Command::new("llama-bench")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to run llama-bench: {}", e))?;

        let mut output = String::new();
        if let Some(ref mut stdout) = child.stdout {
            stdout
                .read_to_string(&mut output)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        child.wait().map_err(|e| anyhow::anyhow!("{}", e))?;

        BenchmarkResult::from_output(&output)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse benchmark output"))
    }
}

pub struct LlamaCliRunner;

impl BenchmarkRunner for LlamaCliRunner {
    fn run(&self, config: &BenchmarkConfig) -> AhResult<BenchmarkResult> {
        let prompt = config.prompt.as_deref().unwrap_or("Hello, how are you?");
        let mut args = vec![
            "-m".to_string(),
            config.model_path.clone(),
            "-p".to_string(),
            prompt.to_string(),
            "-n".to_string(),
            config.generation_tokens.unwrap_or(100).to_string(),
            "--threads".to_string(),
            config.n_threads.unwrap_or(4).to_string(),
        ];

        if config.n_gpu_layers.unwrap_or(0) > 0 {
            args.push("--gpu-layers".to_string());
            args.push(config.n_gpu_layers.unwrap_or(0).to_string());
        }

        if let Some(ctx) = config.ctx_size {
            args.push("-c".to_string());
            args.push(ctx.to_string());
        }

        let output = Command::new("llama-cli")
            .args(&args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run llama-cli: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        BenchmarkResult::from_output(&stdout)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse benchmark output"))
    }
}
