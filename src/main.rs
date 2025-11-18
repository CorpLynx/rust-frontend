use anyhow::Result;
use log::error;
use std::fs;

mod app;
mod config;
mod conversation;
mod icons;
mod markdown;
mod search;

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

    // Create conversations directory if it doesn't exist
    let conversations_dir = "conversations";
    if !std::path::Path::new(conversations_dir).exists() {
        fs::create_dir_all(conversations_dir)?;
    }

    // Create metadata file if it doesn't exist
    let metadata_path = format!("{}/metadata.json", conversations_dir);
    if !std::path::Path::new(&metadata_path).exists() {
        let empty_metadata = serde_json::json!({ "conversations": [] });
        fs::write(&metadata_path, serde_json::to_string_pretty(&empty_metadata)?)?;
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

