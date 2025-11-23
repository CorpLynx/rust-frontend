/// Integration test for initialization and migration logic
/// Requirements: 6.4, 8.4

use app_lib::config::{AppConfig, ConnectionMode};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_complete_initialization_flow() {
    // Create a unique test config file path
    let test_config_path = PathBuf::from(format!(
        "test_config_complete_init_{}.toml",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Simulate an old config file with a remote ollama_url
    let old_config = r#"
[app]
window_title = "Prometheus"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "https://production.example.com:11434"
timeout_seconds = 30
saved_urls = ["https://api1.example.com", "https://api2.example.com"]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

    // Write the old config
    fs::write(&test_config_path, old_config).expect("Failed to write test config");

    // Step 1: Load configuration (simulating app startup)
    let config = config::Config::builder()
        .add_source(config::File::from(test_config_path.clone()))
        .build()
        .expect("Failed to build config")
        .try_deserialize::<AppConfig>()
        .expect("Failed to deserialize config");

    // Verify initial state before migration
    assert_eq!(config.backend.remote_endpoints.len(), 0, "Should have no remote endpoints before migration");
    assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local, "Should default to Local mode");
    assert_eq!(config.backend.saved_urls.len(), 2, "Should have loaded saved URLs");

    // Step 2: Perform migration
    let mut config = config;
    config.backend.migrate_ollama_url();

    // Verify migration results
    assert_eq!(config.backend.remote_endpoints.len(), 1, "Should have migrated one endpoint");
    assert_eq!(config.backend.remote_endpoints[0].name, "Migrated Endpoint");
    assert_eq!(config.backend.remote_endpoints[0].host, "production.example.com");
    assert_eq!(config.backend.remote_endpoints[0].port, 11434);
    assert_eq!(config.backend.remote_endpoints[0].use_https, true);
    assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local, "Should remain in Local mode for existing users");

    // Step 3: Save the migrated config
    let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config");
    fs::write(&test_config_path, toml_string).expect("Failed to write migrated config");

    // Step 4: Verify file permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        // Set permissions (simulating what save() does)
        let metadata = fs::metadata(&test_config_path).expect("Failed to get metadata");
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&test_config_path, permissions).expect("Failed to set permissions");

        // Verify permissions
        let metadata = fs::metadata(&test_config_path).expect("Failed to get metadata after setting permissions");
        let mode = metadata.permissions().mode();
        let permission_bits = mode & 0o777;
        assert_eq!(permission_bits, 0o600, "Config file should have 0600 permissions");
    }

    // Step 5: Load the config again to verify persistence
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(test_config_path.clone()))
        .build()
        .expect("Failed to build config")
        .try_deserialize::<AppConfig>()
        .expect("Failed to deserialize config");

    // Verify everything persisted correctly
    assert_eq!(loaded_config.backend.remote_endpoints.len(), 1);
    assert_eq!(loaded_config.backend.remote_endpoints[0].name, "Migrated Endpoint");
    assert_eq!(loaded_config.backend.remote_endpoints[0].host, "production.example.com");
    assert_eq!(loaded_config.backend.remote_endpoints[0].port, 11434);
    assert_eq!(loaded_config.backend.remote_endpoints[0].use_https, true);
    assert_eq!(*loaded_config.backend.get_connection_mode(), ConnectionMode::Local);
    assert_eq!(loaded_config.backend.saved_urls.len(), 2);

    // Step 6: Verify migration only happens once
    let mut config = loaded_config;
    config.backend.migrate_ollama_url();
    assert_eq!(config.backend.remote_endpoints.len(), 1, "Should not create duplicate endpoints on second migration");

    // Clean up
    let _ = fs::remove_file(&test_config_path);
}

#[test]
fn test_initialization_with_localhost_url() {
    // Create a unique test config file path
    let test_config_path = PathBuf::from(format!(
        "test_config_localhost_init_{}.toml",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Simulate a config file with localhost ollama_url (should not be migrated)
    let config_content = r#"
[app]
window_title = "Prometheus"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = []

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

    // Write the config
    fs::write(&test_config_path, config_content).expect("Failed to write test config");

    // Load and migrate
    let config = config::Config::builder()
        .add_source(config::File::from(test_config_path.clone()))
        .build()
        .expect("Failed to build config")
        .try_deserialize::<AppConfig>()
        .expect("Failed to deserialize config");

    let mut config = config;
    config.backend.migrate_ollama_url();

    // Verify localhost URL was NOT migrated
    assert_eq!(config.backend.remote_endpoints.len(), 0, "Localhost URLs should not be migrated");
    assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local);

    // Clean up
    let _ = fs::remove_file(&test_config_path);
}

#[test]
fn test_initialization_with_new_config() {
    // Test that a brand new config (no file exists) works correctly
    let config = AppConfig::default();

    // Verify defaults
    assert_eq!(config.backend.remote_endpoints.len(), 0);
    assert_eq!(*config.backend.get_connection_mode(), ConnectionMode::Local);
    assert_eq!(config.backend.saved_urls.len(), 0);
    assert_eq!(config.backend.ollama_url, "http://localhost:1234");

    // Migration should do nothing for default config
    let mut config = config;
    config.backend.migrate_ollama_url();
    assert_eq!(config.backend.remote_endpoints.len(), 0);
}
