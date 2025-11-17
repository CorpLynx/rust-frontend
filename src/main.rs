use anyhow::Result;
use log::error;
use std::fs;

mod app;
mod config;

use app::ChatApp;
use config::AppConfig;
use iced::Application;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Create log directory if it doesn't exist
    let log_dir = "logs";
    if !std::path::Path::new(log_dir).exists() {
        fs::create_dir_all(log_dir)?;
    }

    // Load configuration for window size
    let config = AppConfig::load().unwrap_or_default();
    
    // Run the application
    let result = ChatApp::run(iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(config.app.window_width, config.app.window_height),
            ..Default::default()
        },
        ..Default::default()
    });

    if let Err(e) = result {
        error!("Application error: {}", e);
        eprintln!("Application error: {}", e);
    }

    Ok(())
}

