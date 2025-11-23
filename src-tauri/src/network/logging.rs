use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::ConnectionMode;
use super::NetworkError;

/// Context information for error logging
/// Requirements: 7.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: String,
    pub connection_mode: String,
    pub endpoint: Option<String>,
    pub error_category: String,
    pub additional_info: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(
        mode: &ConnectionMode,
        endpoint: Option<String>,
        error_category: String,
    ) -> Self {
        let timestamp: DateTime<Utc> = Utc::now();
        Self {
            timestamp: timestamp.to_rfc3339(),
            connection_mode: format!("{:?}", mode),
            endpoint: endpoint.map(|e| redact_api_key(&e)),
            error_category,
            additional_info: None,
        }
    }
    
    /// Add additional information to the context
    pub fn with_info(mut self, info: String) -> Self {
        self.additional_info = Some(info);
        self
    }
}

/// Logger for error messages
/// Requirements: 7.5
pub struct ErrorLogger {
    log_path: PathBuf,
}

impl ErrorLogger {
    /// Create a new error logger
    pub fn new() -> Self {
        Self {
            log_path: PathBuf::from("logs/error.log"),
        }
    }
    
    /// Create a logger with a custom path (useful for testing)
    pub fn with_path(path: PathBuf) -> Self {
        Self { log_path: path }
    }
    
    /// Log an error with context
    /// Requirements: 7.5
    pub fn log_error(&self, error: &NetworkError, context: ErrorContext) -> Result<(), std::io::Error> {
        // Ensure the logs directory exists
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open the log file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        
        // Format the log entry
        let log_entry = self.format_log_entry(error, &context);
        
        // Write to file
        writeln!(file, "{}", log_entry)?;
        
        Ok(())
    }
    
    /// Format a log entry with error and context
    fn format_log_entry(&self, error: &NetworkError, context: &ErrorContext) -> String {
        let mut entry = String::new();
        
        entry.push_str(&format!("[{}] ", context.timestamp));
        entry.push_str(&format!("ERROR: {} - ", context.error_category));
        entry.push_str(&format!("{}", get_user_friendly_message(error)));
        entry.push_str(&format!(" | Mode: {}", context.connection_mode));
        
        if let Some(endpoint) = &context.endpoint {
            entry.push_str(&format!(" | Endpoint: {}", endpoint));
        }
        
        if let Some(info) = &context.additional_info {
            entry.push_str(&format!(" | Info: {}", info));
        }
        
        // Add technical details for debugging
        entry.push_str(&format!(" | Technical: {:?}", error));
        
        entry
    }
}

impl Default for ErrorLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Get user-friendly error message for display
/// Requirements: 7.1, 7.2, 7.3, 7.4
pub fn get_user_friendly_message(error: &NetworkError) -> String {
    match error {
        NetworkError::Timeout => {
            "Server unreachable - connection timed out".to_string()
        }
        NetworkError::ConnectionRefused => {
            "Server not accepting connections".to_string()
        }
        NetworkError::InvalidResponse(msg) => {
            format!("Invalid response from server - protocol mismatch: {}", msg)
        }
        NetworkError::TlsError(msg) => {
            format!("Certificate error: {}", msg)
        }
        NetworkError::Other(msg) => {
            format!("Network error: {}", msg)
        }
    }
}

/// Redact API keys from URLs and strings
/// Requirements: 8.5
pub fn redact_api_key(text: &str) -> String {
    // Common patterns for API keys in URLs and headers
    let patterns = [
        (r"(?i)api[_-]?key=([^&\s]+)", "api_key=***REDACTED***"),
        (r"(?i)apikey=([^&\s]+)", "apikey=***REDACTED***"),
        (r"(?i)token=([^&\s]+)", "token=***REDACTED***"),
        (r"(?i)authorization:\s*bearer\s+([^\s]+)", "authorization: bearer ***REDACTED***"),
        (r"(?i)bearer\s+([^\s]+)", "bearer ***REDACTED***"),
    ];
    
    let mut result = text.to_string();
    
    for (pattern, replacement) in patterns.iter() {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }
    
    result
}

/// Get error category for logging
pub fn get_error_category(error: &NetworkError) -> String {
    match error {
        NetworkError::Timeout => "TIMEOUT".to_string(),
        NetworkError::ConnectionRefused => "CONNECTION_REFUSED".to_string(),
        NetworkError::InvalidResponse(_) => "INVALID_RESPONSE".to_string(),
        NetworkError::TlsError(_) => "TLS_ERROR".to_string(),
        NetworkError::Other(_) => "NETWORK_ERROR".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConnectionMode;
    use std::fs;
    
    #[test]
    fn test_user_friendly_messages() {
        let timeout = NetworkError::Timeout;
        assert_eq!(
            get_user_friendly_message(&timeout),
            "Server unreachable - connection timed out"
        );
        
        let refused = NetworkError::ConnectionRefused;
        assert_eq!(
            get_user_friendly_message(&refused),
            "Server not accepting connections"
        );
        
        let invalid = NetworkError::InvalidResponse("test".to_string());
        assert!(get_user_friendly_message(&invalid).contains("protocol mismatch"));
        
        let tls = NetworkError::TlsError("certificate expired".to_string());
        assert!(get_user_friendly_message(&tls).contains("Certificate error"));
    }
    
    #[test]
    fn test_api_key_redaction() {
        let url_with_key = "https://api.example.com?api_key=secret123&other=value";
        let redacted = redact_api_key(url_with_key);
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("***REDACTED***"));
        
        let bearer = "authorization: bearer sk-1234567890abcdef";
        let redacted_bearer = redact_api_key(bearer);
        assert!(!redacted_bearer.contains("sk-1234567890abcdef"));
        assert!(redacted_bearer.contains("***REDACTED***"));
    }
    
    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new(
            &ConnectionMode::Remote,
            Some("https://example.com:11434".to_string()),
            "TEST_ERROR".to_string(),
        );
        
        assert_eq!(context.connection_mode, "Remote");
        assert_eq!(context.error_category, "TEST_ERROR");
        assert!(context.endpoint.is_some());
    }
    
    #[test]
    fn test_error_logging() {
        // Create a temporary log file
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join(format!("test_error_{}.log", std::process::id()));
        
        let logger = ErrorLogger::with_path(log_path.clone());
        
        let error = NetworkError::Timeout;
        let context = ErrorContext::new(
            &ConnectionMode::Local,
            Some("http://localhost:11434".to_string()),
            "TIMEOUT".to_string(),
        );
        
        // Log the error
        let result = logger.log_error(&error, context);
        assert!(result.is_ok());
        
        // Verify the log file was created and contains content
        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("ERROR"));
        assert!(content.contains("TIMEOUT"));
        assert!(content.contains("Server unreachable"));
        
        // Clean up
        fs::remove_file(&log_path).ok();
    }
    
    #[test]
    fn test_error_category() {
        assert_eq!(get_error_category(&NetworkError::Timeout), "TIMEOUT");
        assert_eq!(get_error_category(&NetworkError::ConnectionRefused), "CONNECTION_REFUSED");
        assert_eq!(get_error_category(&NetworkError::InvalidResponse("test".to_string())), "INVALID_RESPONSE");
        assert_eq!(get_error_category(&NetworkError::TlsError("test".to_string())), "TLS_ERROR");
        assert_eq!(get_error_category(&NetworkError::Other("test".to_string())), "NETWORK_ERROR");
    }
}
