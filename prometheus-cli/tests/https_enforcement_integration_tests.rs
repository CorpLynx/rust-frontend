use anyhow::Result;
use prometheus_cli::backend::BackendClient;
use prometheus_cli::config::AppConfig;
use prometheus_cli::url_validator::{UrlValidator, UrlValidationError};
use std::time::{Duration, Instant};
use std::fs;
use tempfile::TempDir;

/// Integration tests for HTTPS-only enforcement feature
/// 
/// These tests validate the complete end-to-end URL validation flow,
/// error messages, user experience, performance requirements, and
/// security properties across all integration points.

/// Test complete end-to-end URL validation flow from CLI arguments to backend client creation
/// **Validates: Requirements 1.1, 1.2, 2.1, 2.2, 2.3, 4.1, 4.2, 4.4**
#[tokio::test]
async fn test_end_to_end_url_validation_flow() -> Result<()> {
    // Test 1: Remote HTTPS URL should work end-to-end
    let https_url = "https://api.example.com:8080";
    
    // Validate URL
    assert!(UrlValidator::validate_backend_url(https_url).is_ok());
    
    // Create backend client (should succeed)
    let client = BackendClient::new(https_url.to_string(), 30);
    assert!(client.is_ok());
    
    let client = client.unwrap();
    assert_eq!(client.base_url(), https_url);
    
    // Test 2: Remote HTTP URL should fail at validation step
    let http_url = "http://api.example.com:8080";
    
    // Validate URL (should fail)
    let validation_result = UrlValidator::validate_backend_url(http_url);
    assert!(validation_result.is_err());
    
    // Verify error type and message
    match validation_result.unwrap_err() {
        UrlValidationError::InvalidProtocol { url, suggested } => {
            assert_eq!(url, http_url);
            assert_eq!(suggested, "https://api.example.com:8080");
        }
        _ => panic!("Expected InvalidProtocol error"),
    }
    
    // Backend client creation should also fail
    let client_result = BackendClient::new(http_url.to_string(), 30);
    assert!(client_result.is_err());
    
    // Test 3: Localhost HTTP URL should work end-to-end
    let localhost_url = "http://localhost:11434";
    
    // Validate URL (should succeed)
    assert!(UrlValidator::validate_backend_url(localhost_url).is_ok());
    
    // Create backend client (should succeed)
    let client = BackendClient::new(localhost_url.to_string(), 30);
    assert!(client.is_ok());
    
    let client = client.unwrap();
    assert_eq!(client.base_url(), localhost_url);
    
    // Test 4: Localhost HTTPS URL should also work
    let localhost_https_url = "https://localhost:8080";
    
    // Validate URL (should succeed)
    assert!(UrlValidator::validate_backend_url(localhost_https_url).is_ok());
    
    // Create backend client (should succeed)
    let client = BackendClient::new(localhost_https_url.to_string(), 30);
    assert!(client.is_ok());
    
    Ok(())
}

/// Test error messages and user experience for various invalid URL scenarios
/// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**
#[tokio::test]
async fn test_error_messages_and_user_experience() -> Result<()> {
    // Test 1: Remote HTTP URL error message completeness
    let http_url = "http://remote-server.com:8080/api/v1";
    let validation_result = UrlValidator::validate_backend_url(http_url);
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    let error_message = error.to_string();
    
    // Verify error message contains all required components
    assert!(error_message.contains("Invalid backend URL protocol"));
    assert!(error_message.contains(http_url)); // Original URL
    assert!(error_message.contains("https://remote-server.com:8080/api/v1")); // Suggested URL
    assert!(error_message.contains("HTTPS protocol for remote endpoints")); // Requirement explanation
    assert!(error_message.contains("encrypt your prompts and responses")); // Security explanation
    assert!(error_message.contains("Examples of valid URLs")); // Examples
    assert!(error_message.contains("https://api.example.com:8080"));
    assert!(error_message.contains("https://my-ollama-server.com"));
    assert!(error_message.contains("http://localhost:11434 (localhost only)"));
    
    // Test 2: Invalid format URL error message
    let invalid_url = "not-a-valid-url";
    let validation_result = UrlValidator::validate_backend_url(invalid_url);
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    let error_message = error.to_string();
    
    assert!(error_message.contains("Invalid URL format"));
    assert!(error_message.contains(invalid_url));
    assert!(error_message.contains("Examples of valid URLs"));
    
    // Test 3: Empty URL error message
    let empty_url = "";
    let validation_result = UrlValidator::validate_backend_url(empty_url);
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    let error_message = error.to_string();
    
    assert!(error_message.contains("Backend URL cannot be empty"));
    assert!(error_message.contains("Examples of valid URLs"));
    
    // Test 4: Whitespace-only URL error message
    let whitespace_url = "   \t\n  ";
    let validation_result = UrlValidator::validate_backend_url(whitespace_url);
    
    assert!(validation_result.is_err());
    let error = validation_result.unwrap_err();
    
    match error {
        UrlValidationError::EmptyUrl => {
            // Expected - whitespace-only URLs are treated as empty
        }
        _ => panic!("Expected EmptyUrl error for whitespace-only URL"),
    }
    
    Ok(())
}

/// Test performance requirements for URL validation under various conditions
/// **Validates: Requirements 4.5**
#[tokio::test]
async fn test_validation_performance_requirements() -> Result<()> {
    // Test 1: Single URL validation should complete within 100ms
    let test_urls = vec![
        "https://api.example.com",
        "http://api.example.com", // Will fail validation but should be fast
        "http://localhost:11434",
        "https://very-long-domain-name-that-might-take-longer-to-process.example.com:8080/api/v1/endpoint",
        "invalid-url-format",
        "",
    ];
    
    for url in test_urls {
        let start = Instant::now();
        let _result = UrlValidator::validate_backend_url(url);
        let duration = start.elapsed();
        
        assert!(
            duration < Duration::from_millis(100),
            "URL validation for '{}' took {:?}, which exceeds 100ms requirement",
            url,
            duration
        );
    }
    
    // Test 2: Batch validation performance
    let batch_urls: Vec<String> = (0..100)
        .map(|i| format!("https://api{}.example.com", i))
        .collect();
    
    let start = Instant::now();
    let (valid_urls, invalid_urls) = UrlValidator::filter_valid_urls(batch_urls.clone());
    let duration = start.elapsed();
    
    // Batch validation should complete within reasonable time (10ms per URL max)
    assert!(
        duration < Duration::from_millis(1000),
        "Batch validation of {} URLs took {:?}, which is too slow",
        batch_urls.len(),
        duration
    );
    
    // Verify results are correct
    assert_eq!(valid_urls.len(), 100);
    assert_eq!(invalid_urls.len(), 0);
    
    // Test 3: Performance with mixed valid/invalid URLs
    let mixed_urls = vec![
        "https://api1.example.com".to_string(),
        "http://api2.example.com".to_string(), // Invalid
        "http://localhost:11434".to_string(),   // Valid localhost
        "invalid-url".to_string(),              // Invalid format
        "https://api3.example.com".to_string(),
    ];
    
    let start = Instant::now();
    let (valid_urls, invalid_urls) = UrlValidator::filter_valid_urls(mixed_urls);
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_millis(100));
    assert_eq!(valid_urls.len(), 3); // 2 HTTPS + 1 localhost
    assert_eq!(invalid_urls.len(), 2); // 1 HTTP remote + 1 invalid format
    
    Ok(())
}

/// Test configuration loading and URL validation integration
/// **Validates: Requirements 4.3, 5.1, 5.2, 5.3, 5.4, 5.5**
#[tokio::test]
async fn test_configuration_url_validation_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.toml");
    
    // Test 1: Configuration with mixed valid/invalid URLs
    let test_config = r#"
[app]
window_title = "Test App"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://api.example.com"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = [
    "https://api1.example.com",
    "http://api2.example.com",
    "http://localhost:8080",
    "https://api3.example.com",
    "invalid-url",
    "http://192.168.1.100:8080"
]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;
    
    fs::write(&config_path, test_config)?;
    
    // Load and validate configuration
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(config_path.clone()))
        .build()?
        .try_deserialize::<AppConfig>()?;
    
    // Perform migration and validation
    let mut migrated_config = loaded_config;
    let migration_result = AppConfig::migrate_configuration(&mut migrated_config)?;
    
    // Validate saved URLs
    let invalid_urls = migrated_config.backend.validate_and_filter_saved_urls();
    
    // Verify migration occurred for HTTP URLs
    assert!(migration_result.migration_performed);
    assert_eq!(migrated_config.backend.url, "https://api.example.com"); // Migrated
    assert_eq!(migrated_config.backend.ollama_url, "http://localhost:11434"); // Localhost unchanged
    
    // Verify saved URLs were filtered correctly
    assert!(migrated_config.backend.saved_urls.contains(&"https://api1.example.com".to_string()));
    assert!(migrated_config.backend.saved_urls.contains(&"https://api2.example.com".to_string())); // Migrated
    assert!(migrated_config.backend.saved_urls.contains(&"http://localhost:8080".to_string())); // Localhost preserved
    assert!(migrated_config.backend.saved_urls.contains(&"https://api3.example.com".to_string()));
    
    // Verify invalid URLs were removed
    assert!(!migrated_config.backend.saved_urls.contains(&"invalid-url".to_string()));
    assert!(!migrated_config.backend.saved_urls.contains(&"http://192.168.1.100:8080".to_string()));
    
    // Verify invalid URLs list
    assert!(invalid_urls.contains(&"invalid-url".to_string()));
    
    // Test 2: Configuration with only valid URLs (no migration needed)
    let valid_config = r#"
[app]
window_title = "Valid Config"
window_width = 800.0
window_height = 600.0

[backend]
url = "https://api.example.com"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = [
    "https://api1.example.com",
    "https://api2.example.com",
    "http://localhost:8080"
]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;
    
    let valid_config_path = temp_dir.path().join("valid_config.toml");
    fs::write(&valid_config_path, valid_config)?;
    
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(valid_config_path))
        .build()?
        .try_deserialize::<AppConfig>()?;
    
    let mut config_copy = loaded_config;
    let migration_result = AppConfig::migrate_configuration(&mut config_copy)?;
    
    // No migration should be needed
    assert!(!migration_result.migration_performed);
    assert_eq!(migration_result.migrated_urls.len(), 0);
    
    Ok(())
}

/// Test security properties across all integration points
/// **Validates: Requirements 1.1, 1.2, 2.1, 2.2, 2.3, 5.1, 5.4**
#[tokio::test]
async fn test_security_properties_integration() -> Result<()> {
    // Test 1: Remote HTTP URLs are consistently rejected across all components
    let remote_http_urls = vec![
        "http://api.example.com",
        "http://remote-server.com:8080",
        "http://192.168.1.100:11434",
        "http://10.0.0.1:8080",
        "http://172.16.0.1:11434",
    ];
    
    for url in remote_http_urls {
        // URL validator should reject
        assert!(UrlValidator::validate_backend_url(url).is_err());
        
        // Backend client creation should fail
        assert!(BackendClient::new(url.to_string(), 30).is_err());
        
        // Configuration should not save these URLs
        let mut config = AppConfig::default();
        let add_result = config.backend.add_saved_url(url.to_string());
        assert!(add_result.is_err());
        
        // URL should not be detected as localhost
        assert!(!UrlValidator::is_localhost_url(url));
    }
    
    // Test 2: Remote HTTPS URLs are consistently accepted
    let remote_https_urls = vec![
        "https://api.example.com",
        "https://remote-server.com:8080",
        "https://my-ollama-server.com:11434",
    ];
    
    for url in remote_https_urls {
        // URL validator should accept
        assert!(UrlValidator::validate_backend_url(url).is_ok());
        
        // Backend client creation should succeed
        assert!(BackendClient::new(url.to_string(), 30).is_ok());
        
        // Configuration should save these URLs
        let mut config = AppConfig::default();
        let add_result = config.backend.add_saved_url(url.to_string());
        assert!(add_result.is_ok());
        
        // URL should not be detected as localhost
        assert!(!UrlValidator::is_localhost_url(url));
    }
    
    // Test 3: Localhost URLs (HTTP and HTTPS) are consistently accepted
    let localhost_urls = vec![
        "http://localhost:11434",
        "https://localhost:8080",
        "http://127.0.0.1:11434",
        "https://127.0.0.1:8080",
        "http://[::1]:11434",
        "https://[::1]:8080",
    ];
    
    for url in localhost_urls {
        // URL validator should accept
        assert!(UrlValidator::validate_backend_url(url).is_ok());
        
        // Backend client creation should succeed
        assert!(BackendClient::new(url.to_string(), 30).is_ok());
        
        // URL should be detected as localhost
        assert!(UrlValidator::is_localhost_url(url));
        
        // Localhost URLs should not be saved to configuration (by design)
        let mut config = AppConfig::default();
        let add_result = config.backend.add_saved_url(url.to_string());
        assert!(add_result.is_ok()); // Operation succeeds but URL is not actually saved
        assert!(!config.backend.saved_urls.contains(&url.to_string()));
    }
    
    Ok(())
}

/// Test application startup validation with various URL configurations
/// **Validates: Requirements 4.4**
#[tokio::test]
async fn test_application_startup_validation() -> Result<()> {
    // Test 1: Valid HTTPS URL should allow startup
    let valid_url = "https://api.example.com";
    assert!(UrlValidator::validate_backend_url(valid_url).is_ok());
    
    // Test 2: Invalid HTTP URL should prevent startup
    let invalid_url = "http://api.example.com";
    let validation_result = UrlValidator::validate_backend_url(invalid_url);
    assert!(validation_result.is_err());
    
    // Verify the error would cause proper exit code
    match validation_result.unwrap_err() {
        UrlValidationError::InvalidProtocol { .. } => {
            // This would result in URL_VALIDATION_ERROR exit code
        }
        _ => panic!("Expected InvalidProtocol error"),
    }
    
    // Test 3: Localhost URL should allow startup
    let localhost_url = "http://localhost:11434";
    assert!(UrlValidator::validate_backend_url(localhost_url).is_ok());
    
    // Test 4: Empty URL should prevent startup
    let empty_url = "";
    let validation_result = UrlValidator::validate_backend_url(empty_url);
    assert!(validation_result.is_err());
    
    match validation_result.unwrap_err() {
        UrlValidationError::EmptyUrl => {
            // This would result in URL_VALIDATION_ERROR exit code
        }
        _ => panic!("Expected EmptyUrl error"),
    }
    
    // Test 5: Malformed URL should prevent startup
    let malformed_url = "not-a-url";
    let validation_result = UrlValidator::validate_backend_url(malformed_url);
    assert!(validation_result.is_err());
    
    match validation_result.unwrap_err() {
        UrlValidationError::InvalidFormat { .. } => {
            // This would result in URL_VALIDATION_ERROR exit code
        }
        _ => panic!("Expected InvalidFormat error"),
    }
    
    Ok(())
}

/// Test URL persistence security across configuration operations
/// **Validates: Requirements 5.1, 5.4, 5.5**
#[tokio::test]
async fn test_url_persistence_security() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Test 1: Only HTTPS remote URLs should be persisted
    let mut config = AppConfig::default();
    
    // Try to add various URLs
    let test_urls = vec![
        ("https://api.example.com", true),           // Should be saved
        ("http://api.example.com", false),           // Should be rejected
        ("https://secure-server.com:8080", true),   // Should be saved
        ("http://insecure-server.com:8080", false), // Should be rejected
        ("http://localhost:11434", false),           // Should not be saved (localhost)
        ("https://localhost:8080", false),           // Should not be saved (localhost)
        ("invalid-url", false),                      // Should be rejected
    ];
    
    for (url, should_be_saved) in test_urls {
        let initial_count = config.backend.saved_urls.len();
        let add_result = config.backend.add_saved_url(url.to_string());
        
        if should_be_saved {
            assert!(add_result.is_ok(), "Failed to add valid URL: {}", url);
            assert_eq!(
                config.backend.saved_urls.len(),
                initial_count + 1,
                "Valid URL was not saved: {}",
                url
            );
            assert!(
                config.backend.saved_urls.contains(&url.to_string()),
                "Valid URL not found in saved URLs: {}",
                url
            );
        } else {
            if url.contains("localhost") || url.contains("127.0.0.1") {
                // Localhost URLs succeed but are not saved
                assert!(add_result.is_ok(), "Localhost URL should be accepted: {}", url);
                assert_eq!(
                    config.backend.saved_urls.len(),
                    initial_count,
                    "Localhost URL should not be saved: {}",
                    url
                );
            } else {
                // Invalid URLs should be rejected
                assert!(add_result.is_err(), "Invalid URL should be rejected: {}", url);
                assert_eq!(
                    config.backend.saved_urls.len(),
                    initial_count,
                    "Invalid URL should not change saved URLs count: {}",
                    url
                );
            }
        }
    }
    
    // Test 2: Configuration round-trip should preserve only valid URLs
    let config_path = temp_dir.path().join("persistence_test.toml");
    
    // Save configuration
    let toml_string = toml::to_string_pretty(&config)?;
    fs::write(&config_path, toml_string)?;
    
    // Load configuration back
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(config_path))
        .build()?
        .try_deserialize::<AppConfig>()?;
    
    // Verify only HTTPS remote URLs were persisted
    for url in &loaded_config.backend.saved_urls {
        assert!(url.starts_with("https://"), "Non-HTTPS URL found in persisted config: {}", url);
        assert!(!UrlValidator::is_localhost_url(url), "Localhost URL found in persisted config: {}", url);
        assert!(UrlValidator::validate_backend_url(url).is_ok(), "Invalid URL found in persisted config: {}", url);
    }
    
    Ok(())
}

/// Test comprehensive error propagation through the application stack
/// **Validates: Requirements 3.5, 7.1, 7.2**
#[tokio::test]
async fn test_error_propagation_integration() -> Result<()> {
    // Test 1: URL validation errors should propagate correctly
    let invalid_urls = vec![
        ("http://api.example.com", "InvalidProtocol"),
        ("invalid-url", "InvalidFormat"),
        ("", "EmptyUrl"),
        ("   ", "EmptyUrl"),
    ];
    
    for (url, expected_error_type) in invalid_urls {
        // Test validation error
        let validation_result = UrlValidator::validate_backend_url(url);
        assert!(validation_result.is_err());
        
        let error = validation_result.unwrap_err();
        match expected_error_type {
            "InvalidProtocol" => {
                assert!(matches!(error, UrlValidationError::InvalidProtocol { .. }));
            }
            "InvalidFormat" => {
                assert!(matches!(error, UrlValidationError::InvalidFormat { .. }));
            }
            "EmptyUrl" => {
                assert!(matches!(error, UrlValidationError::EmptyUrl));
            }
            _ => panic!("Unexpected error type: {}", expected_error_type),
        }
        
        // Test backend client creation error
        let client_result = BackendClient::new(url.to_string(), 30);
        assert!(client_result.is_err());
        
        // Verify error message contains validation information
        let error_message = match client_result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("Expected error but got success"),
        };
        
        if expected_error_type == "InvalidProtocol" {
            assert!(error_message.contains("Invalid backend URL protocol"));
        } else if expected_error_type == "InvalidFormat" {
            assert!(error_message.contains("Invalid URL format"));
        } else if expected_error_type == "EmptyUrl" {
            assert!(error_message.contains("Backend URL cannot be empty"));
        }
    }
    
    Ok(())
}

/// Test edge cases and boundary conditions in URL validation
/// **Validates: Requirements 1.1, 1.2, 2.1, 2.2, 2.3**
#[tokio::test]
async fn test_edge_cases_and_boundary_conditions() -> Result<()> {
    // Test 1: URLs with various port numbers
    let port_test_cases = vec![
        ("https://api.example.com:1", true),      // Minimum port
        ("https://api.example.com:65535", true),  // Maximum port
        ("http://localhost:1", true),             // Localhost with minimum port
        ("http://localhost:65535", true),         // Localhost with maximum port
        ("http://api.example.com:80", false),     // Standard HTTP port (still invalid for remote)
        ("https://api.example.com:443", true),    // Standard HTTPS port
    ];
    
    for (url, should_be_valid) in port_test_cases {
        let result = UrlValidator::validate_backend_url(url);
        if should_be_valid {
            assert!(result.is_ok(), "URL should be valid: {}", url);
        } else {
            assert!(result.is_err(), "URL should be invalid: {}", url);
        }
    }
    
    // Test 2: URLs with paths and query parameters
    let path_test_cases = vec![
        ("https://api.example.com/v1/chat", true),
        ("https://api.example.com/api/generate?model=llama2", true),
        ("http://api.example.com/v1/chat", false), // HTTP remote still invalid
        ("http://localhost:11434/api/generate?stream=true", true), // Localhost OK
    ];
    
    for (url, should_be_valid) in path_test_cases {
        let result = UrlValidator::validate_backend_url(url);
        if should_be_valid {
            assert!(result.is_ok(), "URL should be valid: {}", url);
        } else {
            assert!(result.is_err(), "URL should be invalid: {}", url);
        }
    }
    
    // Test 3: Case sensitivity and normalization
    let case_test_cases = vec![
        ("HTTPS://API.EXAMPLE.COM", true),        // Uppercase scheme and host
        ("HTTP://LOCALHOST:11434", true),         // Uppercase localhost
        ("https://API.example.COM:8080", true),   // Mixed case host
    ];
    
    for (url, should_be_valid) in case_test_cases {
        let result = UrlValidator::validate_backend_url(url);
        if should_be_valid {
            assert!(result.is_ok(), "URL should be valid: {}", url);
        } else {
            assert!(result.is_err(), "URL should be invalid: {}", url);
        }
    }
    
    // Test 4: IPv6 addresses
    let ipv6_test_cases = vec![
        ("http://[::1]:11434", true),             // IPv6 localhost
        ("https://[::1]:8080", true),             // IPv6 localhost HTTPS
        ("http://[2001:db8::1]:8080", false),     // IPv6 remote HTTP (invalid)
        ("https://[2001:db8::1]:8080", true),     // IPv6 remote HTTPS (valid)
    ];
    
    for (url, should_be_valid) in ipv6_test_cases {
        let result = UrlValidator::validate_backend_url(url);
        if should_be_valid {
            assert!(result.is_ok(), "URL should be valid: {}", url);
        } else {
            assert!(result.is_err(), "URL should be invalid: {}", url);
        }
    }
    
    Ok(())
}

/// Test concurrent URL validation operations
/// **Validates: Requirements 4.5 (performance under concurrent load)**
#[tokio::test]
async fn test_concurrent_url_validation() -> Result<()> {
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    // Test concurrent validation of different URLs
    let test_urls = Arc::new(vec![
        "https://api1.example.com",
        "https://api2.example.com", 
        "http://api3.example.com", // Invalid
        "http://localhost:11434",
        "https://api4.example.com",
        "invalid-url", // Invalid
        "https://api5.example.com",
        "http://127.0.0.1:8080",
    ]);
    
    let mut join_set = JoinSet::new();
    
    // Spawn concurrent validation tasks
    for i in 0..10 {
        let urls = Arc::clone(&test_urls);
        join_set.spawn(async move {
            let start = Instant::now();
            let mut results = Vec::new();
            
            for url in urls.iter() {
                let result = UrlValidator::validate_backend_url(url);
                results.push((url.to_string(), result.is_ok()));
            }
            
            let duration = start.elapsed();
            (i, results, duration)
        });
    }
    
    // Collect results
    let mut all_results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        let (task_id, results, duration) = result?;
        
        // Each task should complete within reasonable time
        assert!(
            duration < Duration::from_millis(500),
            "Task {} took too long: {:?}",
            task_id,
            duration
        );
        
        all_results.push(results);
    }
    
    // Verify all tasks produced consistent results
    let expected_results = vec![
        ("https://api1.example.com", true),
        ("https://api2.example.com", true),
        ("http://api3.example.com", false),
        ("http://localhost:11434", true),
        ("https://api4.example.com", true),
        ("invalid-url", false),
        ("https://api5.example.com", true),
        ("http://127.0.0.1:8080", true),
    ];
    
    for results in all_results {
        for (i, (url, is_valid)) in results.iter().enumerate() {
            assert_eq!(
                *is_valid,
                expected_results[i].1,
                "Inconsistent result for URL: {}",
                url
            );
        }
    }
    
    Ok(())
}