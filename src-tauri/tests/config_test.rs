use app_lib::config::{RemoteEndpoint, ConnectionMode, BackendSettings};

#[test]
fn test_remote_endpoint_creation() {
    let endpoint = RemoteEndpoint::new(
        "Test Server".to_string(),
        "192.168.1.100".to_string(),
        11434,
        false,
        None,
    );
    
    assert!(endpoint.is_ok());
    let endpoint = endpoint.unwrap();
    assert_eq!(endpoint.name, "Test Server");
    assert_eq!(endpoint.host, "192.168.1.100");
    assert_eq!(endpoint.port, 11434);
    assert_eq!(endpoint.use_https, false);
    assert_eq!(endpoint.url(), "http://192.168.1.100:11434");
}

#[test]
fn test_remote_endpoint_validation_empty_name() {
    let endpoint = RemoteEndpoint::new(
        "".to_string(),
        "192.168.1.100".to_string(),
        11434,
        false,
        None,
    );
    
    assert!(endpoint.is_err());
}

#[test]
fn test_remote_endpoint_validation_empty_host() {
    let endpoint = RemoteEndpoint::new(
        "Test Server".to_string(),
        "".to_string(),
        11434,
        false,
        None,
    );
    
    assert!(endpoint.is_err());
}

#[test]
fn test_remote_endpoint_validation_invalid_port() {
    let endpoint = RemoteEndpoint::new(
        "Test Server".to_string(),
        "192.168.1.100".to_string(),
        0,
        false,
        None,
    );
    
    assert!(endpoint.is_err());
}

#[test]
fn test_remote_endpoint_url_with_https() {
    let endpoint = RemoteEndpoint::new(
        "Test Server".to_string(),
        "example.com".to_string(),
        443,
        true,
        None,
    ).unwrap();
    
    assert_eq!(endpoint.url(), "https://example.com:443");
}

#[test]
fn test_connection_mode_default() {
    let mode = ConnectionMode::default();
    assert_eq!(mode, ConnectionMode::Local);
}

#[test]
fn test_backend_settings_new_fields() {
    let settings = BackendSettings {
        url: "http://localhost:1234".to_string(),
        ollama_url: "http://localhost:11434".to_string(),
        timeout_seconds: 30,
        saved_urls: Vec::new(),
        remote_endpoints: Vec::new(),
        connection_mode: ConnectionMode::Local,
        active_remote_endpoint_id: None,
    };
    
    assert_eq!(settings.connection_mode, ConnectionMode::Local);
    assert_eq!(settings.remote_endpoints.len(), 0);
    assert_eq!(settings.active_remote_endpoint_id, None);
}

#[test]
fn test_backend_settings_with_remote_endpoint() {
    let endpoint = RemoteEndpoint::new(
        "Remote Server".to_string(),
        "10.0.0.50".to_string(),
        11434,
        false,
        Some("test-api-key".to_string()),
    ).unwrap();
    
    let settings = BackendSettings {
        url: "http://localhost:1234".to_string(),
        ollama_url: "http://localhost:11434".to_string(),
        timeout_seconds: 30,
        saved_urls: Vec::new(),
        remote_endpoints: vec![endpoint.clone()],
        connection_mode: ConnectionMode::Remote,
        active_remote_endpoint_id: Some(endpoint.id.clone()),
    };
    
    assert_eq!(settings.connection_mode, ConnectionMode::Remote);
    assert_eq!(settings.remote_endpoints.len(), 1);
    assert_eq!(settings.remote_endpoints[0].name, "Remote Server");
    assert_eq!(settings.active_remote_endpoint_id, Some(endpoint.id));
}
