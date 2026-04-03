use anyhow::Result;
use benchmark::{BenchmarkConfig, BenchmarkRunner};
use config_gen::LlamaCppConfig;
use hw_probe::probe_all;
use mem_mgr::{compute_partition, UMAPartition};
use profile_db::ProfileDatabase;
use std::path::PathBuf;

pub fn probe() -> Result<()> {
    let profile = probe_all()?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

pub fn configure(engine: String, model: Option<String>, output: Option<String>) -> Result<()> {
    let profile = probe_all()?;

    let total_mb = profile.ram.total_bytes / (1024 * 1024);
    let device_mb = profile
        .igpu
        .as_ref()
        .and_then(|igpu| igpu.memory_mb)
        .unwrap_or(0);

    let partition: UMAPartition = compute_partition(total_mb, device_mb);

    let model_size_mb = if let Some(model_path) = &model {
        if let Ok(metadata) = config_gen::read_gguf_metadata(model_path) {
            metadata
                .model
                .get("embedding_length")
                .map(|v| {
                    if let config_gen::GGUFValue::Int(n) = v {
                        (n / 1024) as u64
                    } else {
                        4000
                    }
                })
                .unwrap_or(4000)
        } else {
            4000
        }
    } else {
        4000
    };

    let has_vulkan = profile.platform.compute_backend.contains("vulkan");

    let cpu_cores = profile.cpu.threads;

    match engine.as_str() {
        "llamacpp" | "llama.cpp" | "llama" => {
            let llama_config =
                LlamaCppConfig::generate(total_mb, device_mb, model_size_mb, cpu_cores, has_vulkan);

            println!("\nGenerated llama.cpp configuration:\n");

            if let Some(model_path) = &model {
                println!("  --model {}", model_path);
            }
            for flag in llama_config.to_flags() {
                println!("  {}", flag);
            }

            let tensor_ratio = if device_mb > 0 {
                device_mb as f64 / total_mb as f64
            } else {
                0.0
            };
            println!(
                "\nUMA pool: {:.1}GB iGPU / {:.1}GB CPU",
                partition.device_memory_mb as f64 / 1024.0,
                partition.host_memory_mb as f64 / 1024.0
            );

            if device_mb > 0 {
                println!("iGPU memory ratio: {:.0}%", tensor_ratio * 100.0);
            }
        }
        "ollama" => {
            println!("\nOllama Modelfile:");
            println!("  PARAMETER num_gpu {}", device_mb.min(total_mb / 4) / 1024);
            println!("  PARAMETER num_thread {}", cpu_cores);
            println!(
                "  PARAMETER ctx_size {}",
                match model_size_mb {
                    m if m < 2000 => 2048,
                    m if m < 6000 => 4096,
                    _ => 8192,
                }
            );
        }
        "ktransformers" => {
            println!("\nKTransformers configuration:");
            println!("  expert_gpu_layers: {}", (device_mb / 500).max(1));
            println!(
                "  attention_gpu_layers: {}",
                llama_cpp_layers(model_size_mb, device_mb)
            );
            println!("  cpu_threads: {}", cpu_cores);
        }
        _ => {
            println!(
                "Unknown engine: {}. Use llamacpp, ollama, or ktransformers.",
                engine
            );
        }
    }

    if let Some(out_path) = output {
        let config = serde_json::json!({
            "hardware": profile,
            "partition": partition,
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&config)?)?;
        println!("\nConfig written to {}", out_path);
    }

    Ok(())
}

fn llama_cpp_layers(model_size_mb: u64, device_mb: u64) -> u32 {
    if device_mb == 0 {
        return 0;
    }
    let usable = (device_mb as f64 * 0.7) as u64;
    match model_size_mb {
        m if m < 1000 => 32,
        m if m < 4000 => 35,
        m if m < 8000 => 40,
        _ => 48,
    }
    .min(((usable / 200) as u32).max(14))
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
        .join("openuma");

    std::fs::create_dir_all(&config_dir).ok();

    let db_path = config_dir.join("profiles.db");
    let db = ProfileDatabase::new(&db_path).map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Ok(profiles) = db.get_all_profiles() {
        if profiles.is_empty() {
            db.seed_defaults()?;
            if let Ok(new_profiles) = db.get_all_profiles() {
                println!(
                    "Initialized database with {} hardware profiles.\n",
                    new_profiles.len()
                );
            }
        }
    }

    match db.get_all_profiles() {
        Ok(profiles) => {
            if profiles.is_empty() {
                println!("No profiles found.");
            } else {
                println!("\n{} hardware profiles:\n", profiles.len());
                for profile in &profiles {
                    println!("  - {} ({})", profile.name, profile.cpu_model);
                    if let Some(ref igpu) = profile.igpu {
                        println!("    iGPU: {}", igpu);
                    }
                    println!("    RAM: {} MB", profile.ram_mb);
                }
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
