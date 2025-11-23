pub mod connection;
pub mod logging;
pub mod retry;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::{Duration, Instant};
use thiserror::Error as ThisError;

pub use connection::ConnectionManager;
pub use logging::{ErrorLogger, ErrorContext, get_user_friendly_message, redact_api_key, get_error_category};
pub use retry::{RetryConfig, retry_with_backoff};

/// Network error types for Ollama client operations
/// Requirements: 7.1, 7.2, 7.3, 7.4
#[derive(Debug, ThisError)]
pub enum NetworkError {
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("TLS/SSL error: {0}")]
    TlsError(String),
    
    #[error("Network error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for NetworkError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            NetworkError::Timeout
        } else if err.is_connect() {
            NetworkError::ConnectionRefused
        } else if err.is_request() {
            // Check if it's a TLS error
            if let Some(source) = err.source() {
                let source_str = source.to_string().to_lowercase();
                if source_str.contains("tls") || source_str.contains("ssl") || source_str.contains("certificate") {
                    return NetworkError::TlsError(source.to_string());
                }
            }
            NetworkError::Other(err.to_string())
        } else {
            NetworkError::Other(err.to_string())
        }
    }
}

/// Result of a connection test
/// Requirements: 4.3, 4.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

/// Client for communicating with Ollama instances
/// Requirements: 3.1, 3.2, 4.2
pub struct OllamaClient {
    client: Client,
}

impl OllamaClient {
    /// Create a new OllamaClient with default settings
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // Default timeout for requests
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }
    
    /// Send a request to an Ollama endpoint
    /// Requirements: 3.1, 3.2
    /// 
    /// # Arguments
    /// * `endpoint` - The base URL of the Ollama instance (e.g., "http://localhost:11434")
    /// * `prompt` - The prompt to send to the model
    /// * `model` - The model name to use
    /// 
    /// # Returns
    /// A Result containing the Response or a NetworkError
    pub async fn send_request(
        &self,
        endpoint: &str,
        prompt: &str,
        model: &str,
    ) -> Result<Response, NetworkError> {
        let url = format!("{}/api/generate", endpoint);
        
        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": true
        });
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(NetworkError::InvalidResponse(
                format!("Server returned status: {}", response.status())
            ));
        }
        
        Ok(response)
    }
    
    /// Test connection to an Ollama endpoint
    /// Requirements: 4.2, 4.5
    /// 
    /// # Arguments
    /// * `endpoint` - The base URL of the Ollama instance to test
    /// 
    /// # Returns
    /// A Result containing ConnectionTestResult or NetworkError
    pub async fn test_connection(&self, endpoint: &str) -> Result<ConnectionTestResult, NetworkError> {
        let url = format!("{}/api/tags", endpoint);
        let start = Instant::now();
        
        // Create a client with a shorter timeout for connection tests (10 seconds)
        let test_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        
        match test_client.get(&url).send().await {
            Ok(response) => {
                let elapsed = start.elapsed();
                
                if response.status().is_success() {
                    Ok(ConnectionTestResult {
                        success: true,
                        response_time_ms: elapsed.as_millis() as u64,
                        error_message: None,
                    })
                } else {
                    Ok(ConnectionTestResult {
                        success: false,
                        response_time_ms: elapsed.as_millis() as u64,
                        error_message: Some(format!("Server returned status: {}", response.status())),
                    })
                }
            }
            Err(e) => {
                let elapsed = start.elapsed();
                let network_error: NetworkError = e.into();
                
                Ok(ConnectionTestResult {
                    success: false,
                    response_time_ms: elapsed.as_millis() as u64,
                    error_message: Some(network_error.to_string()),
                })
            }
        }
    }
    
    /// List available models from an Ollama endpoint
    /// Requirements: 3.1, 3.2
    /// 
    /// # Arguments
    /// * `endpoint` - The base URL of the Ollama instance
    /// 
    /// # Returns
    /// A Result containing a vector of model names or NetworkError
    pub async fn list_models(&self, endpoint: &str) -> Result<Vec<String>, NetworkError> {
        let url = format!("{}/api/tags", endpoint);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(NetworkError::InvalidResponse(
                format!("Server returned status: {}", response.status())
            ));
        }
        
        let response_text = response.text().await
            .map_err(|e| NetworkError::Other(format!("Failed to read response: {}", e)))?;
        
        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| NetworkError::InvalidResponse(format!("Failed to parse JSON: {}", e)))?;
        
        // Parse Ollama response format
        let model_names: Vec<String> = if let Some(models) = json.get("models").and_then(|v| v.as_array()) {
            models
                .iter()
                .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };
        
        if model_names.is_empty() {
            Err(NetworkError::InvalidResponse("No models found".to_string()))
        } else {
            Ok(model_names)
        }
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new();
        // Just verify we can create a client without panicking
        assert!(std::mem::size_of_val(&client) > 0);
    }
    
    #[test]
    fn test_network_error_display() {
        let timeout_err = NetworkError::Timeout;
        assert_eq!(timeout_err.to_string(), "Connection timeout");
        
        let refused_err = NetworkError::ConnectionRefused;
        assert_eq!(refused_err.to_string(), "Connection refused");
        
        let invalid_err = NetworkError::InvalidResponse("test".to_string());
        assert_eq!(invalid_err.to_string(), "Invalid response: test");
        
        let tls_err = NetworkError::TlsError("certificate error".to_string());
        assert_eq!(tls_err.to_string(), "TLS/SSL error: certificate error");
    }
}
