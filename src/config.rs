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
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "Hacker Green".to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorTheme {
    HackerGreen,
    CyberBlue,
    NeonPurple,
    MatrixRed,
}

impl ColorTheme {
    pub fn all() -> Vec<String> {
        vec![
            "Hacker Green".to_string(),
            "Cyber Blue".to_string(),
            "Neon Purple".to_string(),
            "Matrix Red".to_string(),
        ]
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "Cyber Blue" => ColorTheme::CyberBlue,
            "Neon Purple" => ColorTheme::NeonPurple,
            "Matrix Red" => ColorTheme::MatrixRed,
            _ => ColorTheme::HackerGreen, // Default to preserve current look
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ColorTheme::HackerGreen => "Hacker Green".to_string(),
            ColorTheme::CyberBlue => "Cyber Blue".to_string(),
            ColorTheme::NeonPurple => "Neon Purple".to_string(),
            ColorTheme::MatrixRed => "Matrix Red".to_string(),
        }
    }

    pub fn primary_color(&self) -> (f32, f32, f32) {
        match self {
            ColorTheme::HackerGreen => (0.0, 1.0, 0.6),  // Cyan-green (current default)
            ColorTheme::CyberBlue => (0.0, 0.8, 1.0),    // Bright blue
            ColorTheme::NeonPurple => (0.8, 0.4, 1.0),   // Purple
            ColorTheme::MatrixRed => (1.0, 0.2, 0.4),    // Red
        }
    }

    pub fn secondary_color(&self) -> (f32, f32, f32) {
        match self {
            ColorTheme::HackerGreen => (0.0, 0.7, 0.5),  // Darker green
            ColorTheme::CyberBlue => (0.0, 0.6, 0.8),    // Darker blue
            ColorTheme::NeonPurple => (0.6, 0.2, 0.8),   // Darker purple
            ColorTheme::MatrixRed => (0.8, 0.1, 0.3),    // Darker red
        }
    }
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
                theme: "Hacker Green".to_string(),
            },
        }
    }
}

