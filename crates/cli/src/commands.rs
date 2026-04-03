use anyhow::Result;
use benchmark::{BenchmarkConfig, BenchmarkRunner};
use hw_probe::probe_all;
use mem_mgr::{compute_partition, UMAPartition};
use profile_db::load_profiles_from_dir;
use std::path::PathBuf;

pub fn probe() -> Result<()> {
    let profile = probe_all()?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

pub fn configure(model: Option<String>, output: Option<String>) -> Result<()> {
    let profile = probe_all()?;

    let total_mb = profile.ram.total_bytes / (1024 * 1024);
    let device_mb = profile
        .igpu
        .as_ref()
        .and_then(|igpu| igpu.memory_mb)
        .unwrap_or(0);

    let partition: UMAPartition = compute_partition(total_mb, device_mb);

    println!("Memory Partition:");
    println!("  Host: {} MB", partition.host_memory_mb);
    println!("  Device: {} MB", partition.device_memory_mb);
    println!("  Shared: {} MB", partition.shared_memory_mb);

    if let Some(model_path) = model {
        if let Ok(metadata) = config_gen::read_gguf_metadata(&model_path) {
            println!("Model Metadata:");
            println!("  {}", serde_json::to_string_pretty(&metadata)?);
        }
    }

    if let Some(out_path) = output {
        let config = serde_json::json!({
            "hardware": profile,
            "partition": partition,
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&config)?)?;
        println!("Config written to {}", out_path);
    }

    Ok(())
}

pub fn benchmark(model: String, tokens: u32, threads: Option<u32>) -> Result<()> {
    let config = BenchmarkConfig {
        model_path: model.clone(),
        n_threads: threads.or(Some(4)),
        n_gpu_layers: Some(0),
        batch_size: Some(512),
        ctx_size: Some(2048),
        prompt: Some("The quick brown fox".to_string()),
        generation_tokens: Some(tokens),
    };

    let runner = benchmark::LlamaCliRunner;
    let result = runner.run(&config)?;

    println!("Benchmark Results:");
    println!("  Model: {}", result.model);
    println!("  Total tokens: {}", result.total_tokens);
    println!("  Tokens/sec: {:.2}", result.tokens_per_second);
    println!("  Prompt eval: {} ms", result.prompt_eval_time_ms);
    println!("  Eval time: {} ms", result.eval_time_ms);

    Ok(())
}

pub fn list_profiles() -> Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openuma")
        .join("profiles");

    if !config_dir.exists() {
        println!("No profiles found");
        return Ok(());
    }

    match load_profiles_from_dir(&config_dir) {
        Ok(profiles) => {
            println!("Loaded {} profiles:", profiles.len());
            for profile in profiles {
                println!("  - {}", profile.name);
            }
        }
        Err(e) => {
            eprintln!("Error loading profiles: {}", e);
        }
    }

    Ok(())
}

pub fn serve(port: u16) -> Result<()> {
    println!("Starting API server on port {}...", port);
    Ok(())
}
