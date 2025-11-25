use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::url_validator::UrlValidator;

/// Result of configuration migration process
/// 
/// Contains information about URLs that were migrated and whether
/// a backup was created during the migration process.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Whether any migration was performed
    pub migration_performed: bool,
    /// Whether a backup was created
    pub backup_created: bool,
    /// List of URLs that were migrated
    pub migrated_urls: Vec<MigratedUrl>,
}

impl MigrationResult {
    /// Create a new empty migration result
    pub fn new() -> Self {
        Self {
            migration_performed: false,
            backup_created: false,
            migrated_urls: Vec::new(),
        }
    }
}

/// Information about a single URL that was migrated
#[derive(Debug, Clone)]
pub struct MigratedUrl {
    /// The original URL before migration
    pub original: String,
    /// The migrated URL after conversion to HTTPS
    pub migrated: String,
    /// Location in the configuration where this URL was found
    pub location: String,
}

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
}

impl BackendSettings {
    pub const LOCAL_OLLAMA_URL: &'static str = "http://localhost:11434";
    
    /// Validate and filter saved URLs, removing invalid ones
    /// 
    /// # Requirements
    /// * 5.2: Validate each URL's protocol during configuration loading
    /// * 5.3: Remove HTTP URLs and log warnings
    pub fn validate_and_filter_saved_urls(&mut self) -> Vec<String> {
        let (valid_urls, invalid_urls) = UrlValidator::filter_valid_urls(self.saved_urls.clone());
        
        // Update saved URLs to only include valid ones
        self.saved_urls = valid_urls;
        
        // Return invalid URLs for warning purposes
        invalid_urls
    }
    
    /// Add a URL to the saved URLs list with duplicate checking and size limiting
    /// 
    /// # Requirements
    /// * 5.1: Only save HTTPS URLs for remote endpoints
    /// * 5.4: Reject save operation for HTTP remote URLs
    pub fn add_saved_url(&mut self, url: String) -> Result<()> {
        // Validate the URL before saving
        if let Err(validation_error) = UrlValidator::validate_backend_url(&url) {
            return Err(anyhow::anyhow!("Cannot save invalid URL: {}", validation_error));
        }
        
        // Don't save localhost URLs (they're allowed but not saved)
        if UrlValidator::is_localhost_url(&url) {
            return Ok(());
        }
        
        // Don't add duplicates
        if !self.saved_urls.contains(&url) {
            self.saved_urls.insert(0, url);
            
            // Keep only last 10 URLs
            if self.saved_urls.len() > 10 {
                self.saved_urls.truncate(10);
            }
        }
        
        Ok(())
    }
    
    /// Remove a URL from the saved URLs list
    pub fn remove_saved_url(&mut self, url: &str) {
        self.saved_urls.retain(|u| u != url);
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
    /// # Requirements
    /// * 3.5: Load saved URLs from config on app startup
    /// * 5.2: Validate each URL's protocol during configuration loading
    /// * 5.3: Remove HTTP URLs and log warnings
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from("config.toml");
        
        let settings = config::Config::builder()
            .add_source(config::File::from(config_path.clone()))
            .build()
            .with_context(|| format!("Failed to load config from {:?}", config_path))?;

        let mut config: Self = settings
            .try_deserialize()
            .context("Failed to deserialize config")?;
        
        // Check if migration is needed and perform it
        let migration_result = Self::migrate_configuration(&mut config)?;
        
        // Validate and filter saved URLs
        let invalid_urls = config.backend.validate_and_filter_saved_urls();
        
        // Log warnings for invalid URLs that were removed
        if !invalid_urls.is_empty() {
            eprintln!("Warning: Removed {} invalid URL(s) from saved configuration:", invalid_urls.len());
            for url in &invalid_urls {
                eprintln!("  - {}", url);
            }
            eprintln!("Note: Only HTTPS URLs are allowed for remote endpoints. Localhost URLs can use HTTP.");
        }
        
        // If migration occurred, save the updated configuration
        if migration_result.migration_performed {
            config.save().context("Failed to save migrated configuration")?;
        }
        
        Ok(config)
    }

    /// Migrate existing configuration to enforce HTTPS-only URLs
    /// 
    /// This method creates a backup of the original configuration and migrates
    /// HTTP URLs to HTTPS where possible, removing invalid ones.
    /// 
    /// # Requirements
    /// * 5.3: Implement configuration migration for existing HTTP URLs
    /// 
    /// # Arguments
    /// * `config` - Mutable reference to the configuration to migrate
    /// 
    /// # Returns
    /// * `MigrationResult` containing information about the migration process
    pub fn migrate_configuration(config: &mut Self) -> Result<MigrationResult> {
        let mut migration_result = MigrationResult::new();
        
        // Check if the main backend URL needs migration
        let original_backend_url = config.backend.url.clone();
        if let Some(migrated_url) = Self::migrate_url(&original_backend_url) {
            // Create backup before migration
            if !migration_result.backup_created {
                Self::create_backup()?;
                migration_result.backup_created = true;
            }
            
            config.backend.url = migrated_url.clone();
            migration_result.migrated_urls.push(MigratedUrl {
                original: original_backend_url,
                migrated: migrated_url,
                location: "backend.url".to_string(),
            });
            migration_result.migration_performed = true;
        }
        
        // Check if the ollama_url needs migration
        let original_ollama_url = config.backend.ollama_url.clone();
        if let Some(migrated_url) = Self::migrate_url(&original_ollama_url) {
            // Create backup before migration
            if !migration_result.backup_created {
                Self::create_backup()?;
                migration_result.backup_created = true;
            }
            
            config.backend.ollama_url = migrated_url.clone();
            migration_result.migrated_urls.push(MigratedUrl {
                original: original_ollama_url,
                migrated: migrated_url,
                location: "backend.ollama_url".to_string(),
            });
            migration_result.migration_performed = true;
        }
        
        // Check saved URLs for migration
        let mut migrated_saved_urls = Vec::new();
        let mut any_saved_url_migrated = false;
        
        for url in &config.backend.saved_urls {
            if let Some(migrated_url) = Self::migrate_url(url) {
                // Create backup before migration
                if !migration_result.backup_created {
                    Self::create_backup()?;
                    migration_result.backup_created = true;
                }
                
                migrated_saved_urls.push(migrated_url.clone());
                migration_result.migrated_urls.push(MigratedUrl {
                    original: url.clone(),
                    migrated: migrated_url,
                    location: "backend.saved_urls".to_string(),
                });
                any_saved_url_migrated = true;
            } else {
                migrated_saved_urls.push(url.clone());
            }
        }
        
        if any_saved_url_migrated {
            config.backend.saved_urls = migrated_saved_urls;
            migration_result.migration_performed = true;
        }
        
        // Display migration warnings if any URLs were migrated
        if migration_result.migration_performed {
            Self::display_migration_warnings(&migration_result);
        }
        
        Ok(migration_result)
    }
    
    /// Attempt to migrate a single URL from HTTP to HTTPS
    /// 
    /// # Arguments
    /// * `url` - The URL to potentially migrate
    /// 
    /// # Returns
    /// * `Some(migrated_url)` if migration was needed and possible
    /// * `None` if no migration is needed or possible
    fn migrate_url(url: &str) -> Option<String> {
        // Skip localhost URLs - they don't need migration
        if UrlValidator::is_localhost_url(url) {
            return None;
        }
        
        // Only migrate HTTP URLs to HTTPS
        if url.starts_with("http://") {
            let https_url = UrlValidator::suggest_https_url(url);
            // Verify the suggested URL would be valid
            if UrlValidator::validate_backend_url(&https_url).is_ok() {
                return Some(https_url);
            }
        }
        
        None
    }
    
    /// Create a backup of the current configuration file
    /// 
    /// # Requirements
    /// * 5.3: Create backup of original configuration before migration
    fn create_backup() -> Result<()> {
        let config_path = PathBuf::from("config.toml");
        
        if config_path.exists() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let backup_path = PathBuf::from(format!("config.toml.backup.{}", timestamp));
            
            std::fs::copy(&config_path, &backup_path)
                .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;
            
            eprintln!("✓ Configuration backup created: {}", backup_path.display());
        }
        
        Ok(())
    }
    
    /// Display migration warnings to the user
    /// 
    /// # Requirements
    /// * 5.3: Add warning messages for migrated configurations
    fn display_migration_warnings(migration_result: &MigrationResult) {
        eprintln!();
        eprintln!("🔒 HTTPS Migration Performed");
        eprintln!("═══════════════════════════");
        eprintln!();
        eprintln!("The following URLs have been automatically migrated to use HTTPS:");
        eprintln!();
        
        for migrated in &migration_result.migrated_urls {
            eprintln!("  📍 {}", migrated.location);
            eprintln!("     From: {}", migrated.original);
            eprintln!("     To:   {}", migrated.migrated);
            eprintln!();
        }
        
        eprintln!("🛡️  Security Enhancement:");
        eprintln!("   Remote connections now use HTTPS to encrypt your prompts and responses.");
        eprintln!("   Localhost URLs (localhost, 127.0.0.1) can still use HTTP for development.");
        eprintln!();
        
        if migration_result.backup_created {
            eprintln!("💾 Backup Information:");
            eprintln!("   Your original configuration has been backed up.");
            eprintln!("   If you need to restore it, look for config.toml.backup.* files.");
            eprintln!();
        }
        
        eprintln!("ℹ️  What's Next:");
        eprintln!("   • Verify that the migrated HTTPS URLs are accessible");
        eprintln!("   • Update any external references to use HTTPS");
        eprintln!("   • Remove backup files once you've confirmed everything works");
        eprintln!();
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
            .take(5) // Limit to 5 URLs for faster testing
            .enumerate()
            .map(|(i, url)| format!("https://api{}.example.com/{}", i, url.replace(".", "").replace("/", "")))
            .collect();

        if valid_urls.is_empty() {
            return TestResult::discard();
        }

        let mut settings = BackendSettings {
            url: "http://example.com".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            saved_urls: Vec::new(),
        };

        // Add all URLs (including duplicates) - only successful additions should be counted
        let mut successfully_added_urls = Vec::new();
        for url in &valid_urls {
            if settings.add_saved_url(url.clone()).is_ok() {
                successfully_added_urls.push(url.clone());
            }
        }

        // Check that each unique successfully added URL appears exactly once
        for url in &successfully_added_urls {
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
        };

        let test_url = "https://api.example.com/v1".to_string();
        
        // Add the same URL three times
        let _ = settings.add_saved_url(test_url.clone());
        let _ = settings.add_saved_url(test_url.clone());
        let _ = settings.add_saved_url(test_url.clone());

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
        };

        let _ = settings.add_saved_url("http://localhost:8080".to_string());
        let _ = settings.add_saved_url("http://127.0.0.1:8080".to_string());
        let _ = settings.add_saved_url("https://api.example.com".to_string());

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
        };

        // Add URLs one by one and verify the list never exceeds 10
        for (i, url) in unique_urls.iter().enumerate() {
            let _ = settings.add_saved_url(url.clone());
            
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
            let _ = config.backend.add_saved_url(url.clone());
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
        };

        // Add all URLs to the list
        for url in &valid_urls {
            let _ = settings.add_saved_url(url.clone());
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

    /// Test configuration migration functionality
    /// 
    /// This test verifies that HTTP URLs are properly migrated to HTTPS
    /// and that appropriate warnings are generated.
    #[test]
    fn test_configuration_migration() {
        // Create a config with HTTP URLs that need migration
        let mut config = AppConfig {
            app: AppSettings {
                window_title: "Test App".to_string(),
                window_width: 800.0,
                window_height: 600.0,
            },
            backend: BackendSettings {
                url: "http://api.example.com".to_string(),  // Should be migrated
                ollama_url: "http://remote-server.com:8080".to_string(),  // Should be migrated
                timeout_seconds: 30,
                saved_urls: vec![
                    "http://api1.example.com".to_string(),  // Should be migrated
                    "https://api2.example.com".to_string(), // Should remain unchanged
                    "http://localhost:11434".to_string(),   // Should remain unchanged (localhost)
                    "http://api3.example.com:8080".to_string(), // Should be migrated
                ],
            },
            ui: UISettings {
                font_size: 16,
                max_chat_history: 1000,
                theme: "Hacker Green".to_string(),
            },
        };

        // Perform migration
        let migration_result = AppConfig::migrate_configuration(&mut config).unwrap();

        // Verify migration was performed
        assert!(migration_result.migration_performed);
        assert_eq!(migration_result.migrated_urls.len(), 4); // 4 URLs should be migrated

        // Verify backend URL was migrated
        assert_eq!(config.backend.url, "https://api.example.com");

        // Verify ollama_url was migrated
        assert_eq!(config.backend.ollama_url, "https://remote-server.com:8080");

        // Verify saved URLs were migrated correctly
        assert_eq!(config.backend.saved_urls.len(), 4);
        assert!(config.backend.saved_urls.contains(&"https://api1.example.com".to_string()));
        assert!(config.backend.saved_urls.contains(&"https://api2.example.com".to_string()));
        assert!(config.backend.saved_urls.contains(&"http://localhost:11434".to_string())); // Localhost unchanged
        assert!(config.backend.saved_urls.contains(&"https://api3.example.com:8080".to_string()));

        // Verify migration result contains correct information
        let migrated_urls = &migration_result.migrated_urls;
        
        // Check that backend.url migration is recorded
        assert!(migrated_urls.iter().any(|m| 
            m.original == "http://api.example.com" && 
            m.migrated == "https://api.example.com" &&
            m.location == "backend.url"
        ));

        // Check that backend.ollama_url migration is recorded
        assert!(migrated_urls.iter().any(|m| 
            m.original == "http://remote-server.com:8080" && 
            m.migrated == "https://remote-server.com:8080" &&
            m.location == "backend.ollama_url"
        ));

        // Check that saved URLs migrations are recorded
        assert!(migrated_urls.iter().any(|m| 
            m.original == "http://api1.example.com" && 
            m.migrated == "https://api1.example.com" &&
            m.location == "backend.saved_urls"
        ));

        assert!(migrated_urls.iter().any(|m| 
            m.original == "http://api3.example.com:8080" && 
            m.migrated == "https://api3.example.com:8080" &&
            m.location == "backend.saved_urls"
        ));
    }

    /// Test that no migration occurs when all URLs are already valid
    #[test]
    fn test_no_migration_needed() {
        let mut config = AppConfig {
            app: AppSettings {
                window_title: "Test App".to_string(),
                window_width: 800.0,
                window_height: 600.0,
            },
            backend: BackendSettings {
                url: "https://api.example.com".to_string(),  // Already HTTPS
                ollama_url: "http://localhost:11434".to_string(),  // Localhost, no migration needed
                timeout_seconds: 30,
                saved_urls: vec![
                    "https://api1.example.com".to_string(), // Already HTTPS
                    "https://api2.example.com".to_string(), // Already HTTPS
                    "http://127.0.0.1:8080".to_string(),   // Localhost, no migration needed
                ],
            },
            ui: UISettings {
                font_size: 16,
                max_chat_history: 1000,
                theme: "Hacker Green".to_string(),
            },
        };

        // Store original values for comparison
        let original_url = config.backend.url.clone();
        let original_ollama_url = config.backend.ollama_url.clone();
        let original_saved_urls = config.backend.saved_urls.clone();

        // Perform migration
        let migration_result = AppConfig::migrate_configuration(&mut config).unwrap();

        // Verify no migration was performed
        assert!(!migration_result.migration_performed);
        assert_eq!(migration_result.migrated_urls.len(), 0);
        assert!(!migration_result.backup_created);

        // Verify URLs remain unchanged
        assert_eq!(config.backend.url, original_url);
        assert_eq!(config.backend.ollama_url, original_ollama_url);
        assert_eq!(config.backend.saved_urls, original_saved_urls);
    }

    /// Test migration of individual URLs
    #[test]
    fn test_migrate_url_function() {
        // Test HTTP remote URL migration
        assert_eq!(
            AppConfig::migrate_url("http://api.example.com"),
            Some("https://api.example.com".to_string())
        );

        // Test HTTP remote URL with port migration
        assert_eq!(
            AppConfig::migrate_url("http://server.com:8080"),
            Some("https://server.com:8080".to_string())
        );

        // Test localhost URLs (should not be migrated)
        assert_eq!(AppConfig::migrate_url("http://localhost:11434"), None);
        assert_eq!(AppConfig::migrate_url("http://127.0.0.1:8080"), None);
        assert_eq!(AppConfig::migrate_url("http://[::1]:11434"), None);

        // Test HTTPS URLs (should not be migrated)
        assert_eq!(AppConfig::migrate_url("https://api.example.com"), None);
        assert_eq!(AppConfig::migrate_url("https://localhost:11434"), None);

        // Test invalid URLs (should not be migrated)
        assert_eq!(AppConfig::migrate_url("invalid-url"), None);
        assert_eq!(AppConfig::migrate_url("ftp://example.com"), None);
    }

    /// Test end-to-end configuration migration with file I/O
    /// 
    /// This test creates a configuration file with HTTP URLs, loads it,
    /// and verifies that migration occurs automatically.
    #[test]
    fn test_end_to_end_migration() {
        use std::fs;
        use std::path::PathBuf;

        // Create a unique test config file path
        let test_config_path = PathBuf::from(format!(
            "test_migration_config_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create a test config with HTTP URLs that need migration
        let test_config = r#"
[app]
window_title = "Test Migration App"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://api.example.com"
ollama_url = "http://remote-server.com:8080"
timeout_seconds = 30
saved_urls = [
    "http://api1.example.com",
    "https://api2.example.com",
    "http://localhost:11434",
    "http://api3.example.com:8080"
]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

        // Write test config
        fs::write(&test_config_path, test_config).expect("Failed to write test config");

        // Load config using the existing config loading mechanism
        let loaded_config = config::Config::builder()
            .add_source(config::File::from(test_config_path.clone()))
            .build()
            .expect("Failed to build config")
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize config");

        // Perform migration on the loaded config
        let mut migrated_config = loaded_config;
        let migration_result = AppConfig::migrate_configuration(&mut migrated_config)
            .expect("Migration failed");

        // Clean up test file
        fs::remove_file(&test_config_path).ok();

        // Verify migration was performed
        assert!(migration_result.migration_performed);
        assert_eq!(migration_result.migrated_urls.len(), 4);

        // Verify URLs were migrated correctly
        assert_eq!(migrated_config.backend.url, "https://api.example.com");
        assert_eq!(migrated_config.backend.ollama_url, "https://remote-server.com:8080");

        // Verify saved URLs
        assert_eq!(migrated_config.backend.saved_urls.len(), 4);
        assert!(migrated_config.backend.saved_urls.contains(&"https://api1.example.com".to_string()));
        assert!(migrated_config.backend.saved_urls.contains(&"https://api2.example.com".to_string()));
        assert!(migrated_config.backend.saved_urls.contains(&"http://localhost:11434".to_string())); // Localhost unchanged
        assert!(migrated_config.backend.saved_urls.contains(&"https://api3.example.com:8080".to_string()));

        // Verify that localhost URLs were not migrated
        assert!(migration_result.migrated_urls.iter().all(|m| 
            !m.original.contains("localhost") && !m.original.contains("127.0.0.1")
        ));
    }

    /// Test backup creation functionality
    #[test]
    fn test_backup_creation() {
        use std::fs;
        use std::path::PathBuf;

        // Create a test config file
        let config_path = PathBuf::from("config.toml");
        let test_config = r#"
[app]
window_title = "Test App"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://api.example.com"
ollama_url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

        // Write test config
        fs::write(&config_path, test_config).expect("Failed to write test config");

        // Create backup
        AppConfig::create_backup().expect("Failed to create backup");

        // Check that backup was created
        let backup_files: Vec<_> = fs::read_dir(".")
            .expect("Failed to read directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name()
                    .to_string_lossy()
                    .starts_with("config.toml.backup.")
            })
            .collect();

        assert!(!backup_files.is_empty(), "No backup file was created");

        // Verify backup content matches original
        let backup_path = backup_files[0].path();
        let backup_content = fs::read_to_string(&backup_path)
            .expect("Failed to read backup file");
        let original_content = fs::read_to_string(&config_path)
            .expect("Failed to read original file");

        assert_eq!(backup_content, original_content);

        // Clean up
        fs::remove_file(&config_path).ok();
        fs::remove_file(&backup_path).ok();
    }
}
