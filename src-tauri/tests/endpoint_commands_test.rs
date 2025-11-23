use app_lib::config::{AppConfig, RemoteEndpoint};
use app_lib::network::{ConnectionManager, OllamaClient};
use std::sync::{Arc, RwLock};

#[test]
fn test_endpoint_management_workflow() {
    // Initialize config
    let config = Arc::new(RwLock::new(AppConfig::default()));
    
    // Test adding an endpoint
    {
        let mut cfg = config.write().unwrap();
        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();
        
        let endpoint_id = cfg.backend.add_remote_endpoint(endpoint).unwrap();
        
        // Verify endpoint was added
        assert_eq!(cfg.backend.list_remote_endpoints().len(), 1);
        assert!(cfg.backend.get_remote_endpoint(&endpoint_id).is_some());
    }
    
    // Test listing endpoints
    {
        let cfg = config.read().unwrap();
        let endpoints = cfg.backend.list_remote_endpoints();
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].name, "Test Server");
    }
    
    // Test updating an endpoint
    {
        let mut cfg = config.write().unwrap();
        let endpoint_id = cfg.backend.list_remote_endpoints()[0].id.clone();
        
        let mut updated_endpoint = RemoteEndpoint::new(
            "Updated Server".to_string(),
            "192.168.1.101".to_string(),
            8080,
            true,
            Some("test-key".to_string()),
        ).unwrap();
        updated_endpoint.id = endpoint_id.clone();
        
        cfg.backend.update_remote_endpoint(&endpoint_id, updated_endpoint).unwrap();
        
        // Verify update
        let endpoint = cfg.backend.get_remote_endpoint(&endpoint_id).unwrap();
        assert_eq!(endpoint.name, "Updated Server");
        assert_eq!(endpoint.host, "192.168.1.101");
        assert_eq!(endpoint.port, 8080);
        assert_eq!(endpoint.use_https, true);
        assert_eq!(endpoint.api_key, Some("test-key".to_string()));
    }
    
    // Test removing an endpoint
    {
        let mut cfg = config.write().unwrap();
        let endpoint_id = cfg.backend.list_remote_endpoints()[0].id.clone();
        
        cfg.backend.remove_remote_endpoint(&endpoint_id).unwrap();
        
        // Verify removal
        assert_eq!(cfg.backend.list_remote_endpoints().len(), 0);
        assert!(cfg.backend.get_remote_endpoint(&endpoint_id).is_none());
    }
}

#[test]
fn test_connection_manager_integration() {
    // Initialize config with an endpoint
    let mut config = AppConfig::default();
    let endpoint = RemoteEndpoint::new(
        "Test Server".to_string(),
        "192.168.1.100".to_string(),
        11434,
        false,
        None,
    ).unwrap();
    
    let endpoint_id = config.backend.add_remote_endpoint(endpoint).unwrap();
    config.backend.set_active_remote_endpoint(&endpoint_id).unwrap();
    
    let config = Arc::new(RwLock::new(config));
    let client = Arc::new(OllamaClient::new());
    
    // Create connection manager
    let manager = ConnectionManager::new(Arc::clone(&config), client);
    
    // Test getting active endpoint in local mode
    {
        let cfg = config.read().unwrap();
        assert_eq!(*cfg.backend.get_connection_mode(), app_lib::config::ConnectionMode::Local);
    }
    
    let endpoint = manager.get_active_endpoint().unwrap();
    assert_eq!(endpoint, "http://localhost:11434");
    
    // Switch to remote mode
    {
        let mut cfg = config.write().unwrap();
        cfg.backend.set_connection_mode(app_lib::config::ConnectionMode::Remote);
    }
    
    let endpoint = manager.get_active_endpoint().unwrap();
    assert_eq!(endpoint, "http://192.168.1.100:11434");
}

#[test]
fn test_endpoint_validation_in_commands() {
    let _config = Arc::new(RwLock::new(AppConfig::default()));
    
    // Test adding endpoint with invalid port
    {
        let result = RemoteEndpoint::new(
            "Invalid Port".to_string(),
            "192.168.1.100".to_string(),
            0, // Invalid port
            false,
            None,
        );
        
        assert!(result.is_err());
    }
    
    // Test adding endpoint with empty name
    {
        let result = RemoteEndpoint::new(
            "".to_string(), // Empty name
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        );
        
        assert!(result.is_err());
    }
    
    // Test adding endpoint with empty host
    {
        let result = RemoteEndpoint::new(
            "Test Server".to_string(),
            "".to_string(), // Empty host
            11434,
            false,
            None,
        );
        
        assert!(result.is_err());
    }
    
    // Test adding valid endpoint
    {
        let result = RemoteEndpoint::new(
            "Valid Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        );
        
        assert!(result.is_ok());
    }
}

#[test]
fn test_duplicate_endpoint_prevention() {
    let config = Arc::new(RwLock::new(AppConfig::default()));
    
    // Add first endpoint
    {
        let mut cfg = config.write().unwrap();
        let endpoint = RemoteEndpoint::new(
            "Server 1".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();
        
        cfg.backend.add_remote_endpoint(endpoint).unwrap();
    }
    
    // Try to add duplicate endpoint (same host:port)
    {
        let mut cfg = config.write().unwrap();
        let endpoint = RemoteEndpoint::new(
            "Server 2".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();
        
        let result = cfg.backend.add_remote_endpoint(endpoint);
        assert!(result.is_err());
    }
    
    // Verify only one endpoint exists
    {
        let cfg = config.read().unwrap();
        assert_eq!(cfg.backend.list_remote_endpoints().len(), 1);
    }
}
