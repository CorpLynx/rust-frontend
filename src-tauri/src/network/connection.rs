use super::{ConnectionTestResult, NetworkError, OllamaClient};
use crate::config::{AppConfig, ConnectionMode};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Cache entry for connection test results
#[derive(Debug, Clone)]
struct CachedTestResult {
    result: ConnectionTestResult,
    timestamp: SystemTime,
}

impl CachedTestResult {
    /// Check if the cached result is still valid (less than 5 minutes old)
    fn is_valid(&self) -> bool {
        if let Ok(elapsed) = self.timestamp.elapsed() {
            elapsed < Duration::from_secs(300) // 5 minutes
        } else {
            false
        }
    }
}

/// Manages connections to Ollama instances and caches test results
/// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
pub struct ConnectionManager {
    config: Arc<RwLock<AppConfig>>,
    client: Arc<OllamaClient>,
    test_cache: Arc<RwLock<HashMap<String, CachedTestResult>>>,
}

impl ConnectionManager {
    /// Create a new ConnectionManager
    /// 
    /// # Arguments
    /// * `config` - Shared application configuration
    /// * `client` - Shared Ollama client
    pub fn new(config: Arc<RwLock<AppConfig>>, client: Arc<OllamaClient>) -> Self {
        Self {
            config,
            client,
            test_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the active endpoint URL based on the current connection mode
    /// Requirements: 2.2, 2.3
    pub fn get_active_endpoint(&self) -> Result<String, String> {
        let config = self.config.read()
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        config.backend.get_active_endpoint_url()
    }

    /// Test connection to a specific endpoint
    /// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
    /// 
    /// This method tests connectivity to an Ollama endpoint and caches the result
    /// for 5 minutes to avoid excessive network requests.
    /// 
    /// # Arguments
    /// * `endpoint` - The endpoint URL to test (e.g., "http://localhost:11434")
    /// 
    /// # Returns
    /// A Result containing ConnectionTestResult or NetworkError
    pub async fn test_connection(&self, endpoint: &str) -> Result<ConnectionTestResult, NetworkError> {
        // Check cache first
        if let Ok(cache) = self.test_cache.read() {
            if let Some(cached) = cache.get(endpoint) {
                if cached.is_valid() {
                    return Ok(cached.result.clone());
                }
            }
        }

        // Perform the actual connection test
        let result = self.client.test_connection(endpoint).await?;

        // Cache the result
        if let Ok(mut cache) = self.test_cache.write() {
            cache.insert(
                endpoint.to_string(),
                CachedTestResult {
                    result: result.clone(),
                    timestamp: SystemTime::now(),
                },
            );
        }

        Ok(result)
    }

    /// Test connection to the currently active endpoint
    /// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
    /// 
    /// # Returns
    /// A Result containing ConnectionTestResult or an error message
    pub async fn test_active_connection(&self) -> Result<ConnectionTestResult, String> {
        let endpoint = self.get_active_endpoint()?;
        self.test_connection(&endpoint)
            .await
            .map_err(|e| e.to_string())
    }

    /// Switch connection mode
    /// Requirements: 2.4, 2.5
    /// 
    /// # Arguments
    /// * `mode` - The connection mode to switch to
    pub async fn switch_mode(&self, mode: ConnectionMode) -> Result<(), String> {
        let mut config = self.config.write()
            .map_err(|e| format!("Failed to write config: {}", e))?;
        
        config.backend.set_connection_mode(mode);
        
        // Save the configuration to persist the change
        config.save()
            .map_err(|e| format!("Failed to save config: {}", e))?;
        
        Ok(())
    }

    /// Set the active remote endpoint
    /// Requirements: 2.3, 5.2
    /// 
    /// # Arguments
    /// * `endpoint_id` - The ID of the endpoint to set as active
    pub async fn set_active_remote_endpoint(&self, endpoint_id: &str) -> Result<(), String> {
        let mut config = self.config.write()
            .map_err(|e| format!("Failed to write config: {}", e))?;
        
        config.backend.set_active_remote_endpoint(endpoint_id)?;
        
        // Save the configuration to persist the change
        config.save()
            .map_err(|e| format!("Failed to save config: {}", e))?;
        
        Ok(())
    }

    /// Clear the test cache
    /// This can be useful for forcing fresh connection tests
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.test_cache.write() {
            cache.clear();
        }
    }

    /// Clear cached result for a specific endpoint
    /// 
    /// # Arguments
    /// * `endpoint` - The endpoint URL to clear from cache
    pub fn clear_endpoint_cache(&self, endpoint: &str) {
        if let Ok(mut cache) = self.test_cache.write() {
            cache.remove(endpoint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RemoteEndpoint;

    fn create_test_config() -> AppConfig {
        let mut config = AppConfig::default();
        
        // Add a test remote endpoint
        let endpoint = RemoteEndpoint::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            11434,
            false,
            None,
        ).unwrap();
        
        let endpoint_id = config.backend.add_remote_endpoint(endpoint).unwrap();
        config.backend.set_active_remote_endpoint(&endpoint_id).unwrap();
        
        config
    }

    #[test]
    fn test_connection_manager_creation() {
        let config = Arc::new(RwLock::new(create_test_config()));
        let client = Arc::new(OllamaClient::new());
        
        let manager = ConnectionManager::new(config, client);
        
        // Verify manager was created successfully
        assert!(std::mem::size_of_val(&manager) > 0);
    }

    #[test]
    fn test_get_active_endpoint_local_mode() {
        let config = Arc::new(RwLock::new(AppConfig::default()));
        let client = Arc::new(OllamaClient::new());
        
        let manager = ConnectionManager::new(config, client);
        
        let endpoint = manager.get_active_endpoint().unwrap();
        assert_eq!(endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_get_active_endpoint_remote_mode() {
        let mut config = create_test_config();
        config.backend.set_connection_mode(ConnectionMode::Remote);
        
        let config = Arc::new(RwLock::new(config));
        let client = Arc::new(OllamaClient::new());
        
        let manager = ConnectionManager::new(config, client);
        
        let endpoint = manager.get_active_endpoint().unwrap();
        assert_eq!(endpoint, "http://192.168.1.100:11434");
    }

    #[test]
    fn test_clear_cache() {
        let config = Arc::new(RwLock::new(AppConfig::default()));
        let client = Arc::new(OllamaClient::new());
        
        let manager = ConnectionManager::new(config, client);
        
        // Add a fake cache entry
        {
            let mut cache = manager.test_cache.write().unwrap();
            cache.insert(
                "http://test:11434".to_string(),
                CachedTestResult {
                    result: ConnectionTestResult {
                        success: true,
                        response_time_ms: 100,
                        error_message: None,
                    },
                    timestamp: SystemTime::now(),
                },
            );
        }
        
        // Verify cache has an entry
        {
            let cache = manager.test_cache.read().unwrap();
            assert_eq!(cache.len(), 1);
        }
        
        // Clear cache
        manager.clear_cache();
        
        // Verify cache is empty
        {
            let cache = manager.test_cache.read().unwrap();
            assert_eq!(cache.len(), 0);
        }
    }

    #[test]
    fn test_clear_endpoint_cache() {
        let config = Arc::new(RwLock::new(AppConfig::default()));
        let client = Arc::new(OllamaClient::new());
        
        let manager = ConnectionManager::new(config, client);
        
        // Add two fake cache entries
        {
            let mut cache = manager.test_cache.write().unwrap();
            cache.insert(
                "http://test1:11434".to_string(),
                CachedTestResult {
                    result: ConnectionTestResult {
                        success: true,
                        response_time_ms: 100,
                        error_message: None,
                    },
                    timestamp: SystemTime::now(),
                },
            );
            cache.insert(
                "http://test2:11434".to_string(),
                CachedTestResult {
                    result: ConnectionTestResult {
                        success: true,
                        response_time_ms: 150,
                        error_message: None,
                    },
                    timestamp: SystemTime::now(),
                },
            );
        }
        
        // Verify cache has two entries
        {
            let cache = manager.test_cache.read().unwrap();
            assert_eq!(cache.len(), 2);
        }
        
        // Clear one endpoint
        manager.clear_endpoint_cache("http://test1:11434");
        
        // Verify only one entry remains
        {
            let cache = manager.test_cache.read().unwrap();
            assert_eq!(cache.len(), 1);
            assert!(cache.contains_key("http://test2:11434"));
            assert!(!cache.contains_key("http://test1:11434"));
        }
    }

    #[test]
    fn test_cached_result_is_valid() {
        let result = CachedTestResult {
            result: ConnectionTestResult {
                success: true,
                response_time_ms: 100,
                error_message: None,
            },
            timestamp: SystemTime::now(),
        };
        
        assert!(result.is_valid());
    }

    #[test]
    fn test_cached_result_expires() {
        let result = CachedTestResult {
            result: ConnectionTestResult {
                success: true,
                response_time_ms: 100,
                error_message: None,
            },
            timestamp: SystemTime::now() - Duration::from_secs(301), // 5 minutes + 1 second ago
        };
        
        assert!(!result.is_valid());
    }
}
