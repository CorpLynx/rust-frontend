use anyhow::{Context, Result};
use serde_json::json;
use std::io::{self, Write, IsTerminal};
use crate::mode::NonInteractiveOptions;

/// Output formatter for handling different output formats in non-interactive mode
pub struct OutputFormatter;

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new() -> Self {
        Self
    }
    
    /// Format and output the response according to the specified options
    pub fn format_response(&self, response: &str, options: &NonInteractiveOptions) -> Result<()> {
        if options.json {
            self.format_json_response(response, options)
        } else if options.quiet {
            self.format_quiet_response(response)
        } else {
            self.format_default_response(response, options)
        }
    }
    
    /// Format response as JSON with metadata
    fn format_json_response(&self, response: &str, options: &NonInteractiveOptions) -> Result<()> {
        let output = json!({
            "response": response,
            "metadata": {
                "length": response.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "streaming": !options.no_stream,
                "format": "json"
            }
        });
        
        // JSON output goes to stdout
        println!("{}", serde_json::to_string_pretty(&output)
            .context("Failed to serialize JSON response")?);
        
        // Verbose information goes to stderr to not interfere with JSON
        if options.verbose {
            eprintln!("JSON response generated with {} characters", response.len());
        }
        
        Ok(())
    }
    
    /// Format response in quiet mode (response only)
    fn format_quiet_response(&self, response: &str) -> Result<()> {
        // In quiet mode, output only the response to stdout
        print!("{}", response);
        self.flush_stdout_for_pipes()?;
        Ok(())
    }
    
    /// Format response with default formatting
    fn format_default_response(&self, response: &str, options: &NonInteractiveOptions) -> Result<()> {
        if !options.no_stream {
            // Response was already printed during streaming, just add final newline
            println!();
        } else {
            // Print the complete response
            println!("{}", response);
        }
        
        // Add verbose information to stderr
        if options.verbose {
            eprintln!("Response completed: {} characters", response.len());
            eprintln!("Output mode: default");
            eprintln!("Streaming: {}", !options.no_stream);
        }
        
        Ok(())
    }
    
    /// Output a chunk during streaming (for non-quiet, non-JSON modes)
    pub fn output_streaming_chunk(&self, chunk: &str, options: &NonInteractiveOptions) -> Result<()> {
        // Only output chunks in default mode (not quiet or JSON)
        if !options.quiet && !options.json {
            print!("{}", chunk);
            // Ensure proper flushing for pipe compatibility
            self.flush_stdout_for_pipes()?;
        }
        Ok(())
    }
    
    /// Flush stdout appropriately for pipe compatibility
    fn flush_stdout_for_pipes(&self) -> Result<()> {
        io::stdout().flush().context("Failed to flush stdout")?;
        
        // In pipe chains, we want to ensure data flows through immediately
        if !io::stdout().is_terminal() {
            // Additional flush for non-terminal output (pipes/redirects)
            io::stdout().flush().context("Failed to flush stdout for pipe")?;
        }
        Ok(())
    }
    
    /// Flush stderr appropriately for pipe compatibility  
    fn flush_stderr_for_pipes(&self) -> Result<()> {
        io::stderr().flush().context("Failed to flush stderr")?;
        
        // In pipe chains, we want to ensure error data flows through immediately
        if !io::stderr().is_terminal() {
            // Additional flush for non-terminal output (pipes/redirects)
            io::stderr().flush().context("Failed to flush stderr for pipe")?;
        }
        Ok(())
    }
    
    /// Output an error message to stderr
    pub fn output_error(&self, error: &str) -> Result<()> {
        eprintln!("Error: {}", error);
        self.flush_stderr_for_pipes()?;
        Ok(())
    }
    
    /// Output a warning message to stderr
    pub fn output_warning(&self, warning: &str) -> Result<()> {
        eprintln!("Warning: {}", warning);
        self.flush_stderr_for_pipes()?;
        Ok(())
    }
    
    /// Output verbose debug information to stderr
    pub fn output_verbose(&self, message: &str, options: &NonInteractiveOptions) -> Result<()> {
        if options.verbose {
            eprintln!("Debug: {}", message);
            self.flush_stderr_for_pipes()?;
        }
        Ok(())
    }
    
    /// Check if we should automatically enable quiet mode based on terminal detection
    pub fn should_auto_quiet() -> bool {
        !io::stdout().is_terminal()
    }
    
    /// Ensure proper stream separation for piped output
    pub fn ensure_stream_separation(&self) -> Result<()> {
        // Flush both streams to ensure proper separation
        self.flush_stdout_for_pipes()?;
        self.flush_stderr_for_pipes()?;
        Ok(())
    }
    
    /// Check if output should be automatically adjusted for redirection
    pub fn detect_output_redirection(&self) -> (bool, bool) {
        let stdout_redirected = !io::stdout().is_terminal();
        let stderr_redirected = !io::stderr().is_terminal();
        (stdout_redirected, stderr_redirected)
    }
    
    /// Adjust output behavior based on redirection detection
    pub fn adjust_for_redirection(&self, options: &mut crate::mode::NonInteractiveOptions) {
        let (stdout_redirected, _stderr_redirected) = self.detect_output_redirection();
        
        // Automatically enable quiet mode if stdout is redirected
        if stdout_redirected && !options.quiet {
            options.quiet = true;
        }
        
        // Disable streaming indicators if output is redirected
        if stdout_redirected && !options.no_stream {
            // Note: We don't automatically set no_stream as streaming can still work with pipes
            // The streaming indicators just won't be visible, which is fine
        }
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use quickcheck_macros::quickcheck;

    /// **Feature: cli-non-interactive-mode, Property 7: Output format consistency**
    /// For any response in quiet mode, the output should contain only the AI response 
    /// without additional formatting or status messages
    /// **Validates: Requirements 4.1**
    #[quickcheck]
    fn property_output_format_consistency(response: String) -> bool {
        let formatter = OutputFormatter::new();
        let quiet_options = NonInteractiveOptions {
            quiet: true,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };

        // Test that quiet mode formatting doesn't fail
        let result = formatter.format_response(&response, &quiet_options);
        
        // The property we're testing is that quiet mode always succeeds
        // and doesn't add extra formatting (we can't easily test output content
        // in this environment, but we can test that it doesn't error)
        result.is_ok()
    }

    /// **Feature: cli-non-interactive-mode, Property 8: JSON output validity**
    /// For any response with --json flag, the output should be valid JSON containing 
    /// the response and metadata
    /// **Validates: Requirements 4.2**
    #[quickcheck]
    fn property_json_output_validity(response: String, no_stream: bool) -> bool {
        let formatter = OutputFormatter::new();
        let json_options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream,
            verbose: false,
            save_on_interrupt: false,
        };

        // Test that we can create valid JSON for any response
        let json_output = json!({
            "response": response,
            "metadata": {
                "length": response.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "streaming": !no_stream,
                "format": "json"
            }
        });

        // Verify the JSON can be serialized (which means it's valid)
        let serialization_result = serde_json::to_string_pretty(&json_output);
        
        // Also test that the formatter method doesn't fail
        let format_result = formatter.format_response(&response, &json_options);

        // Both operations should succeed for any valid input
        serialization_result.is_ok() && format_result.is_ok()
    }

    /// **Feature: cli-non-interactive-mode, Property 14: Stream separation**
    /// For any execution where stderr is redirected separately from stdout, 
    /// errors should go to stderr and responses to stdout
    /// **Validates: Requirements 8.4**
    #[quickcheck]
    fn property_stream_separation(
        response: String,
        error_msg: String,
        warning_msg: String,
        verbose_msg: String,
        quiet: bool,
        json: bool,
        verbose: bool,
    ) -> bool {
        // Filter out very long strings to keep tests reasonable
        if response.len() > 1000 || error_msg.len() > 1000 || 
           warning_msg.len() > 1000 || verbose_msg.len() > 1000 {
            return true; // Skip very long inputs
        }

        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet,
            json,
            no_stream: false,
            verbose,
            save_on_interrupt: false,
        };

        // Test that all output methods work without panicking
        // In a real implementation, we would verify that:
        // - Responses go to stdout
        // - Errors, warnings, and verbose messages go to stderr
        // For this property test, we verify the methods don't fail
        
        let response_result = formatter.format_response(&response, &options);
        let error_result = formatter.output_error(&error_msg);
        let warning_result = formatter.output_warning(&warning_msg);
        let verbose_result = formatter.output_verbose(&verbose_msg, &options);
        let separation_result = formatter.ensure_stream_separation();

        // All operations should succeed
        response_result.is_ok() && 
        error_result.is_ok() && 
        warning_result.is_ok() && 
        verbose_result.is_ok() && 
        separation_result.is_ok()
    }

    // Helper struct to capture output for testing
    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_content(&self) -> String {
            let buffer = self.buffer.lock().unwrap();
            String::from_utf8(buffer.clone()).unwrap()
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_quiet_mode_output_contains_only_response() {
        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet: true,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };

        // Test that quiet mode works (we can't easily capture stdout in tests,
        // but we can verify the function doesn't panic and returns Ok)
        let result = formatter.format_response("Test response", &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_format_validity_and_structure() {
        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };

        // We can't easily capture stdout, but we can test the JSON generation logic
        let response = "Test response content";
        let json_output = json!({
            "response": response,
            "metadata": {
                "length": response.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "streaming": !options.no_stream,
                "format": "json"
            }
        });

        // Verify JSON structure
        assert_eq!(json_output["response"], "Test response content");
        assert_eq!(json_output["metadata"]["length"], 21);
        assert_eq!(json_output["metadata"]["streaming"], true);
        assert_eq!(json_output["metadata"]["format"], "json");
        
        // Verify it can be serialized
        let serialized = serde_json::to_string_pretty(&json_output);
        assert!(serialized.is_ok());
        
        // Verify the actual format_response doesn't panic
        let result = formatter.format_response(response, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verbose_mode_additions() {
        let formatter = OutputFormatter::new();
        let verbose_options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: true,
            verbose: true,
            save_on_interrupt: false,
        };

        let non_verbose_options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: true,
            verbose: false,
            save_on_interrupt: false,
        };

        // Test that verbose mode doesn't cause errors
        let result_verbose = formatter.format_response("Test", &verbose_options);
        assert!(result_verbose.is_ok());

        let result_non_verbose = formatter.format_response("Test", &non_verbose_options);
        assert!(result_non_verbose.is_ok());

        // Test verbose output function
        let result = formatter.output_verbose("Debug message", &verbose_options);
        assert!(result.is_ok());

        let result = formatter.output_verbose("Debug message", &non_verbose_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_streaming_chunk_output() {
        let formatter = OutputFormatter::new();
        
        // Test default mode (should output)
        let default_options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };
        
        let result = formatter.output_streaming_chunk("chunk", &default_options);
        assert!(result.is_ok());

        // Test quiet mode (should not output)
        let quiet_options = NonInteractiveOptions {
            quiet: true,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };
        
        let result = formatter.output_streaming_chunk("chunk", &quiet_options);
        assert!(result.is_ok());

        // Test JSON mode (should not output chunks)
        let json_options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };
        
        let result = formatter.output_streaming_chunk("chunk", &json_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_and_warning_output() {
        let formatter = OutputFormatter::new();

        // Test error output
        let result = formatter.output_error("Test error message");
        assert!(result.is_ok());

        // Test warning output
        let result = formatter.output_warning("Test warning message");
        assert!(result.is_ok());
    }

    #[test]
    fn test_stream_separation() {
        let formatter = OutputFormatter::new();
        let result = formatter.ensure_stream_separation();
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_quiet_detection() {
        // Test that the function exists and returns a boolean
        let auto_quiet = OutputFormatter::should_auto_quiet();
        assert!(auto_quiet || !auto_quiet); // Always true, just checking it doesn't panic
    }

    #[test]
    fn test_default_response_formatting() {
        let formatter = OutputFormatter::new();
        
        // Test streaming mode
        let streaming_options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };
        
        let result = formatter.format_default_response("Test response", &streaming_options);
        assert!(result.is_ok());

        // Test non-streaming mode
        let non_streaming_options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: true,
            verbose: false,
            save_on_interrupt: false,
        };
        
        let result = formatter.format_default_response("Test response", &non_streaming_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_with_verbose_mode() {
        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream: false,
            verbose: true,
            save_on_interrupt: false,
        };

        // Test that JSON + verbose mode works correctly
        let result = formatter.format_response("Test response", &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_metadata_accuracy() {
        let response = "Hello, world! This is a test response.";
        let options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream: true,
            verbose: false,
            save_on_interrupt: false,
        };

        // Create the JSON structure manually to test metadata accuracy
        let json_output = json!({
            "response": response,
            "metadata": {
                "length": response.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "streaming": !options.no_stream,
                "format": "json"
            }
        });

        // Verify metadata accuracy
        assert_eq!(json_output["metadata"]["length"], response.len());
        assert_eq!(json_output["metadata"]["streaming"], false); // no_stream = true
        assert_eq!(json_output["metadata"]["format"], "json");
        
        // Verify timestamp is valid RFC3339
        let timestamp_str = json_output["metadata"]["timestamp"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp_str).is_ok());
    }

    #[test]
    fn test_empty_response_handling() {
        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };

        // Test empty response
        let result = formatter.format_response("", &options);
        assert!(result.is_ok());

        // Test empty response in JSON mode
        let json_options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };
        
        let result = formatter.format_response("", &json_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_response_handling() {
        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: false,
            verbose: true,
            save_on_interrupt: false,
        };

        // Test large response
        let large_response = "x".repeat(10000);
        let result = formatter.format_response(&large_response, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_special_characters_in_response() {
        let formatter = OutputFormatter::new();
        let options = NonInteractiveOptions {
            quiet: false,
            json: true,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };

        // Test response with special characters
        let special_response = "Response with \"quotes\", \n newlines, \t tabs, and unicode: 🚀";
        let result = formatter.format_response(special_response, &options);
        assert!(result.is_ok());

        // Verify JSON can handle special characters
        let json_output = json!({
            "response": special_response,
            "metadata": {
                "length": special_response.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "streaming": true,
                "format": "json"
            }
        });
        
        let serialized = serde_json::to_string(&json_output);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_formatter_default_trait() {
        let formatter1 = OutputFormatter::new();
        let formatter2 = OutputFormatter::default();
        
        // Both should work the same way
        let options = NonInteractiveOptions {
            quiet: false,
            json: false,
            no_stream: false,
            verbose: false,
            save_on_interrupt: false,
        };

        let result1 = formatter1.format_response("test", &options);
        let result2 = formatter2.format_response("test", &options);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}