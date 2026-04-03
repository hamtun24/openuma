use anyhow::Result;
use benchmark::{BenchmarkConfig, BenchmarkRunner};
use config_gen::LlamaCppConfig;
use hw_probe::probe_all;
use mem_mgr::{compute_partition, detect_architecture, PartitionConfig, UMAPartition};
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

    let (model_size_mb, model_arch, num_layers, hidden_size) = if let Some(model_path) = &model {
        if let Ok(metadata) = config_gen::read_gguf_metadata(model_path) {
            let arch = detect_architecture(model_path);
            let size = metadata
                .model
                .get("embedding_length")
                .map(|v| {
                    if let config_gen::GGUFValue::Int(n) = v {
                        (n / 1024) as u64
                    } else {
                        4000
                    }
                })
                .unwrap_or(4000);
            let layers = metadata
                .model
                .get("layer_count")
                .map(|v| {
                    if let config_gen::GGUFValue::Int(n) = v {
                        *n as u32
                    } else {
                        32
                    }
                })
                .unwrap_or(32);
            let hidden = metadata
                .model
                .get("hidden_size")
                .map(|v| {
                    if let config_gen::GGUFValue::Int(n) = v {
                        *n as u32
                    } else {
                        4096
                    }
                })
                .unwrap_or(4096);
            (size, arch, layers, hidden)
        } else {
            (4000, mem_mgr::ModelArchitecture::Standard, 32, 4096)
        }
    } else {
        (4000, mem_mgr::ModelArchitecture::Standard, 32, 4096)
    };

    let partition_config = PartitionConfig {
        model_size_mb,
        model_architecture: model_arch,
        ctx_size: match model_size_mb {
            m if m < 2000 => 2048,
            m if m < 6000 => 4096,
            _ => 8192,
        },
        batch_size: 512,
        num_layers,
        hidden_size,
        num_experts: if model_arch == mem_mgr::ModelArchitecture::MoE {
            Some(8)
        } else {
            None
        },
        num_active_experts: if model_arch == mem_mgr::ModelArchitecture::MoE {
            Some(2)
        } else {
            None
        },
    };

    let partition: UMAPartition = compute_partition(total_mb, device_mb, Some(partition_config));

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
                if partition.architecture == mem_mgr::ModelArchitecture::MoE {
                    println!("Strategy: MoE-optimized (experts on CPU, attention on GPU)");
                } else {
                    println!("Strategy: Hybrid (attention layers on GPU)");
                }
                if partition.kv_cache_mb > 0 {
                    println!("KV cache estimate: {:.1} MB", partition.kv_cache_mb as f64);
                }
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

pub async fn serve(port: u16) {
    use axum::serve;
    use std::net::SocketAddr;

    println!("Starting OpenUMA API server on port {}...", port);
    println!("Endpoints:");
    println!("  GET  /health              - Health check");
    println!("  GET  /api/v1/probe       - Detect hardware");
    println!("  POST /api/v1/configure    - Generate config");
    println!("  POST /api/v1/benchmark   - Run benchmark");
    println!("  GET  /api/v1/profiles    - List hardware profiles");

    let router = api_server::create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind port");
    serve(listener, router).await.expect("Server error");
}

pub fn interactive() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║            OpenUMA Configuration Wizard                       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    println!("Step 1: Detecting hardware...");
    let profile = probe_all()?;

    println!("  ✓ Detected: {}", profile.cpu.model);
    if let Some(ref igpu) = profile.igpu {
        println!("  ✓ iGPU: {}", igpu.name);
    }
    println!(
        "  ✓ RAM: {:.1} GB",
        profile.ram.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    let _total_mb = profile.ram.total_bytes / (1024 * 1024);
    let _device_mb = profile
        .igpu
        .as_ref()
        .and_then(|igpu| igpu.memory_mb)
        .unwrap_or(0);

    println!("\nStep 2: Select inference engine");
    println!("  [1] llama.cpp  - Best for standard models");
    println!("  [2] Ollama     - Docker-based inference");
    println!("  [3] KTransformers - For MoE models (Mixtral, DeepSeek)");

    let engine = loop {
        print!("\nEnter choice (1-3) [default: 1]: ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            match input.trim() {
                "" | "1" => break "llamacpp".to_string(),
                "2" => break "ollama".to_string(),
                "3" => break "ktransformers".to_string(),
                _ => println!("Invalid choice, please try again."),
            }
        }
    };

    println!("\nStep 3: Model path (optional)");
    print!("  Enter path to GGUF model [leave empty to skip]: ");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut model_input = String::new();
    std::io::stdin().read_line(&mut model_input).ok();
    let model = if model_input.trim().is_empty() {
        None
    } else {
        Some(model_input.trim().to_string())
    };

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                     Generated Configuration                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    configure(engine, model, None)?;

    println!("\n✓ Configuration complete!");
    println!("  Run 'openuma serve' for REST API access");
    println!("  Run 'openuma benchmark --model <path>' to test performance\n");

    Ok(())
}
