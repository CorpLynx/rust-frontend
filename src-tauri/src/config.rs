use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use validator::Validate;

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
    #[serde(default)]
    pub saved_urls: Vec<String>,
    
    // New fields for remote integration
    #[serde(default)]
    pub remote_endpoints: Vec<RemoteEndpoint>,
    #[serde(default)]
    pub connection_mode: ConnectionMode,
    #[serde(default)]
    pub active_remote_endpoint_id: Option<String>,
    #[serde(default)]
    pub enable_encryption: bool,
}

impl BackendSettings {
    pub const LOCAL_OLLAMA_URL: &'static str = "http://localhost:11434";
    
    /// Add a URL to the saved URLs list with duplicate checking and size limiting
    pub fn add_saved_url(&mut self, url: String) {
        // Don't save localhost URLs
        if url.contains("localhost") || url.contains("127.0.0.1") {
            return;
        }
        
        // Don't add duplicates
        if !self.saved_urls.contains(&url) {
            self.saved_urls.insert(0, url);
            
            // Keep only last 10 URLs
            if self.saved_urls.len() > 10 {
                self.saved_urls.truncate(10);
            }
        }
    }
    
    /// Remove a URL from the saved URLs list
    pub fn remove_saved_url(&mut self, url: &str) {
        self.saved_urls.retain(|u| u != url);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RemoteEndpoint {
    pub id: String,
    #[validate(length(min = 1, message = "Endpoint name cannot be empty"))]
    pub name: String,
    #[validate(length(min = 1, message = "Host cannot be empty"))]
    pub host: String,
    #[validate(range(min = 1, max = 65535, message = "Port must be between 1 and 65535"))]
    pub port: u16,
    pub use_https: bool,
    pub api_key: Option<String>,
    pub last_tested: Option<String>,
    pub last_test_success: Option<bool>,
}

impl RemoteEndpoint {
    /// Create a new RemoteEndpoint with validation
    pub fn new(name: String, host: String, port: u16, use_https: bool, api_key: Option<String>) -> Result<Self, validator::ValidationErrors> {
        let endpoint = Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            host,
            port,
            use_https,
            api_key,
            last_tested: None,
            last_test_success: None,
        };
        
        endpoint.validate()?;
        Ok(endpoint)
    }
    
    /// Get the full URL for this endpoint
    pub fn url(&self) -> String {
        let protocol = if self.use_https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionMode {
    Local,
    Remote,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Local
    }
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
    /// Load application configuration from config.toml
    /// 
    /// This method loads the complete application configuration including saved URLs.
    /// The saved_urls field in BackendSettings uses #[serde(default)] to gracefully
    /// handle missing or corrupted saved URLs by defaulting to an empty vector.
    /// 
    /// # Requirement 3.5
    /// This ensures saved URLs are loaded from config on app startup and handles
    /// missing or corrupted saved URLs gracefully.
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
                window_title: "Prometheus".to_string(),
                window_width: 800.0,
                window_height: 600.0,
            },
            backend: BackendSettings {
                url: "http://localhost:1234".to_string(),
                ollama_url: "http://localhost:1234".to_string(),
                timeout_seconds: 30,
                saved_urls: Vec::new(),
                remote_endpoints: Vec::new(),
                connection_mode: ConnectionMode::Local,
                active_remote_endpoint_id: None,
                enable_encryption: false,
            },
            ui: UISettings {
                font_size: 16,
                max_chat_history: 1000,
                theme: "Hacker Green".to_string(),
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    /// Test that saved URLs are loaded from config on app startup
    /// This verifies Requirement 3.5
    #[test]
    fn test_saved_urls_loaded_on_startup() {
        use std::fs;
        use std::path::PathBuf;

        // Create a test config file with saved URLs
        let test_config_path = PathBuf::from("test_config_startup.toml");
        let test_config = r#"
[app]
window_title = "Test App"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = [
    "https://api.example1.com",
    "https://api.example2.com",
    "https://api.example3.com"
]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

        // Write test config
        fs::write(&test_config_path, test_config).expect("Failed to write test config");

        // Load config
        let loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        // Clean up
        fs::remove_file(&test_config_path).ok();

        // Verify saved URLs were loaded
        assert_eq!(loaded_config.backend.saved_urls.len(), 3);
        assert!(loaded_config.backend.saved_urls.contains(&"https://api.example1.com".to_string()));
        assert!(loaded_config.backend.saved_urls.contains(&"https://api.example2.com".to_string()));
        assert!(loaded_config.backend.saved_urls.contains(&"https://api.example3.com".to_string()));
    }

    /// Test that missing saved_urls field defaults to empty vector
    /// This verifies graceful handling of missing saved URLs
    #[test]
    fn test_missing_saved_urls_defaults_to_empty() {
        use std::fs;
        use std::path::PathBuf;

        // Create a test config file WITHOUT saved_urls field
        let test_config_path = PathBuf::from("test_config_missing_urls.toml");
        let test_config = r#"
[app]
window_title = "Test App"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

        // Write test config
        fs::write(&test_config_path, test_config).expect("Failed to write test config");

        // Load config
        let loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        // Clean up
        fs::remove_file(&test_config_path).ok();

        // Verify saved URLs defaults to empty vector
        assert_eq!(loaded_config.backend.saved_urls.len(), 0);
        assert!(loaded_config.backend.saved_urls.is_empty());
    }

    /// Test that corrupted saved_urls field (wrong type) is handled gracefully
    /// This verifies graceful handling of corrupted saved URLs
    #[test]
    fn test_corrupted_saved_urls_handled_gracefully() {
        use std::fs;
        use std::path::PathBuf;

        // Create a test config file with corrupted saved_urls (string instead of array)
        let test_config_path = PathBuf::from("test_config_corrupted_urls.toml");
        let test_config = r#"
[app]
window_title = "Test App"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = "not an array"

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

        // Write test config
        fs::write(&test_config_path, test_config).expect("Failed to write test config");

        // Try to load config - should fail gracefully
        let result = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .and_then(|cfg| cfg.try_deserialize::<AppConfig>());

        // Clean up
        fs::remove_file(&test_config_path).ok();

        // The config should fail to load due to type mismatch
        // In the actual app, this is handled by unwrap_or_default()
        assert!(result.is_err());
    }

    /// **Feature: enhanced-settings-ui, Property 1: URL addition preserves uniqueness**
    /// **Validates: Requirements 3.1, 3.3**
    /// 
    /// For any backend URL that is not localhost, when added to the saved URLs list,
    /// the URL should appear exactly once in the list.
    #[quickcheck]
    fn prop_url_addition_preserves_uniqueness(urls: Vec<String>) -> TestResult {
        // Filter out localhost URLs as they shouldn't be added per the spec
        // Also filter out strings that are not reasonable URL-like strings
        let valid_urls: Vec<String> = urls
            .into_iter()
            .filter(|url| !url.contains("localhost") && !url.contains("127.0.0.1"))
            .filter(|url| !url.is_empty())
            .filter(|url| url.chars().all(|c| c.is_ascii_graphic())) // Only printable ASCII
            .filter(|url| url.len() > 0) // Non-empty after filtering
            .collect();

        if valid_urls.is_empty() {
            return TestResult::discard();
        }

        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            enable_encryption: false,
        };

        // Add all URLs (including duplicates)
        for url in &valid_urls {
            settings.add_saved_url(url.clone());
        }

        // Check that each unique URL appears exactly once
        for url in &valid_urls {
            let count = settings.saved_urls.iter().filter(|u| *u == url).count();
            if count != 1 {
                return TestResult::failed();
            }
        }

        TestResult::passed()
    }

    /// Additional test: Verify that adding the same URL multiple times results in only one entry
    #[test]
    fn test_duplicate_url_not_added() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            enable_encryption: false,
        };

        let test_url = "https://api.example.com/v1".to_string();
        
        // Add the same URL three times
        settings.add_saved_url(test_url.clone());
        settings.add_saved_url(test_url.clone());
        settings.add_saved_url(test_url.clone());

        // Should only appear once
        assert_eq!(settings.saved_urls.len(), 1);
        assert_eq!(settings.saved_urls[0], test_url);
    }

    /// Test that localhost URLs are not added to saved_urls
    #[test]
    fn test_localhost_urls_not_saved() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            enable_encryption: false,
        };

        settings.add_saved_url("http://localhost:8080".to_string());
        settings.add_saved_url("http://127.0.0.1:8080".to_string());
        settings.add_saved_url("https://api.example.com".to_string());

        // Only the non-localhost URL should be saved
        assert_eq!(settings.saved_urls.len(), 1);
        assert_eq!(settings.saved_urls[0], "https://api.example.com");
    }

    /// **Feature: enhanced-settings-ui, Property 2: URL list size is bounded**
    /// **Validates: Requirements 3.4**
    /// 
    /// For any sequence of URL additions, the saved URLs list should never exceed 10 entries,
    /// and when the 11th URL is added, the oldest URL should be removed.
    #[quickcheck]
    fn prop_url_list_size_is_bounded(urls: Vec<String>) -> TestResult {
        // Filter out localhost URLs and ensure we have valid, unique URLs
        let valid_urls: Vec<String> = urls
            .into_iter()
            .filter(|url| !url.contains("localhost") && !url.contains("127.0.0.1"))
            .filter(|url| !url.is_empty())
            .filter(|url| url.chars().all(|c| c.is_ascii_graphic()))
            .collect();

        // We need at least 11 unique URLs to test the boundary condition
        let unique_urls: Vec<String> = valid_urls
            .into_iter()
            .enumerate()
            .map(|(i, url)| format!("{}_{}", url, i)) // Make each URL unique
            .collect();

        if unique_urls.len() < 11 {
            return TestResult::discard();
        }

        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            enable_encryption: false,
        };

        // Add URLs one by one and verify the list never exceeds 10
        for (i, url) in unique_urls.iter().enumerate() {
            settings.add_saved_url(url.clone());
            
            // The list should never exceed 10 entries
            if settings.saved_urls.len() > 10 {
                return TestResult::failed();
            }
            
            // After adding 10 or more URLs, the list should be exactly 10
            if i >= 9 && settings.saved_urls.len() != 10 {
                return TestResult::failed();
            }
        }

        // Verify that the oldest URLs were removed (most recent 10 should remain)
        // The list is ordered with most recent first, so check the last 10 added URLs
        let expected_urls: Vec<String> = unique_urls
            .iter()
            .rev()
            .take(10)
            .rev()
            .cloned()
            .collect();

        if settings.saved_urls != expected_urls {
            return TestResult::failed();
        }

        TestResult::passed()
    }

    /// **Feature: enhanced-settings-ui, Property 3: URL persistence round-trip**
    /// **Validates: Requirements 3.2**
    /// 
    /// For any saved URL, after persisting the configuration and reloading it,
    /// the URL should still be present in the saved URLs list.
    #[quickcheck]
    fn prop_url_persistence_round_trip(urls: Vec<String>) -> TestResult {
        use std::fs;
        use std::path::PathBuf;

        // Filter out localhost URLs and ensure we have valid URLs
        let valid_urls: Vec<String> = urls
            .into_iter()
            .filter(|url| !url.contains("localhost") && !url.contains("127.0.0.1"))
            .filter(|url| !url.is_empty())
            .filter(|url| url.chars().all(|c| c.is_ascii_graphic()))
            .take(5) // Limit to 5 URLs for faster testing
            .enumerate()
            .map(|(i, url)| format!("https://api{}.example.com/{}", i, url))
            .collect();

        if valid_urls.is_empty() {
            return TestResult::discard();
        }

        // Create a unique test config file path
        let test_config_path = PathBuf::from(format!(
            "test_config_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create a config with saved URLs
        let mut config = AppConfig::default();
        for url in &valid_urls {
            config.backend.add_saved_url(url.clone());
        }

        // Save the config to a test file
        let toml_string = match toml::to_string_pretty(&config) {
            Ok(s) => s,
            Err(_) => {
                return TestResult::error("Failed to serialize config");
            }
        };

        if let Err(_) = fs::write(&test_config_path, toml_string) {
            return TestResult::error("Failed to write test config file");
        }

        // Load the config back
        let loaded_config = match config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
        {
            Ok(settings) => match settings.try_deserialize::<AppConfig>() {
                Ok(cfg) => cfg,
                Err(_) => {
                    let _ = fs::remove_file(&test_config_path);
                    return TestResult::error("Failed to deserialize config");
                }
            },
            Err(_) => {
                let _ = fs::remove_file(&test_config_path);
                return TestResult::error("Failed to load config");
            }
        };

        // Clean up test file
        let _ = fs::remove_file(&test_config_path);

        // Verify all URLs are present in the loaded config
        for url in &valid_urls {
            if !loaded_config.backend.saved_urls.contains(url) {
                return TestResult::failed();
            }
        }

        // Verify the count matches
        if loaded_config.backend.saved_urls.len() != valid_urls.len() {
            return TestResult::failed();
        }

        TestResult::passed()
    }

    /// **Feature: enhanced-settings-ui, Property 6: URL deletion removes from list**
    /// **Validates: Requirements 5.2**
    /// 
    /// For any URL in the saved URLs list, after deletion, the URL should not appear
    /// in the saved URLs list.
    #[quickcheck]
    fn prop_url_deletion_removes_from_list(urls: Vec<String>) -> TestResult {
        // Filter out localhost URLs and ensure we have valid URLs
        let valid_urls: Vec<String> = urls
            .into_iter()
            .filter(|url| !url.contains("localhost") && !url.contains("127.0.0.1"))
            .filter(|url| !url.is_empty())
            .filter(|url| url.chars().all(|c| c.is_ascii_graphic()))
            .take(10) // Limit to 10 URLs (the max)
            .enumerate()
            .map(|(i, url)| format!("https://api{}.example.com/{}", i, url))
            .collect();

        if valid_urls.is_empty() {
            return TestResult::discard();
        }

        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            enable_encryption: false,
        };

        // Add all URLs to the list
        for url in &valid_urls {
            settings.add_saved_url(url.clone());
        }

        // Verify all URLs are present before deletion
        for url in &valid_urls {
            if !settings.saved_urls.contains(url) {
                return TestResult::error("URL not added properly");
            }
        }

        // Delete each URL and verify it's removed
        for url in &valid_urls {
            let initial_count = settings.saved_urls.len();
            
            settings.remove_saved_url(url);
            
            // The URL should no longer be in the list
            if settings.saved_urls.contains(url) {
                return TestResult::failed();
            }
            
            // The list should be one element shorter
            if settings.saved_urls.len() != initial_count - 1 {
                return TestResult::failed();
            }
        }

        // After deleting all URLs, the list should be empty
        if !settings.saved_urls.is_empty() {
            return TestResult::failed();
        }

        TestResult::passed()
    }
}
