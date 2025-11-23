use app_lib::config::{AppConfig, ConnectionMode, RemoteEndpoint};
use std::sync::{Arc, RwLock};

/// Test that set_connection_mode and get_connection_mode work correctly
#[test]
fn test_connection_mode_commands() {
    let config = Arc::new(RwLock::new(AppConfig::default()));
    
    // Initially should be Local mode
    {
        let cfg = config.read().unwrap();
        assert_eq!(*cfg.backend.get_connection_mode(), ConnectionMode::Local);
    }
    
    // Set to Remote mode
    {
        let mut cfg = config.write().unwrap();
        cfg.backend.set_connection_mode(ConnectionMode::Remote);
    }
    
    // Verify it changed
    {
        let cfg = config.read().unwrap();
        assert_eq!(*cfg.backend.get_connection_mode(), ConnectionMode::Remote);
    }
    
    // Set back to Local mode
    {
        let mut cfg = config.write().unwrap();
        cfg.backend.set_connection_mode(ConnectionMode::Local);
    }
    
    // Verify it changed back
    {
        let cfg = config.read().unwrap();
        assert_eq!(*cfg.backend.get_connection_mode(), ConnectionMode::Local);
    }
}

/// Test that set_active_remote_endpoint and get_active_endpoint work correctly
#[test]
fn test_active_endpoint_commands() {
    let mut config = AppConfig::default();
    
    // Add a remote endpoint
    let endpoint = RemoteEndpoint::new(
        "Test Server".to_string(),
        "192.168.1.100".to_string(),
        11434,
        false,
        None,
    ).unwrap();
    
    let endpoint_id = config.backend.add_remote_endpoint(endpoint).unwrap();
    
    // Set it as active
    config.backend.set_active_remote_endpoint(&endpoint_id).unwrap();
    
    // Switch to remote mode
    config.backend.set_connection_mode(ConnectionMode::Remote);
    
    // Get active endpoint URL
    let active_url = config.backend.get_active_endpoint_url().unwrap();
    assert_eq!(active_url, "http://192.168.1.100:11434");
    
    // Switch back to local mode
    config.backend.set_connection_mode(ConnectionMode::Local);
    
    // Get active endpoint URL (should be localhost now)
    let active_url = config.backend.get_active_endpoint_url().unwrap();
    assert_eq!(active_url, "http://localhost:11434");
}

/// Test that get_active_endpoint returns error when no endpoint is selected in remote mode
#[test]
fn test_get_active_endpoint_no_selection() {
    let mut config = AppConfig::default();
    
    // Switch to remote mode without setting an active endpoint
    config.backend.set_connection_mode(ConnectionMode::Remote);
    
    // Should return an error
    let result = config.backend.get_active_endpoint_url();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No active remote endpoint selected"));
}

/// Test that set_active_remote_endpoint validates endpoint exists
#[test]
fn test_set_active_endpoint_validation() {
    let mut config = AppConfig::default();
    
    // Try to set a non-existent endpoint as active
    let result = config.backend.set_active_remote_endpoint("non-existent-id");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

/// Test complete workflow: add endpoint, set active, switch modes
#[test]
fn test_complete_connection_mode_workflow() {
    let mut config = AppConfig::default();
    
    // Step 1: Add multiple endpoints
    let endpoint1 = RemoteEndpoint::new(
        "Server 1".to_string(),
        "192.168.1.100".to_string(),
        11434,
        false,
        None,
    ).unwrap();
    
    let endpoint2 = RemoteEndpoint::new(
        "Server 2".to_string(),
        "10.0.0.50".to_string(),
        11434,
        true,
        Some("test-key".to_string()),
    ).unwrap();
    
    let id1 = config.backend.add_remote_endpoint(endpoint1).unwrap();
    let id2 = config.backend.add_remote_endpoint(endpoint2).unwrap();
    
    // Step 2: Set first endpoint as active
    config.backend.set_active_remote_endpoint(&id1).unwrap();
    
    // Step 3: Switch to remote mode
    config.backend.set_connection_mode(ConnectionMode::Remote);
    
    // Step 4: Verify active endpoint
    let active_url = config.backend.get_active_endpoint_url().unwrap();
    assert_eq!(active_url, "http://192.168.1.100:11434");
    
    // Step 5: Switch to second endpoint
    config.backend.set_active_remote_endpoint(&id2).unwrap();
    
    // Step 6: Verify new active endpoint (with HTTPS)
    let active_url = config.backend.get_active_endpoint_url().unwrap();
    assert_eq!(active_url, "https://10.0.0.50:11434");
    
    // Step 7: Switch back to local mode
    config.backend.set_connection_mode(ConnectionMode::Local);
    
    // Step 8: Verify localhost is active
    let active_url = config.backend.get_active_endpoint_url().unwrap();
    assert_eq!(active_url, "http://localhost:11434");
    
    // Step 9: Switch back to remote mode
    config.backend.set_connection_mode(ConnectionMode::Remote);
    
    // Step 10: Verify second endpoint is still active
    let active_url = config.backend.get_active_endpoint_url().unwrap();
    assert_eq!(active_url, "https://10.0.0.50:11434");
}
