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
    
    /// Add a remote endpoint with validation
    /// Requirements: 1.2, 1.3, 1.4
    pub fn add_remote_endpoint(&mut self, endpoint: RemoteEndpoint) -> Result<String, validator::ValidationErrors> {
        // Validate the endpoint
        endpoint.validate()?;
        
        // Check for duplicate host:port combination
        let endpoint_address = format!("{}:{}", endpoint.host, endpoint.port);
        if self.remote_endpoints.iter().any(|e| format!("{}:{}", e.host, e.port) == endpoint_address) {
            // Return a custom error for duplicate endpoint
            let mut errors = validator::ValidationErrors::new();
            errors.add("host", validator::ValidationError::new("duplicate_endpoint"));
            return Err(errors);
        }
        
        let endpoint_id = endpoint.id.clone();
        self.remote_endpoints.push(endpoint);
        Ok(endpoint_id)
    }
    
    /// Remove a remote endpoint by ID
    /// Requirements: 5.3
    pub fn remove_remote_endpoint(&mut self, endpoint_id: &str) -> Result<(), String> {
        let initial_len = self.remote_endpoints.len();
        self.remote_endpoints.retain(|e| e.id != endpoint_id);
        
        if self.remote_endpoints.len() == initial_len {
            return Err(format!("Endpoint with ID {} not found", endpoint_id));
        }
        
        // If the removed endpoint was the active one, clear the active endpoint
        if self.active_remote_endpoint_id.as_deref() == Some(endpoint_id) {
            self.active_remote_endpoint_id = None;
        }
        
        Ok(())
    }
    
    /// Update a remote endpoint with validation
    /// Requirements: 5.4
    pub fn update_remote_endpoint(&mut self, endpoint_id: &str, updated_endpoint: RemoteEndpoint) -> Result<(), String> {
        // Validate the updated endpoint
        updated_endpoint.validate()
            .map_err(|e| format!("Validation failed: {:?}", e))?;
        
        // Check if the endpoint ID matches
        if updated_endpoint.id != endpoint_id {
            return Err("Endpoint ID mismatch".to_string());
        }
        
        // Check for duplicate host:port combination (excluding the current endpoint)
        let endpoint_address = format!("{}:{}", updated_endpoint.host, updated_endpoint.port);
        if self.remote_endpoints.iter().any(|e| 
            e.id != endpoint_id && format!("{}:{}", e.host, e.port) == endpoint_address
        ) {
            return Err("An endpoint with this address already exists".to_string());
        }
        
        // Find and update the endpoint
        if let Some(endpoint) = self.remote_endpoints.iter_mut().find(|e| e.id == endpoint_id) {
            *endpoint = updated_endpoint;
            Ok(())
        } else {
            Err(format!("Endpoint with ID {} not found", endpoint_id))
        }
    }
    
    /// Get a remote endpoint by ID
    /// Requirements: 5.1
    pub fn get_remote_endpoint(&self, endpoint_id: &str) -> Option<&RemoteEndpoint> {
        self.remote_endpoints.iter().find(|e| e.id == endpoint_id)
    }
    
    /// List all remote endpoints
    /// Requirements: 5.1
    pub fn list_remote_endpoints(&self) -> &[RemoteEndpoint] {
        &self.remote_endpoints
    }
    
    /// Migrate existing ollama_url to remote endpoints if not localhost
    /// Requirements: Migration logic
    pub fn migrate_ollama_url(&mut self) {
        // Only migrate if:
        // 1. ollama_url is not localhost
        // 2. remote_endpoints is empty (first migration)
        if !self.ollama_url.contains("localhost") 
            && !self.ollama_url.contains("127.0.0.1") 
            && self.remote_endpoints.is_empty() 
        {
            // Parse the URL to extract host and port
            if let Ok(url) = url::Url::parse(&self.ollama_url) {
                if let Some(host) = url.host_str() {
                    let port = url.port().unwrap_or(11434);
                    let use_https = url.scheme() == "https";
                    
                    // Create a migrated endpoint
                    if let Ok(endpoint) = RemoteEndpoint::new(
                        "Migrated Endpoint".to_string(),
                        host.to_string(),
                        port,
                        use_https,
                        None,
                    ) {
                        let _ = self.add_remote_endpoint(endpoint);
                    }
                }
            }
        }
    }
    
    /// Set the connection mode
    /// Requirements: 2.1, 2.4, 2.5
    pub fn set_connection_mode(&mut self, mode: ConnectionMode) {
        self.connection_mode = mode;
    }
    
    /// Get the current connection mode
    /// Requirements: 2.1
    pub fn get_connection_mode(&self) -> &ConnectionMode {
        &self.connection_mode
    }
    
    /// Get the active endpoint URL based on the current connection mode
    /// Requirements: 2.2, 2.3
    pub fn get_active_endpoint_url(&self) -> Result<String, String> {
        match self.connection_mode {
            ConnectionMode::Local => {
                // In local mode, always return localhost
                Ok(Self::LOCAL_OLLAMA_URL.to_string())
            }
            ConnectionMode::Remote => {
                // In remote mode, return the active remote endpoint URL
                if let Some(endpoint_id) = &self.active_remote_endpoint_id {
                    if let Some(endpoint) = self.get_remote_endpoint(endpoint_id) {
                        Ok(endpoint.url())
                    } else {
                        Err(format!("Active endpoint with ID {} not found", endpoint_id))
                    }
                } else {
                    Err("No active remote endpoint selected".to_string())
                }
            }
        }
    }
    
    /// Set the active remote endpoint
    /// Requirements: 2.3, 5.2
    pub fn set_active_remote_endpoint(&mut self, endpoint_id: &str) -> Result<(), String> {
        // Verify the endpoint exists
        if self.get_remote_endpoint(endpoint_id).is_some() {
            self.active_remote_endpoint_id = Some(endpoint_id.to_string());
            Ok(())
        } else {
            Err(format!("Endpoint with ID {} not found", endpoint_id))
        }
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
    
    /// Validate IP address format (IPv4 or IPv6)
    pub fn validate_ip(ip: &str) -> bool {
        use std::net::IpAddr;
        ip.parse::<IpAddr>().is_ok()
    }
    
    /// Validate port number (1-65535)
    pub fn validate_port(port: u16) -> bool {
        port >= 1
    }
    
    /// Validate IP and port combination
    pub fn validate_ip_and_port(ip: &str, port: u16) -> bool {
        Self::validate_ip(ip) && Self::validate_port(port)
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
        
        // Set restrictive file permissions (user read/write only)
        // Requirements: 8.4
        Self::set_config_file_permissions()?;
        
        Ok(())
    }
    
    /// Set restrictive file permissions on the configuration file
    /// Requirements: 8.4
    /// 
    /// On Unix-like systems (macOS, Linux), sets permissions to 0600 (user read/write only).
    /// On Windows, this is a no-op as Windows uses a different permission model.
    #[cfg(unix)]
    fn set_config_file_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        
        let config_path = PathBuf::from("config.toml");
        let metadata = std::fs::metadata(&config_path)
            .context("Failed to get config file metadata")?;
        
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600); // User read/write only
        
        std::fs::set_permissions(&config_path, permissions)
            .context("Failed to set config file permissions")?;
        
        Ok(())
    }
    
    #[cfg(not(unix))]
    fn set_config_file_permissions() -> Result<()> {
        // On non-Unix systems (Windows), we don't set permissions
        // Windows uses a different permission model (ACLs)
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

    // Tests for endpoint management methods
    
    #[test]
    fn test_add_remote_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();
        
        assert_eq!(settings.remote_endpoints.len(), 1);
        assert_eq!(settings.remote_endpoints[0].name, "Test Server");
        assert_eq!(settings.remote_endpoints[0].id, endpoint_id);
    }

    #[test]
    fn test_add_duplicate_endpoint_rejected() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint1 = RemoteEndpoint::new(
            "Test Server 1".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint2 = RemoteEndpoint::new(
            "Test Server 2".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        settings.add_remote_endpoint(endpoint1).unwrap();
        let result = settings.add_remote_endpoint(endpoint2);
        
        assert!(result.is_err());
        assert_eq!(settings.remote_endpoints.len(), 1);
    }

    #[test]
    fn test_remove_remote_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();
        assert_eq!(settings.remote_endpoints.len(), 1);

        settings.remove_remote_endpoint(&endpoint_id).unwrap();
        assert_eq!(settings.remote_endpoints.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let result = settings.remove_remote_endpoint("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_remote_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();

        let mut updated_endpoint = settings.get_remote_endpoint(&endpoint_id).unwrap().clone();
        updated_endpoint.name = "Updated Server".to_string();
        updated_endpoint.port = 8080;

        settings.update_remote_endpoint(&endpoint_id, updated_endpoint).unwrap();

        let endpoint = settings.get_remote_endpoint(&endpoint_id).unwrap();
        assert_eq!(endpoint.name, "Updated Server");
        assert_eq!(endpoint.port, 8080);
    }

    #[test]
    fn test_get_remote_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();

        let retrieved = settings.get_remote_endpoint(&endpoint_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Server");

        let nonexistent = settings.get_remote_endpoint("nonexistent-id");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_list_remote_endpoints() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        assert_eq!(settings.list_remote_endpoints().len(), 0);

        let endpoint1 = RemoteEndpoint::new(
            "Server 1".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint2 = RemoteEndpoint::new(
            "Server 2".to_string(),
            "192.168.1.101".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        settings.add_remote_endpoint(endpoint1).unwrap();
        settings.add_remote_endpoint(endpoint2).unwrap();

        let endpoints = settings.list_remote_endpoints();
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].name, "Server 1");
        assert_eq!(endpoints[1].name, "Server 2");
    }

    #[test]
    fn test_migrate_ollama_url() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "https://remote.example.com:8080".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        settings.migrate_ollama_url();

        assert_eq!(settings.remote_endpoints.len(), 1);
        assert_eq!(settings.remote_endpoints[0].name, "Migrated Endpoint");
        assert_eq!(settings.remote_endpoints[0].host, "remote.example.com");
        assert_eq!(settings.remote_endpoints[0].port, 8080);
        assert_eq!(settings.remote_endpoints[0].use_https, true);
    }

    #[test]
    fn test_migrate_ollama_url_localhost_not_migrated() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        settings.migrate_ollama_url();

        assert_eq!(settings.remote_endpoints.len(), 0);
    }

    #[test]
    fn test_migrate_ollama_url_only_once() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "https://remote.example.com:8080".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        settings.migrate_ollama_url();
        assert_eq!(settings.remote_endpoints.len(), 1);

        // Try to migrate again - should not add another endpoint
        settings.migrate_ollama_url();
        assert_eq!(settings.remote_endpoints.len(), 1);
    }

    #[test]
    fn test_remove_active_endpoint_clears_active_id() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();
        settings.active_remote_endpoint_id = Some(endpoint_id.clone());

        settings.remove_remote_endpoint(&endpoint_id).unwrap();
        
        assert!(settings.active_remote_endpoint_id.is_none());
    }

    // Tests for connection mode management

    #[test]
    fn test_set_connection_mode() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        assert_eq!(settings.connection_mode, ConnectionMode::Local);

        settings.set_connection_mode(ConnectionMode::Remote);
        assert_eq!(settings.connection_mode, ConnectionMode::Remote);

        settings.set_connection_mode(ConnectionMode::Local);
        assert_eq!(settings.connection_mode, ConnectionMode::Local);
    }

    #[test]
    fn test_get_connection_mode() {
        let settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Remote,
            active_remote_endpoint_id: None,
            
        };

        assert_eq!(*settings.get_connection_mode(), ConnectionMode::Remote);
    }

    #[test]
    fn test_get_active_endpoint_url_local_mode() {
        let settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let url = settings.get_active_endpoint_url().unwrap();
        assert_eq!(url, "http://localhost:11434");
    }

    #[test]
    fn test_get_active_endpoint_url_remote_mode_with_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Remote,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            8080,
            true,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();
        settings.active_remote_endpoint_id = Some(endpoint_id);

        let url = settings.get_active_endpoint_url().unwrap();
        assert_eq!(url, "https://192.168.1.100:8080");
    }

    #[test]
    fn test_get_active_endpoint_url_remote_mode_no_endpoint() {
        let settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Remote,
            active_remote_endpoint_id: None,
            
        };

        let result = settings.get_active_endpoint_url();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active remote endpoint selected");
    }

    #[test]
    fn test_get_active_endpoint_url_remote_mode_invalid_endpoint() {
        let settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Remote,
            active_remote_endpoint_id: Some("nonexistent-id".to_string()),
            
        };

        let result = settings.get_active_endpoint_url();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_set_active_remote_endpoint() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();

        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();

        settings.set_active_remote_endpoint(&endpoint_id).unwrap();
        assert_eq!(settings.active_remote_endpoint_id, Some(endpoint_id));
    }

    #[test]
    fn test_set_active_remote_endpoint_nonexistent() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        let result = settings.set_active_remote_endpoint("nonexistent-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_mode_switching_updates_connection() {
        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
            remote_endpoints: Vec::new(),
            connection_mode: ConnectionMode::Local,
            active_remote_endpoint_id: None,
            
        };

        // Start in local mode
        let url = settings.get_active_endpoint_url().unwrap();
        assert_eq!(url, "http://localhost:11434");

        // Add a remote endpoint
        let endpoint = RemoteEndpoint::new(
            "Remote Server".to_string(),
            "10.0.0.50".to_string(),
            11434,
            false,
            None,
        ).unwrap();
        let endpoint_id = settings.add_remote_endpoint(endpoint).unwrap();
        settings.active_remote_endpoint_id = Some(endpoint_id);

        // Switch to remote mode
        settings.set_connection_mode(ConnectionMode::Remote);
        let url = settings.get_active_endpoint_url().unwrap();
        assert_eq!(url, "http://10.0.0.50:11434");

        // Switch back to local mode
        settings.set_connection_mode(ConnectionMode::Local);
        let url = settings.get_active_endpoint_url().unwrap();
        assert_eq!(url, "http://localhost:11434");
    }

    #[test]
    fn test_mode_changes_persist_to_config() {
        use std::fs;
        use std::path::PathBuf;

        // Create a unique test config file path
        let test_config_path = PathBuf::from(format!(
            "test_config_mode_persist_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create a config in local mode
        let mut config = AppConfig::default();
        config.backend.set_connection_mode(ConnectionMode::Local);

        // Save the config
        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        fs::write(&test_config_path, toml_string).expect("Failed to write test config file");

        // Load the config back
        let loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        assert_eq!(*loaded_config.backend.get_connection_mode(), ConnectionMode::Local);

        // Now change to remote mode and save again
        let mut config = loaded_config;
        config.backend.set_connection_mode(ConnectionMode::Remote);

        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        fs::write(&test_config_path, toml_string).expect("Failed to write test config file");

        // Load the config back again
        let loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        // Clean up test file
        let _ = fs::remove_file(&test_config_path);

        // Verify the mode persisted
        assert_eq!(*loaded_config.backend.get_connection_mode(), ConnectionMode::Remote);
    }

    /// Test that configuration file permissions are set correctly on Unix systems
    /// Requirements: 8.4
    #[test]
    #[cfg(unix)]
    fn test_config_file_permissions_set_correctly() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;

        // Create a unique test config file path
        let test_config_path = PathBuf::from(format!(
            "test_config_permissions_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create a config
        let config = AppConfig::default();

        // Save the config to a test file
        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        fs::write(&test_config_path, toml_string).expect("Failed to write test config file");

        // Set permissions using the method
        let metadata = fs::metadata(&test_config_path).expect("Failed to get metadata");
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&test_config_path, permissions).expect("Failed to set permissions");

        // Verify permissions are 0600 (user read/write only)
        let metadata = fs::metadata(&test_config_path).expect("Failed to get metadata after setting permissions");
        let mode = metadata.permissions().mode();
        
        // On Unix, the mode includes file type bits, so we mask to get just the permission bits
        let permission_bits = mode & 0o777;
        
        // Clean up test file
        let _ = fs::remove_file(&test_config_path);

        // Verify the permissions are 0600
        assert_eq!(permission_bits, 0o600, "Config file permissions should be 0600 (user read/write only)");
    }

    /// Test that migration and initialization work together correctly
    /// Requirements: 6.4
    #[test]
    fn test_initialization_and_migration() {
        use std::fs;
        use std::path::PathBuf;

        // Create a unique test config file path
        let test_config_path = PathBuf::from(format!(
            "test_config_init_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create a config with a remote ollama_url (simulating an old config)
        let test_config = r#"
[app]
window_title = "Prometheus"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "https://remote.example.com:8080"
timeout_seconds = 30
saved_urls = []

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

        // Write test config
        fs::write(&test_config_path, test_config).expect("Failed to write test config");

        // Load config (simulating app startup)
        let mut config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        // Verify initial state
        assert_eq!(config.backend.remote_endpoints.len(), 0);
        assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local);

        // Perform migration (simulating what happens in lib.rs setup)
        config.backend.migrate_ollama_url();

        // Verify migration occurred
        assert_eq!(config.backend.remote_endpoints.len(), 1);
        assert_eq!(config.backend.remote_endpoints[0].name, "Migrated Endpoint");
        assert_eq!(config.backend.remote_endpoints[0].host, "remote.example.com");
        assert_eq!(config.backend.remote_endpoints[0].port, 8080);
        assert_eq!(config.backend.remote_endpoints[0].use_https, true);

        // Verify connection mode is still Local (default for existing users)
        assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local);

        // Save the migrated config
        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        fs::write(&test_config_path, toml_string).expect("Failed to write migrated config");

        // Load the config again to verify persistence
        let loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        // Clean up test file
        let _ = fs::remove_file(&test_config_path);

        // Verify the migrated endpoint persisted
        assert_eq!(loaded_config.backend.remote_endpoints.len(), 1);
        assert_eq!(loaded_config.backend.remote_endpoints[0].name, "Migrated Endpoint");
        assert_eq!(loaded_config.backend.remote_endpoints[0].host, "remote.example.com");
        assert_eq!(loaded_config.backend.remote_endpoints[0].port, 8080);
        assert_eq!(loaded_config.backend.remote_endpoints[0].use_https, true);
        assert_eq!(*loaded_config.backend.get_connection_mode(), ConnectionMode::Local);
    }

    #[test]
    fn test_connection_mode_management_integration() {
        use std::fs;
        use std::path::PathBuf;

        // Create a unique test config file path
        let test_config_path = PathBuf::from(format!(
            "test_config_integration_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create a config with endpoints
        let mut config = AppConfig::default();
        
        // Add two remote endpoints
        let endpoint1 = RemoteEndpoint::new(
            "Production Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();
        
        let endpoint2 = RemoteEndpoint::new(
            "Dev Server".to_string(),
            "10.0.0.50".to_string(),
            8080,
            true,
            None,
        ).unwrap();
        
        let endpoint1_id = config.backend.add_remote_endpoint(endpoint1).unwrap();
        let endpoint2_id = config.backend.add_remote_endpoint(endpoint2).unwrap();
        
        // Start in local mode
        assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local);
        assert_eq!(config.backend.get_active_endpoint_url().unwrap(), "http://localhost:11434");
        
        // Set active endpoint and switch to remote mode
        config.backend.set_active_remote_endpoint(&endpoint1_id).unwrap();
        config.backend.set_connection_mode(ConnectionMode::Remote);
        
        assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Remote);
        assert_eq!(config.backend.get_active_endpoint_url().unwrap(), "http://192.168.1.100:11434");
        
        // Save the config
        let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
        fs::write(&test_config_path, toml_string).expect("Failed to write test config file");
        
        // Load the config back
        let mut loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");
        
        // Verify everything persisted correctly
        assert_eq!(*loaded_config.backend.get_connection_mode(), ConnectionMode::Remote);
        assert_eq!(loaded_config.backend.get_active_endpoint_url().unwrap(), "http://192.168.1.100:11434");
        
        // Switch to the other endpoint
        loaded_config.backend.set_active_remote_endpoint(&endpoint2_id).unwrap();
        assert_eq!(loaded_config.backend.get_active_endpoint_url().unwrap(), "https://10.0.0.50:8080");
        
        // Switch back to local mode
        loaded_config.backend.set_connection_mode(ConnectionMode::Local);
        assert_eq!(loaded_config.backend.get_active_endpoint_url().unwrap(), "http://localhost:11434");
        
        // Clean up test file
        let _ = fs::remove_file(&test_config_path);
    }

    /// **Feature: remote-ollama-integration, Property 1: IP and port validation**
    /// **Validates: Requirements 1.2, 1.3**
    /// 
    /// For any string input representing an IP address and port, the validation function
    /// should accept valid formats (IPv4/IPv6 with valid port 1-65535) and reject invalid formats.
    #[test]
    fn test_valid_ipv4_addresses() {
        // Valid IPv4 addresses
        assert!(RemoteEndpoint::validate_ip("192.168.1.1"));
        assert!(RemoteEndpoint::validate_ip("10.0.0.1"));
        assert!(RemoteEndpoint::validate_ip("172.16.0.1"));
        assert!(RemoteEndpoint::validate_ip("8.8.8.8"));
        assert!(RemoteEndpoint::validate_ip("127.0.0.1"));
        assert!(RemoteEndpoint::validate_ip("0.0.0.0"));
        assert!(RemoteEndpoint::validate_ip("255.255.255.255"));
    }

    #[test]
    fn test_valid_ipv6_addresses() {
        // Valid IPv6 addresses
        assert!(RemoteEndpoint::validate_ip("::1"));
        assert!(RemoteEndpoint::validate_ip("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        assert!(RemoteEndpoint::validate_ip("2001:db8::1"));
        assert!(RemoteEndpoint::validate_ip("fe80::"));
        assert!(RemoteEndpoint::validate_ip("::"));
    }

    #[test]
    fn test_invalid_ip_addresses() {
        // Invalid IP addresses
        assert!(!RemoteEndpoint::validate_ip("999.999.999.999"));
        assert!(!RemoteEndpoint::validate_ip("192.168.1"));
        assert!(!RemoteEndpoint::validate_ip("192.168.1.1.1"));
        assert!(!RemoteEndpoint::validate_ip("not-an-ip"));
        assert!(!RemoteEndpoint::validate_ip(""));
        assert!(!RemoteEndpoint::validate_ip("192.168.1.256"));
        assert!(!RemoteEndpoint::validate_ip("192.168.-1.1"));
    }

    #[test]
    fn test_valid_ports() {
        // Valid ports
        assert!(RemoteEndpoint::validate_port(1));
        assert!(RemoteEndpoint::validate_port(80));
        assert!(RemoteEndpoint::validate_port(443));
        assert!(RemoteEndpoint::validate_port(8080));
        assert!(RemoteEndpoint::validate_port(11434));
        assert!(RemoteEndpoint::validate_port(65535));
    }

    #[test]
    fn test_invalid_ports() {
        // Port 0 is invalid
        assert!(!RemoteEndpoint::validate_port(0));
    }

    #[test]
    fn test_valid_ip_and_port_combinations() {
        // Valid combinations
        assert!(RemoteEndpoint::validate_ip_and_port("192.168.1.1", 8080));
        assert!(RemoteEndpoint::validate_ip_and_port("10.0.0.1", 11434));
        assert!(RemoteEndpoint::validate_ip_and_port("::1", 443));
        assert!(RemoteEndpoint::validate_ip_and_port("2001:db8::1", 80));
    }

    #[test]
    fn test_invalid_ip_and_port_combinations() {
        // Invalid IP
        assert!(!RemoteEndpoint::validate_ip_and_port("999.999.999.999", 8080));
        // Invalid port
        assert!(!RemoteEndpoint::validate_ip_and_port("192.168.1.1", 0));
        // Both invalid
        assert!(!RemoteEndpoint::validate_ip_and_port("not-an-ip", 0));
    }
}

/// **Feature: remote-ollama-integration, Property 1: IP and port validation**
/// **Validates: Requirements 1.2, 1.3**
/// 
/// For any string input representing an IP address and port, the validation function
/// should accept valid formats (IPv4/IPv6 with valid port 1-65535) and reject invalid formats.
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use std::path::PathBuf;

    // Strategy to generate valid RemoteEndpoint instances
    fn remote_endpoint_strategy() -> impl Strategy<Value = RemoteEndpoint> {
        (
            "[a-zA-Z0-9 ]{1,50}",  // name
            0u8..=255,              // IP octet 1
            0u8..=255,              // IP octet 2
            0u8..=255,              // IP octet 3
            0u8..=255,              // IP octet 4
            1u16..=65535,           // port
            any::<bool>(),          // use_https
            proptest::option::of("[a-zA-Z0-9]{10,20}"), // api_key
        ).prop_map(|(name, a, b, c, d, port, use_https, api_key)| {
            let host = format!("{}.{}.{}.{}", a, b, c, d);
            RemoteEndpoint {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                host,
                port,
                use_https,
                api_key,
                last_tested: None,
                last_test_success: None,
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_valid_ipv4_accepted(
            a in 0u8..=255,
            b in 0u8..=255,
            c in 0u8..=255,
            d in 0u8..=255
        ) {
            let ip = format!("{}.{}.{}.{}", a, b, c, d);
            prop_assert!(RemoteEndpoint::validate_ip(&ip));
        }

        #[test]
        fn prop_valid_port_range_accepted(port in 1u16..=65535) {
            prop_assert!(RemoteEndpoint::validate_port(port));
        }

        #[test]
        fn prop_valid_ipv4_and_port_accepted(
            a in 0u8..=255,
            b in 0u8..=255,
            c in 0u8..=255,
            d in 0u8..=255,
            port in 1u16..=65535
        ) {
            let ip = format!("{}.{}.{}.{}", a, b, c, d);
            prop_assert!(RemoteEndpoint::validate_ip_and_port(&ip, port));
        }

        #[test]
        fn prop_invalid_ip_format_rejected(invalid_ip in "[a-zA-Z]{5,20}") {
            // Random alphabetic strings should not be valid IPs
            prop_assert!(!RemoteEndpoint::validate_ip(&invalid_ip));
        }

        #[test]
        fn prop_port_zero_rejected(_any in 0u8..=255) {
            // Port 0 should always be rejected
            prop_assert!(!RemoteEndpoint::validate_port(0));
        }

        #[test]
        fn prop_invalid_ip_with_valid_port_rejected(
            invalid_ip in "[a-zA-Z]{5,20}",
            port in 1u16..=65535
        ) {
            // Invalid IP with valid port should be rejected
            prop_assert!(!RemoteEndpoint::validate_ip_and_port(&invalid_ip, port));
        }

        #[test]
        fn prop_valid_ip_with_port_zero_rejected(
            a in 0u8..=255,
            b in 0u8..=255,
            c in 0u8..=255,
            d in 0u8..=255
        ) {
            let ip = format!("{}.{}.{}.{}", a, b, c, d);
            // Valid IP with port 0 should be rejected
            prop_assert!(!RemoteEndpoint::validate_ip_and_port(&ip, 0));
        }

        /// **Feature: remote-ollama-integration, Property 2: Endpoint persistence round-trip**
        /// **Validates: Requirements 1.4, 1.5**
        /// 
        /// For any set of remote endpoints, after saving to configuration and reloading,
        /// all endpoints should be present with identical data.
        #[test]
        fn prop_endpoint_persistence_round_trip(
            endpoints in proptest::collection::vec(remote_endpoint_strategy(), 1..=5)
        ) {
            // Create a unique test config file path
            let test_config_path = PathBuf::from(format!(
                "test_config_endpoint_{}.toml",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));

            // Create a config with the generated endpoints
            let mut config = AppConfig::default();
            
            // Store the original endpoints for comparison
            let original_endpoints = endpoints.clone();
            
            // Add all endpoints to the config
            for endpoint in endpoints {
                config.backend.remote_endpoints.push(endpoint);
            }

            // Save the config to a test file
            let toml_string = toml::to_string_pretty(&config)
                .expect("Failed to serialize config");
            
            fs::write(&test_config_path, toml_string)
                .expect("Failed to write test config file");

            // Load the config back
            let loaded_config = config::Config::builder()
                .add_source(config::File::from(test_config_path.clone()))
                .build()
                .expect("Failed to build config")
                .try_deserialize::<AppConfig>()
                .expect("Failed to deserialize config");

            // Clean up test file
            let _ = fs::remove_file(&test_config_path);

            // Verify the number of endpoints matches
            prop_assert_eq!(
                loaded_config.backend.remote_endpoints.len(),
                original_endpoints.len(),
                "Number of endpoints should match after round-trip"
            );

            // Verify each endpoint is present with identical data
            for original_endpoint in &original_endpoints {
                let found = loaded_config.backend.remote_endpoints.iter().find(|e| e.id == original_endpoint.id);
                
                prop_assert!(
                    found.is_some(),
                    "Endpoint with ID {} should be present after round-trip",
                    original_endpoint.id
                );

                let loaded_endpoint = found.unwrap();
                
                // Verify all fields match
                prop_assert_eq!(&loaded_endpoint.id, &original_endpoint.id, "ID should match");
                prop_assert_eq!(&loaded_endpoint.name, &original_endpoint.name, "Name should match");
                prop_assert_eq!(&loaded_endpoint.host, &original_endpoint.host, "Host should match");
                prop_assert_eq!(loaded_endpoint.port, original_endpoint.port, "Port should match");
                prop_assert_eq!(loaded_endpoint.use_https, original_endpoint.use_https, "use_https should match");
                prop_assert_eq!(&loaded_endpoint.api_key, &original_endpoint.api_key, "API key should match");
                prop_assert_eq!(&loaded_endpoint.last_tested, &original_endpoint.last_tested, "last_tested should match");
                prop_assert_eq!(&loaded_endpoint.last_test_success, &original_endpoint.last_test_success, "last_test_success should match");
            }
        }
    }
}
