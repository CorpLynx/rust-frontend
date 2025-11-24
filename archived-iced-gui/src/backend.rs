use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Response from Ollama's /api/tags endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

/// Backend client for communicating with Ollama instances
#[derive(Clone)]
pub struct BackendClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl BackendClient {
    /// Create a new BackendClient
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Ollama instance (e.g., "http://localhost:11434")
    /// * `timeout_seconds` - Request timeout in seconds
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(timeout_seconds),
        })
    }

    /// Send a prompt to the backend with streaming response handling
    ///
    /// # Arguments
    /// * `prompt` - The prompt to send to the model
    /// * `model` - The model name to use
    /// * `callback` - A callback function that receives each chunk of the response
    ///
    /// # Returns
    /// The complete response text
    pub async fn send_prompt_streaming<F>(
        &self,
        prompt: &str,
        model: &str,
        mut callback: F,
    ) -> Result<String>
    where
        F: FnMut(String) -> Result<()>,
    {
        let url = format!("{}/api/generate", self.base_url);

        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": true
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context(format!("Failed to connect to {}", self.base_url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Backend returned error status: {} - {}",
                status.as_u16(),
                error_body
            );
        }

        let mut stream = response.bytes_stream();
        let mut line_buffer = String::new();
        let mut full_response = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.context("Failed to read stream chunk")?;
            let text = std::str::from_utf8(&bytes)
                .context("Failed to decode stream chunk as UTF-8")?;

            line_buffer.push_str(text);

            // Process complete JSON lines
            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].trim().to_string();
                line_buffer = line_buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<serde_json::Value>(&line) {
                    Ok(json) => {
                        if let Some(response_text) = json.get("response").and_then(|v| v.as_str())
                        {
                            full_response.push_str(response_text);
                            callback(response_text.to_string())?;
                        }

                        // Check if done
                        if json.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                            return Ok(full_response);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to parse JSON line: {} - Error: {}", line, e);
                    }
                }
            }
        }

        Ok(full_response)
    }

    /// Fetch available models from the backend
    ///
    /// # Returns
    /// A vector of model names
    pub async fn fetch_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to connect to {}", self.base_url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Backend returned error status: {} - {}",
                status.as_u16(),
                error_body
            );
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse JSON response")?;

        // Try multiple parsing strategies to support different backend formats
        let model_names: Vec<String> = if let Some(models_array) = json.as_array() {
            // Direct array of strings: ["model1", "model2"]
            models_array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else if let Some(data) = json.get("data").and_then(|v| v.as_array()) {
            // OpenAI-style: {"data": [{"id": "model1"}, ...]}
            data.iter()
                .filter_map(|v| {
                    v.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        } else if let Some(models) = json.get("models").and_then(|v| v.as_array()) {
            // Ollama-style: {"models": [{"name": "model1"}, ...]}
            models
                .iter()
                .filter_map(|v| {
                    if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                        Some(name.to_string())
                    } else if let Some(id) = v.get("id").and_then(|n| n.as_str()) {
                        Some(id.to_string())
                    } else if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            anyhow::bail!("Unexpected response format: {}", response_text);
        };

        if model_names.is_empty() {
            anyhow::bail!("No models found in response: {}", response_text);
        }

        Ok(model_names)
    }

    /// Get the base URL of this client
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_client_creation() {
        let client = BackendClient::new("http://localhost:11434".to_string(), 30);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url(), "http://localhost:11434");
        assert_eq!(client.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_backend_client_trims_trailing_slash() {
        let client =
            BackendClient::new("http://localhost:11434/".to_string(), 30).unwrap();
        assert_eq!(client.base_url(), "http://localhost:11434");
    }

    #[test]
    fn test_backend_client_with_different_timeouts() {
        let client1 = BackendClient::new("http://localhost:11434".to_string(), 10).unwrap();
        assert_eq!(client1.timeout(), Duration::from_secs(10));

        let client2 = BackendClient::new("http://localhost:11434".to_string(), 120).unwrap();
        assert_eq!(client2.timeout(), Duration::from_secs(120));
    }

    #[test]
    fn test_backend_client_with_different_urls() {
        let client1 = BackendClient::new("http://localhost:11434".to_string(), 30).unwrap();
        assert_eq!(client1.base_url(), "http://localhost:11434");

        let client2 = BackendClient::new("https://api.example.com".to_string(), 30).unwrap();
        assert_eq!(client2.base_url(), "https://api.example.com");

        let client3 = BackendClient::new("http://192.168.1.100:8080".to_string(), 30).unwrap();
        assert_eq!(client3.base_url(), "http://192.168.1.100:8080");
    }

    #[test]
    fn test_backend_client_trims_multiple_trailing_slashes() {
        let client =
            BackendClient::new("http://localhost:11434///".to_string(), 30).unwrap();
        // Note: trim_end_matches('/') removes all trailing slashes
        assert_eq!(client.base_url(), "http://localhost:11434");
    }

    // Test for request formatting - we'll use a mock server for integration tests
    // For now, we test that the client can be created with various configurations
    #[test]
    fn test_backend_client_creation_with_edge_cases() {
        // Empty URL should still create a client (validation happens at request time)
        let client = BackendClient::new("".to_string(), 30);
        assert!(client.is_ok());

        // Very short timeout
        let client = BackendClient::new("http://localhost:11434".to_string(), 1);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().timeout(), Duration::from_secs(1));

        // Very long timeout
        let client = BackendClient::new("http://localhost:11434".to_string(), 3600);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().timeout(), Duration::from_secs(3600));
    }

    #[test]
    fn test_ollama_model_serialization() {
        let model = OllamaModel {
            name: "llama2".to_string(),
            modified_at: "2024-01-01T00:00:00Z".to_string(),
            size: 1024,
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("llama2"));
        assert!(json.contains("2024-01-01T00:00:00Z"));
        assert!(json.contains("1024"));

        let deserialized: OllamaModel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "llama2");
        assert_eq!(deserialized.modified_at, "2024-01-01T00:00:00Z");
        assert_eq!(deserialized.size, 1024);
    }

    #[test]
    fn test_ollama_models_response_serialization() {
        let response = OllamaModelsResponse {
            models: vec![
                OllamaModel {
                    name: "model1".to_string(),
                    modified_at: "2024-01-01T00:00:00Z".to_string(),
                    size: 1024,
                },
                OllamaModel {
                    name: "model2".to_string(),
                    modified_at: "2024-01-02T00:00:00Z".to_string(),
                    size: 2048,
                },
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("model1"));
        assert!(json.contains("model2"));

        let deserialized: OllamaModelsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.models.len(), 2);
        assert_eq!(deserialized.models[0].name, "model1");
        assert_eq!(deserialized.models[1].name, "model2");
    }
}
