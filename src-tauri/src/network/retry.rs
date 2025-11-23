use std::time::Duration;
use tokio::time::sleep;

use super::NetworkError;

/// Retry configuration with exponential backoff
/// Requirements: 7.5
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, initial_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
    
    /// Calculate delay for a given attempt number (0-indexed)
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.initial_delay_ms as f64 
            * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let capped_delay = delay_ms.min(self.max_delay_ms);
        Duration::from_millis(capped_delay)
    }
}

/// Retry a network operation with exponential backoff
/// Requirements: 7.5
/// 
/// # Arguments
/// * `config` - Retry configuration
/// * `operation` - Async operation to retry
/// 
/// # Returns
/// Result of the operation or the last error encountered
pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, NetworkError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, NetworkError>>,
{
    let mut last_error = None;
    
    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                // Check if the error is retryable
                if !is_retryable_error(&error) {
                    return Err(error);
                }
                
                last_error = Some(error);
                
                // Don't sleep after the last attempt
                if attempt < config.max_attempts - 1 {
                    let delay = config.calculate_delay(attempt);
                    sleep(delay).await;
                }
            }
        }
    }
    
    // Return the last error if all attempts failed
    Err(last_error.unwrap_or(NetworkError::Other("All retry attempts failed".to_string())))
}

/// Determine if an error is retryable
/// Requirements: 7.5
/// 
/// Retryable errors:
/// - Timeout
/// - Connection refused
/// - Generic network errors
/// 
/// Non-retryable errors:
/// - Invalid response (likely a protocol issue)
/// - TLS errors (require configuration changes)
fn is_retryable_error(error: &NetworkError) -> bool {
    match error {
        NetworkError::Timeout => true,
        NetworkError::ConnectionRefused => true,
        NetworkError::Other(_) => true,
        NetworkError::InvalidResponse(_) => false,
        NetworkError::TlsError(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
    }
    
    #[test]
    fn test_calculate_delay_exponential() {
        let config = RetryConfig::default();
        
        // First attempt: 100ms
        assert_eq!(config.calculate_delay(0).as_millis(), 100);
        
        // Second attempt: 200ms
        assert_eq!(config.calculate_delay(1).as_millis(), 200);
        
        // Third attempt: 400ms
        assert_eq!(config.calculate_delay(2).as_millis(), 400);
        
        // Fourth attempt: 800ms
        assert_eq!(config.calculate_delay(3).as_millis(), 800);
    }
    
    #[test]
    fn test_calculate_delay_capped() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay_ms: 1000,
            max_delay_ms: 3000,
            backoff_multiplier: 2.0,
        };
        
        // Should be capped at max_delay_ms
        let delay = config.calculate_delay(5);
        assert_eq!(delay.as_millis(), 3000);
    }
    
    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error(&NetworkError::Timeout));
        assert!(is_retryable_error(&NetworkError::ConnectionRefused));
        assert!(is_retryable_error(&NetworkError::Other("test".to_string())));
        
        assert!(!is_retryable_error(&NetworkError::InvalidResponse("test".to_string())));
        assert!(!is_retryable_error(&NetworkError::TlsError("test".to_string())));
    }
    
    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        
        let config = RetryConfig::default();
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        
        let result = retry_with_backoff(&config, move || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok::<i32, NetworkError>(42)
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        
        let config = RetryConfig::default();
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        
        let result = retry_with_backoff(&config, move || {
            let count = call_count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                if current == 1 {
                    Err(NetworkError::Timeout)
                } else {
                    Ok::<i32, NetworkError>(42)
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
    
    #[tokio::test]
    async fn test_retry_all_attempts_fail() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        
        let config = RetryConfig::new(3, 10); // Short delays for testing
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        
        let result = retry_with_backoff(&config, move || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<i32, NetworkError>(NetworkError::Timeout)
            }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
        assert!(matches!(result.unwrap_err(), NetworkError::Timeout));
    }
    
    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        
        let config = RetryConfig::default();
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        
        let result = retry_with_backoff(&config, move || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<i32, NetworkError>(NetworkError::TlsError("cert error".to_string()))
            }
        }).await;
        
        assert!(result.is_err());
        // Should fail immediately without retrying
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert!(matches!(result.unwrap_err(), NetworkError::TlsError(_)));
    }
}
