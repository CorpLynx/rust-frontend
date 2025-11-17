use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub backend: BackendSettings,
    pub ui: UISettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub window_title: String,
    pub window_width: f32,
    pub window_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSettings {
    pub url: String,
    pub ollama_url: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub font_size: u16,
    pub max_chat_history: usize,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from("config.toml");
        
        let settings = config::Config::builder()
            .add_source(config::File::from(config_path.clone()))
            .build()
            .with_context(|| format!("Failed to load config from {:?}", config_path))?;

        settings
            .try_deserialize()
            .context("Failed to deserialize config")
    }

    pub fn save(&self) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        
        std::fs::write("config.toml", toml_string)
            .context("Failed to write config.toml")?;
        
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                window_title: "AI Chat".to_string(),
                window_width: 800.0,
                window_height: 600.0,
            },
            backend: BackendSettings {
                url: "http://localhost:1234".to_string(),
                ollama_url: "http://localhost:1234".to_string(),
                timeout_seconds: 30,
            },
            ui: UISettings {
                font_size: 16,
                max_chat_history: 1000,
            },
        }
    }
}

