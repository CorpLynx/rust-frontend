use anyhow::{Context, Result};
use clap::Parser;
use prometheus::cli::app::CliApp;
use prometheus::config::AppConfig;

/// Prometheus CLI - Terminal-based AI chat interface
#[derive(Parser, Debug)]
#[command(name = "prometheus-cli")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ollama backend URL (overrides config file)
    #[arg(short, long, value_name = "URL")]
    url: Option<String>,

    /// Model name to use for chat (overrides config file)
    #[arg(short, long, value_name = "MODEL")]
    model: Option<String>,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Load configuration with fallback to defaults
    let config = match AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: Failed to load config from {}: {}", args.config, e);
            eprintln!("Using default configuration");
            AppConfig::default()
        }
    };

    // Create and run CLI app with CLI argument overrides
    let mut app = CliApp::new(config, args.url, args.model)
        .context("Failed to initialize CLI application")?;

    app.run().await
}
