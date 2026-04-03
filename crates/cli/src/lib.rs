pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "openuma")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "OpenUMA - Unified Memory Abstraction for AI Inference")]
#[command(long_about = "OpenUMA detects shared memory hardware (AMD APUs, Intel iGPUs), \
computes optimal memory partitions, and generates configuration for AI inference engines.

Examples:
  openuma probe
  openuma configure --engine llamacpp --model model.gguf
  openuma configure --engine ollama --model model.gguf --output config.json
  openuma profiles
  openuma serve --port 8080")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Detect and display hardware profile
    Probe,
    
    /// Generate engine configuration with optimal llama.cpp flags
    #[command(long_about = "Generate optimal configuration for the specified inference engine.\n\n\
Examples:\n  openuma configure --engine llamacpp --model model.gguf\n  openuma configure --engine ollama --model model.gguf\n  openuma configure --engine ktransformers --model deepseek-v3.gguf")]
    Configure {
        /// Inference engine: llamacpp, ollama, or ktransformers
        #[arg(short, long, default_value = "llamacpp")]
        engine: String,
        
        /// Path to GGUF model file
        #[arg(short, long)]
        model: Option<String>,
        
        /// Write config to file
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Run inference benchmark
    Benchmark {
        /// Path to GGUF model file
        #[arg(short, long)]
        model: String,
        
        /// Number of tokens to generate
        #[arg(short, long, default_value = "100")]
        tokens: u32,
        
        /// Number of CPU threads
        #[arg(short, long)]
        threads: Option<u32>,
    },
    
    /// List known hardware profiles from database
    Profiles,
    
    /// Interactive configuration wizard
    Interactive,
    
    /// Start REST API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Probe => commands::probe(),
        Commands::Configure {
            engine,
            model,
            output,
        } => commands::configure(engine, model, output),
        Commands::Benchmark {
            model,
            tokens,
            threads,
        } => commands::benchmark(model, tokens, threads),
        Commands::Profiles => commands::list_profiles(),
        Commands::Interactive => commands::interactive(),
        Commands::Serve { port } => {
            tokio::runtime::Runtime::new()?.block_on(commands::serve(port));
            Ok(())
        }
    }
}
