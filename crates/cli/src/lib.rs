pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "openuma")]
#[command(version = "0.1.0")]
#[command(about = "OpenUMA - Unified Model Acceleration")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Probe,
    Configure {
        #[arg(short, long, default_value = "llamacpp")]
        engine: String,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    Benchmark {
        #[arg(short, long)]
        model: String,
        #[arg(short, long, default_value = "100")]
        tokens: u32,
        #[arg(short, long)]
        threads: Option<u32>,
    },
    Profiles,
    Serve {
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
        Commands::Serve { port } => commands::serve(port),
    }
}
