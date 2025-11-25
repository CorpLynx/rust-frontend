use anyhow::{Context, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::{sleep, timeout};

/// Status of the Ollama service
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaServiceStatus {
    /// Service is running and responding
    Running,
    /// Service is not running
    NotRunning,
    /// Service is running but not responding to requests
    NotResponding,
    /// Status could not be determined
    Unknown(String),
}

/// Manager for the Ollama service
pub struct OllamaServiceManager {
    local_url: String,
    startup_timeout_secs: u64,
}

impl OllamaServiceManager {
    /// Create a new OllamaServiceManager with default settings
    ///
    /// # Returns
    /// A new OllamaServiceManager configured for localhost:11434
    ///
    /// # Requirements
    /// * 3.1: Connect to Ollama service at http://localhost:11434
    pub fn new() -> Self {
        Self {
            local_url: "http://localhost:11434".to_string(),
            startup_timeout_secs: 10,
        }
    }

    /// Check if Ollama is currently running
    ///
    /// # Returns
    /// `true` if Ollama is running and responding, `false` otherwise
    ///
    /// # Requirements
    /// * 3.1: Attempt to connect to the Ollama service
    /// * 3.2: Determine that Ollama is running when connection succeeds
    /// * 3.3: Determine that Ollama is not running when connection fails
    pub async fn is_running(&self) -> bool {
        // Try to connect to the Ollama health endpoint
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        // Try to fetch from the /api/tags endpoint as a health check
        let url = format!("{}/api/tags", self.local_url);
        match client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Start the Ollama service as a background process
    ///
    /// # Returns
    /// `Ok(())` if the service was started successfully, `Err` otherwise
    ///
    /// # Requirements
    /// * 4.2: Execute the `ollama serve` command as a background process
    /// * 4.6: Display error message if Ollama is not installed
    pub async fn start_service(&self) -> Result<()> {
        // Spawn the ollama serve command as a detached background process
        Command::new("ollama")
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .context("Failed to start Ollama service. Is Ollama installed and in your PATH?")?;

        Ok(())
    }

    /// Wait for Ollama to become ready with timeout and retry logic
    ///
    /// # Arguments
    /// * `timeout_secs` - Maximum time to wait in seconds
    ///
    /// # Returns
    /// `Ok(())` if Ollama becomes ready within the timeout, `Err` otherwise
    ///
    /// # Requirements
    /// * 4.3: Wait for up to 10 seconds for Ollama to become responsive
    /// * 4.4: Display success message when Ollama becomes responsive
    /// * 4.5: Display error message when Ollama does not become responsive within timeout
    pub async fn wait_for_ready(&self, timeout_secs: u64) -> Result<()> {
        let check_interval = Duration::from_millis(500);
        let max_attempts = (timeout_secs * 1000) / 500; // Convert to number of 500ms intervals

        let result = timeout(Duration::from_secs(timeout_secs), async {
            for _ in 0..max_attempts {
                if self.is_running().await {
                    return Ok(());
                }
                sleep(check_interval).await;
            }
            anyhow::bail!("Ollama did not become ready within {} seconds", timeout_secs)
        })
        .await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => anyhow::bail!(
                "Timeout waiting for Ollama to start. Please check if:\n\
                 - Ollama is installed correctly\n\
                 - Port 11434 is not already in use\n\
                 - You have permission to start the service"
            ),
        }
    }

    /// Get the local URL for the Ollama service
    pub fn local_url(&self) -> &str {
        &self.local_url
    }

    /// Get the startup timeout in seconds
    pub fn startup_timeout_secs(&self) -> u64 {
        self.startup_timeout_secs
    }
}

impl Default for OllamaServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_service_manager_creation() {
        let manager = OllamaServiceManager::new();
        assert_eq!(manager.local_url(), "http://localhost:11434");
        assert_eq!(manager.startup_timeout_secs(), 10);
    }

    #[test]
    fn test_ollama_service_manager_default() {
        let manager = OllamaServiceManager::default();
        assert_eq!(manager.local_url(), "http://localhost:11434");
        assert_eq!(manager.startup_timeout_secs(), 10);
    }

    #[test]
    fn test_ollama_service_status_equality() {
        assert_eq!(OllamaServiceStatus::Running, OllamaServiceStatus::Running);
        assert_eq!(OllamaServiceStatus::NotRunning, OllamaServiceStatus::NotRunning);
        assert_eq!(OllamaServiceStatus::NotResponding, OllamaServiceStatus::NotResponding);
        assert_eq!(
            OllamaServiceStatus::Unknown("test".to_string()),
            OllamaServiceStatus::Unknown("test".to_string())
        );
        assert_ne!(OllamaServiceStatus::Running, OllamaServiceStatus::NotRunning);
    }

    #[test]
    fn test_ollama_service_status_clone() {
        let status = OllamaServiceStatus::Running;
        let cloned = status.clone();
        assert_eq!(status, cloned);

        let status = OllamaServiceStatus::Unknown("error".to_string());
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_ollama_service_status_debug() {
        let status = OllamaServiceStatus::Running;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Running"));

        let status = OllamaServiceStatus::Unknown("test error".to_string());
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Unknown"));
        assert!(debug_str.contains("test error"));
    }

    // Note: Integration tests for is_running(), start_service(), and wait_for_ready()
    // would require a running Ollama instance or mocking, which is better suited for
    // integration tests rather than unit tests.
}
