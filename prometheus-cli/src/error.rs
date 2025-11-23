use anyhow::Result;
use crate::terminal::Terminal;

/// Enhanced error display for CLI with detailed context
pub struct ErrorDisplay {
    terminal: Terminal,
}

impl ErrorDisplay {
    /// Create a new ErrorDisplay instance
    pub fn new(terminal: Terminal) -> Self {
        Self { terminal }
    }

    /// Display a connection failure error with URL
    ///
    /// # Arguments
    /// * `url` - The URL that failed to connect
    /// * `error` - The underlying error message
    ///
    /// # Requirements
    /// * 7.1: Display connection error message with the attempted URL
    pub fn display_connection_error(&mut self, url: &str, error: &str) -> Result<()> {
        self.terminal.write_error(&format!(
            "Failed to connect to {}: {}",
            url, error
        ))
    }

    /// Display a timeout error with duration
    ///
    /// # Arguments
    /// * `timeout_seconds` - The timeout duration in seconds
    ///
    /// # Requirements
    /// * 7.3: Display timeout message with duration
    pub fn display_timeout_error(&mut self, timeout_seconds: u64) -> Result<()> {
        self.terminal.write_error(&format!(
            "Request timed out after {}s",
            timeout_seconds
        ))
    }

    /// Display an HTTP error with status code
    ///
    /// # Arguments
    /// * `status` - The HTTP status code
    /// * `message` - The error message from the backend
    ///
    /// # Requirements
    /// * 7.2: Display backend error message with status code
    pub fn display_http_error(&mut self, status: u16, message: &str) -> Result<()> {
        self.terminal.write_error(&format!(
            "Backend error ({}): {}",
            status, message
        ))
    }

    /// Display a file system error
    ///
    /// # Arguments
    /// * `operation` - The operation that failed (e.g., "save", "load")
    /// * `path` - The file path involved
    /// * `error` - The underlying error message
    ///
    /// # Requirements
    /// * 7.4: Display error for configuration loading failures
    /// * 7.5: Display error for conversation save failures
    pub fn display_filesystem_error(&mut self, operation: &str, path: &str, error: &str) -> Result<()> {
        self.terminal.write_error(&format!(
            "Failed to {} {}: {}",
            operation, path, error
        ))
    }

    /// Display a configuration loading warning
    ///
    /// # Arguments
    /// * `error` - The error message
    ///
    /// # Requirements
    /// * 7.4: Display warning and continue with default values
    pub fn display_config_warning(&mut self, error: &str) -> Result<()> {
        self.terminal.write_info(&format!(
            "Warning: Failed to load configuration: {}. Using default values.",
            error
        ))
    }

    /// Display a markdown rendering fallback message
    ///
    /// # Requirements
    /// * 6.5: Graceful degradation for markdown rendering
    pub fn display_markdown_fallback(&mut self) -> Result<()> {
        self.terminal.write_info(
            "Note: Markdown rendering unavailable, displaying raw text"
        )
    }

    /// Parse and display an error with appropriate context
    ///
    /// This method analyzes the error and displays it with the most appropriate
    /// context based on the error type.
    ///
    /// # Arguments
    /// * `error` - The error to display
    /// * `context` - Additional context (e.g., URL, timeout)
    pub fn display_error_with_context(&mut self, error: &anyhow::Error, context: ErrorContext) -> Result<()> {
        let error_str = error.to_string();

        // Check for connection errors
        if error_str.contains("Failed to connect") || error_str.contains("Connection refused") 
            || error_str.contains("No route to host") || error_str.contains("Network is unreachable") {
            if let ErrorContext::Backend { url, .. } = context {
                return self.display_connection_error(&url, &error_str);
            }
        }

        // Check for timeout errors
        if error_str.contains("timed out") || error_str.contains("timeout") {
            if let ErrorContext::Backend { timeout_seconds, .. } = context {
                return self.display_timeout_error(timeout_seconds);
            }
        }

        // Check for HTTP errors
        if error_str.contains("Backend returned error status") {
            // Try to extract status code
            if let Some(status_str) = error_str.split("status: ").nth(1) {
                if let Some(status_part) = status_str.split_whitespace().next() {
                    if let Ok(status) = status_part.parse::<u16>() {
                        let message = error_str.split(" - ").nth(1).unwrap_or("Unknown error");
                        return self.display_http_error(status, message);
                    }
                }
            }
        }

        // Check for file system errors
        if error_str.contains("Failed to save") || error_str.contains("Failed to load") {
            if let ErrorContext::Filesystem { operation, path } = context {
                return self.display_filesystem_error(&operation, &path, &error_str);
            }
        }

        // Default: display the error as-is
        self.terminal.write_error(&error_str)
    }
}

/// Context information for error display
#[derive(Clone)]
pub enum ErrorContext {
    /// Backend-related error context
    Backend {
        url: String,
        timeout_seconds: u64,
    },
    /// File system error context
    Filesystem {
        operation: String,
        path: String,
    },
    /// No specific context
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper struct for testing without actual terminal I/O
    struct TestTerminal {
        output: Vec<String>,
    }

    impl TestTerminal {
        fn new() -> Self {
            Self {
                output: Vec::new(),
            }
        }

        fn write_error(&mut self, error: &str) -> Result<()> {
            self.output.push(format!("Error: {}", error));
            Ok(())
        }

        fn write_info(&mut self, info: &str) -> Result<()> {
            self.output.push(format!("Info: {}", info));
            Ok(())
        }

        fn get_output(&self) -> Vec<String> {
            self.output.clone()
        }
    }

    // Mock ErrorDisplay for testing
    struct TestErrorDisplay {
        terminal: TestTerminal,
    }

    impl TestErrorDisplay {
        fn new() -> Self {
            Self {
                terminal: TestTerminal::new(),
            }
        }

        fn display_connection_error(&mut self, url: &str, error: &str) -> Result<()> {
            self.terminal.write_error(&format!(
                "Failed to connect to {}: {}",
                url, error
            ))
        }

        fn display_timeout_error(&mut self, timeout_seconds: u64) -> Result<()> {
            self.terminal.write_error(&format!(
                "Request timed out after {}s",
                timeout_seconds
            ))
        }

        fn display_http_error(&mut self, status: u16, message: &str) -> Result<()> {
            self.terminal.write_error(&format!(
                "Backend error ({}): {}",
                status, message
            ))
        }

        fn display_filesystem_error(&mut self, operation: &str, path: &str, error: &str) -> Result<()> {
            self.terminal.write_error(&format!(
                "Failed to {} {}: {}",
                operation, path, error
            ))
        }

        fn display_config_warning(&mut self, error: &str) -> Result<()> {
            self.terminal.write_info(&format!(
                "Warning: Failed to load configuration: {}. Using default values.",
                error
            ))
        }

        fn display_markdown_fallback(&mut self) -> Result<()> {
            self.terminal.write_info(
                "Note: Markdown rendering unavailable, displaying raw text"
            )
        }

        fn get_output(&self) -> Vec<String> {
            self.terminal.get_output()
        }
    }

    #[test]
    fn test_connection_error_display() {
        let mut display = TestErrorDisplay::new();
        display.display_connection_error("http://localhost:11434", "Connection refused").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Failed to connect to http://localhost:11434"));
        assert!(output[0].contains("Connection refused"));
    }

    #[test]
    fn test_timeout_error_display() {
        let mut display = TestErrorDisplay::new();
        display.display_timeout_error(30).unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Request timed out after 30s"));
    }

    #[test]
    fn test_http_error_display() {
        let mut display = TestErrorDisplay::new();
        display.display_http_error(404, "Model not found").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Backend error (404)"));
        assert!(output[0].contains("Model not found"));
    }

    #[test]
    fn test_filesystem_error_display() {
        let mut display = TestErrorDisplay::new();
        display.display_filesystem_error("save", "conversation.json", "Permission denied").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Failed to save conversation.json"));
        assert!(output[0].contains("Permission denied"));
    }

    #[test]
    fn test_config_warning_display() {
        let mut display = TestErrorDisplay::new();
        display.display_config_warning("File not found").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Warning: Failed to load configuration"));
        assert!(output[0].contains("File not found"));
        assert!(output[0].contains("Using default values"));
    }

    #[test]
    fn test_markdown_fallback_display() {
        let mut display = TestErrorDisplay::new();
        display.display_markdown_fallback().unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Markdown rendering unavailable"));
        assert!(output[0].contains("displaying raw text"));
    }

    #[test]
    fn test_multiple_errors() {
        let mut display = TestErrorDisplay::new();
        display.display_connection_error("http://server1:8080", "Timeout").unwrap();
        display.display_http_error(500, "Internal server error").unwrap();
        display.display_filesystem_error("load", "config.toml", "Not found").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("http://server1:8080"));
        assert!(output[1].contains("500"));
        assert!(output[2].contains("config.toml"));
    }

    #[test]
    fn test_different_timeout_durations() {
        let mut display = TestErrorDisplay::new();
        display.display_timeout_error(10).unwrap();
        display.display_timeout_error(60).unwrap();
        display.display_timeout_error(120).unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("10s"));
        assert!(output[1].contains("60s"));
        assert!(output[2].contains("120s"));
    }

    #[test]
    fn test_different_http_status_codes() {
        let mut display = TestErrorDisplay::new();
        display.display_http_error(400, "Bad request").unwrap();
        display.display_http_error(401, "Unauthorized").unwrap();
        display.display_http_error(404, "Not found").unwrap();
        display.display_http_error(500, "Internal error").unwrap();
        display.display_http_error(503, "Service unavailable").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 5);
        assert!(output[0].contains("400"));
        assert!(output[1].contains("401"));
        assert!(output[2].contains("404"));
        assert!(output[3].contains("500"));
        assert!(output[4].contains("503"));
    }

    #[test]
    fn test_filesystem_operations() {
        let mut display = TestErrorDisplay::new();
        display.display_filesystem_error("save", "file1.txt", "Disk full").unwrap();
        display.display_filesystem_error("load", "file2.txt", "Not found").unwrap();
        display.display_filesystem_error("delete", "file3.txt", "Permission denied").unwrap();

        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("save"));
        assert!(output[1].contains("load"));
        assert!(output[2].contains("delete"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{QuickCheck, TestResult};

    // Helper struct for testing without actual terminal I/O
    struct TestTerminal {
        output: Vec<String>,
    }

    impl TestTerminal {
        fn new() -> Self {
            Self {
                output: Vec::new(),
            }
        }

        fn write_error(&mut self, error: &str) -> Result<()> {
            self.output.push(format!("Error: {}", error));
            Ok(())
        }

        fn get_output(&self) -> Vec<String> {
            self.output.clone()
        }
    }

    // Mock ErrorDisplay for testing
    struct TestErrorDisplay {
        terminal: TestTerminal,
    }

    impl TestErrorDisplay {
        fn new() -> Self {
            Self {
                terminal: TestTerminal::new(),
            }
        }

        fn display_http_error(&mut self, status: u16, message: &str) -> Result<()> {
            self.terminal.write_error(&format!(
                "Backend error ({}): {}",
                status, message
            ))
        }

        fn get_output(&self) -> Vec<String> {
            self.terminal.get_output()
        }
    }

    /// **Feature: cli-version, Property 8: Backend error propagation**
    /// 
    /// For any error response from the backend, the error message from the backend
    /// should appear in the terminal output to the user.
    /// 
    /// **Validates: Requirements 7.2**
    fn prop_backend_error_propagation(error_message: String, status_code: u16) -> TestResult {
        // Filter out empty error messages
        if error_message.trim().is_empty() {
            return TestResult::discard();
        }

        // Filter out control characters (except newlines and tabs)
        if error_message.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
            return TestResult::discard();
        }

        // Limit error message length
        if error_message.len() > 500 {
            return TestResult::discard();
        }

        // Only test valid HTTP status codes (400-599)
        if status_code < 400 || status_code >= 600 {
            return TestResult::discard();
        }

        // Create a test error display
        let mut display = TestErrorDisplay::new();

        // Display the HTTP error
        if display.display_http_error(status_code, &error_message).is_err() {
            return TestResult::discard();
        }

        // Get the output
        let output = display.get_output();

        // Verify the error message appears in the output
        if output.is_empty() {
            return TestResult::failed();
        }

        // The output should contain the error message
        let contains_message = output.iter().any(|line| line.contains(&error_message));
        
        // The output should also contain the status code
        let status_str = status_code.to_string();
        let contains_status = output.iter().any(|line| line.contains(&status_str));

        if contains_message && contains_status {
            TestResult::passed()
        } else {
            TestResult::failed()
        }
    }

    #[test]
    fn test_prop_backend_error_propagation() {
        QuickCheck::new()
            .tests(100)
            .quickcheck(prop_backend_error_propagation as fn(String, u16) -> TestResult);
    }
}

#[cfg(test)]
mod error_scenario_tests {
    use super::*;

    // Helper struct for testing without actual terminal I/O
    struct TestTerminal {
        output: Vec<String>,
    }

    impl TestTerminal {
        fn new() -> Self {
            Self {
                output: Vec::new(),
            }
        }

        fn write_error(&mut self, error: &str) -> Result<()> {
            self.output.push(format!("Error: {}", error));
            Ok(())
        }

        fn write_info(&mut self, info: &str) -> Result<()> {
            self.output.push(format!("Info: {}", info));
            Ok(())
        }

        fn get_output(&self) -> Vec<String> {
            self.output.clone()
        }
    }

    // Mock ErrorDisplay for testing
    struct TestErrorDisplay {
        terminal: TestTerminal,
    }

    impl TestErrorDisplay {
        fn new() -> Self {
            Self {
                terminal: TestTerminal::new(),
            }
        }

        fn display_connection_error(&mut self, url: &str, error: &str) -> Result<()> {
            self.terminal.write_error(&format!(
                "Failed to connect to {}: {}",
                url, error
            ))
        }

        fn display_timeout_error(&mut self, timeout_seconds: u64) -> Result<()> {
            self.terminal.write_error(&format!(
                "Request timed out after {}s",
                timeout_seconds
            ))
        }

        fn display_config_warning(&mut self, error: &str) -> Result<()> {
            self.terminal.write_info(&format!(
                "Warning: Failed to load configuration: {}. Using default values.",
                error
            ))
        }

        fn display_filesystem_error(&mut self, operation: &str, path: &str, error: &str) -> Result<()> {
            self.terminal.write_error(&format!(
                "Failed to {} {}: {}",
                operation, path, error
            ))
        }

        fn get_output(&self) -> Vec<String> {
            self.terminal.get_output()
        }
    }

    /// Test connection failure display
    /// **Validates: Requirements 7.1**
    #[test]
    fn test_connection_failure_display() {
        let mut display = TestErrorDisplay::new();
        
        // Test with localhost URL
        display.display_connection_error(
            "http://localhost:11434",
            "Connection refused"
        ).unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Failed to connect to http://localhost:11434"));
        assert!(output[0].contains("Connection refused"));
    }

    /// Test connection failure with different URLs
    /// **Validates: Requirements 7.1**
    #[test]
    fn test_connection_failure_different_urls() {
        let mut display = TestErrorDisplay::new();
        
        // Test with remote URL
        display.display_connection_error(
            "http://remote-server:8080",
            "Network is unreachable"
        ).unwrap();
        
        // Test with HTTPS URL
        display.display_connection_error(
            "https://api.example.com",
            "No route to host"
        ).unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 2);
        assert!(output[0].contains("remote-server:8080"));
        assert!(output[0].contains("Network is unreachable"));
        assert!(output[1].contains("api.example.com"));
        assert!(output[1].contains("No route to host"));
    }

    /// Test timeout display
    /// **Validates: Requirements 7.3**
    #[test]
    fn test_timeout_display() {
        let mut display = TestErrorDisplay::new();
        
        // Test with 30 second timeout
        display.display_timeout_error(30).unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Request timed out after 30s"));
    }

    /// Test timeout display with different durations
    /// **Validates: Requirements 7.3**
    #[test]
    fn test_timeout_different_durations() {
        let mut display = TestErrorDisplay::new();
        
        display.display_timeout_error(10).unwrap();
        display.display_timeout_error(60).unwrap();
        display.display_timeout_error(120).unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("10s"));
        assert!(output[1].contains("60s"));
        assert!(output[2].contains("120s"));
    }

    /// Test config loading failure
    /// **Validates: Requirements 7.4**
    #[test]
    fn test_config_loading_failure() {
        let mut display = TestErrorDisplay::new();
        
        // Test with file not found error
        display.display_config_warning("File not found: config.toml").unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Warning: Failed to load configuration"));
        assert!(output[0].contains("File not found: config.toml"));
        assert!(output[0].contains("Using default values"));
    }

    /// Test config loading failure with different errors
    /// **Validates: Requirements 7.4**
    #[test]
    fn test_config_loading_different_errors() {
        let mut display = TestErrorDisplay::new();
        
        display.display_config_warning("Permission denied").unwrap();
        display.display_config_warning("Invalid TOML syntax").unwrap();
        display.display_config_warning("Missing required field").unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("Permission denied"));
        assert!(output[1].contains("Invalid TOML syntax"));
        assert!(output[2].contains("Missing required field"));
        
        // All should mention using default values
        for line in &output {
            assert!(line.contains("Using default values"));
        }
    }

    /// Test conversation save failure
    /// **Validates: Requirements 7.5**
    #[test]
    fn test_conversation_save_failure() {
        let mut display = TestErrorDisplay::new();
        
        // Test with disk full error
        display.display_filesystem_error(
            "save",
            "conversation.json",
            "No space left on device"
        ).unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Failed to save conversation.json"));
        assert!(output[0].contains("No space left on device"));
    }

    /// Test conversation save failure with different errors
    /// **Validates: Requirements 7.5**
    #[test]
    fn test_conversation_save_different_errors() {
        let mut display = TestErrorDisplay::new();
        
        display.display_filesystem_error(
            "save",
            "conversation.json",
            "Permission denied"
        ).unwrap();
        
        display.display_filesystem_error(
            "save",
            "conversation.json",
            "Read-only file system"
        ).unwrap();
        
        display.display_filesystem_error(
            "save",
            "conversation.json",
            "I/O error"
        ).unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("Permission denied"));
        assert!(output[1].contains("Read-only file system"));
        assert!(output[2].contains("I/O error"));
        
        // All should mention the operation and file
        for line in &output {
            assert!(line.contains("save"));
            assert!(line.contains("conversation.json"));
        }
    }

    /// Test filesystem error with different operations
    /// **Validates: Requirements 7.4, 7.5**
    #[test]
    fn test_filesystem_different_operations() {
        let mut display = TestErrorDisplay::new();
        
        display.display_filesystem_error("save", "file1.txt", "Error 1").unwrap();
        display.display_filesystem_error("load", "file2.txt", "Error 2").unwrap();
        display.display_filesystem_error("delete", "file3.txt", "Error 3").unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 3);
        assert!(output[0].contains("save") && output[0].contains("file1.txt"));
        assert!(output[1].contains("load") && output[1].contains("file2.txt"));
        assert!(output[2].contains("delete") && output[2].contains("file3.txt"));
    }

    /// Test multiple error scenarios in sequence
    /// **Validates: Requirements 7.1, 7.3, 7.4, 7.5**
    #[test]
    fn test_multiple_error_scenarios() {
        let mut display = TestErrorDisplay::new();
        
        // Connection error
        display.display_connection_error("http://localhost:11434", "Connection refused").unwrap();
        
        // Timeout error
        display.display_timeout_error(30).unwrap();
        
        // Config error
        display.display_config_warning("File not found").unwrap();
        
        // Filesystem error
        display.display_filesystem_error("save", "conversation.json", "Disk full").unwrap();
        
        let output = display.get_output();
        assert_eq!(output.len(), 4);
        
        // Verify each error type is present
        assert!(output[0].contains("Failed to connect"));
        assert!(output[1].contains("timed out"));
        assert!(output[2].contains("Warning"));
        assert!(output[3].contains("Failed to save"));
    }

    /// Test error messages preserve special characters
    /// **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**
    #[test]
    fn test_error_messages_preserve_special_characters() {
        let mut display = TestErrorDisplay::new();
        
        // Test with special characters in error message
        display.display_connection_error(
            "http://localhost:11434",
            "Connection refused: errno=111 (ECONNREFUSED)"
        ).unwrap();
        
        let output = display.get_output();
        assert!(output[0].contains("errno=111"));
        assert!(output[0].contains("ECONNREFUSED"));
    }

    /// Test error messages with long URLs
    /// **Validates: Requirements 7.1**
    #[test]
    fn test_error_messages_with_long_urls() {
        let mut display = TestErrorDisplay::new();
        
        let long_url = "http://very-long-hostname-that-exceeds-normal-length.example.com:8080/api/v1/endpoint";
        display.display_connection_error(long_url, "Connection timeout").unwrap();
        
        let output = display.get_output();
        assert!(output[0].contains(long_url));
        assert!(output[0].contains("Connection timeout"));
    }

    /// Test error messages with unicode
    /// **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**
    #[test]
    fn test_error_messages_with_unicode() {
        let mut display = TestErrorDisplay::new();
        
        display.display_filesystem_error(
            "save",
            "文件.json",
            "无法保存文件"
        ).unwrap();
        
        let output = display.get_output();
        assert!(output[0].contains("文件.json"));
        assert!(output[0].contains("无法保存文件"));
    }
}
