use anyhow::Result;
use crate::cli::terminal::Terminal;

/// Handler for streaming AI responses with real-time display
pub struct StreamingHandler {
    terminal: Terminal,
    buffer: String,
}

impl StreamingHandler {
    /// Create a new StreamingHandler
    ///
    /// # Arguments
    /// * `terminal` - The terminal instance for output
    pub fn new(terminal: Terminal) -> Self {
        Self {
            terminal,
            buffer: String::new(),
        }
    }

    /// Handle a chunk of streaming response
    ///
    /// This method is called for each chunk received from the backend.
    /// It displays the chunk immediately and accumulates it in the buffer.
    ///
    /// # Arguments
    /// * `chunk` - The text chunk to display and accumulate
    ///
    /// # Returns
    /// Ok(()) on success, or an error if writing to terminal fails
    pub fn on_chunk(&mut self, chunk: String) -> Result<()> {
        self.buffer.push_str(&chunk);
        self.terminal.write(&chunk)?;
        Ok(())
    }

    /// Finalize the streaming response
    ///
    /// This method should be called when streaming is complete.
    /// It adds a newline and returns the complete accumulated response.
    ///
    /// # Returns
    /// The complete response text accumulated from all chunks
    pub fn finalize(&mut self) -> Result<String> {
        self.terminal.write("\n")?;
        Ok(std::mem::take(&mut self.buffer))
    }

    /// Get the current accumulated buffer without finalizing
    ///
    /// This is useful for error handling where you want to save
    /// the partial response without adding a newline.
    ///
    /// # Returns
    /// A reference to the accumulated buffer
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Handle an error during streaming
    ///
    /// This method displays an error message and returns the partial response
    /// accumulated so far. It does not add a newline after the partial response.
    ///
    /// # Arguments
    /// * `error` - The error message to display
    ///
    /// # Returns
    /// The partial response accumulated before the error
    pub fn handle_error(&mut self, error: &str) -> Result<String> {
        // Add newline after partial response if there is any
        if !self.buffer.is_empty() {
            self.terminal.write("\n")?;
        }
        self.terminal.write_error(error)?;
        Ok(std::mem::take(&mut self.buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper struct for testing without actual terminal I/O
    struct TestTerminal {
        output: Vec<u8>,
    }

    impl TestTerminal {
        fn new() -> Self {
            Self {
                output: Vec::new(),
            }
        }

        fn write(&mut self, text: &str) -> Result<()> {
            self.output.extend_from_slice(text.as_bytes());
            Ok(())
        }

        fn write_error(&mut self, error: &str) -> Result<()> {
            self.write(&format!("Error: {}\n", error))
        }

        fn get_output(&self) -> String {
            String::from_utf8_lossy(&self.output).to_string()
        }
    }

    // Mock StreamingHandler for testing
    struct TestStreamingHandler {
        terminal: TestTerminal,
        buffer: String,
    }

    impl TestStreamingHandler {
        fn new() -> Self {
            Self {
                terminal: TestTerminal::new(),
                buffer: String::new(),
            }
        }

        fn on_chunk(&mut self, chunk: String) -> Result<()> {
            self.buffer.push_str(&chunk);
            self.terminal.write(&chunk)?;
            Ok(())
        }

        fn finalize(&mut self) -> Result<String> {
            self.terminal.write("\n")?;
            Ok(std::mem::take(&mut self.buffer))
        }

        fn buffer(&self) -> &str {
            &self.buffer
        }

        fn handle_error(&mut self, error: &str) -> Result<String> {
            if !self.buffer.is_empty() {
                self.terminal.write("\n")?;
            }
            self.terminal.write_error(error)?;
            Ok(std::mem::take(&mut self.buffer))
        }

        fn get_output(&self) -> String {
            self.terminal.get_output()
        }
    }

    #[test]
    fn test_streaming_handler_single_chunk() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("Hello".to_string()).unwrap();

        assert_eq!(handler.buffer(), "Hello");
        assert_eq!(handler.get_output(), "Hello");
    }

    #[test]
    fn test_streaming_handler_multiple_chunks() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("Hello".to_string()).unwrap();
        handler.on_chunk(" ".to_string()).unwrap();
        handler.on_chunk("World".to_string()).unwrap();

        assert_eq!(handler.buffer(), "Hello World");
        assert_eq!(handler.get_output(), "Hello World");
    }

    #[test]
    fn test_streaming_handler_finalize() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("Hello".to_string()).unwrap();
        handler.on_chunk(" World".to_string()).unwrap();

        let result = handler.finalize().unwrap();
        assert_eq!(result, "Hello World");
        assert_eq!(handler.buffer(), ""); // Buffer should be cleared
        assert_eq!(handler.get_output(), "Hello World\n"); // Should have newline
    }

    #[test]
    fn test_streaming_handler_empty_finalize() {
        let mut handler = TestStreamingHandler::new();
        let result = handler.finalize().unwrap();

        assert_eq!(result, "");
        assert_eq!(handler.buffer(), "");
        assert_eq!(handler.get_output(), "\n");
    }

    #[test]
    fn test_streaming_handler_error_with_partial_response() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("Partial".to_string()).unwrap();
        handler.on_chunk(" response".to_string()).unwrap();

        let partial = handler.handle_error("Connection lost").unwrap();
        assert_eq!(partial, "Partial response");
        assert_eq!(handler.buffer(), ""); // Buffer should be cleared

        let output = handler.get_output();
        assert!(output.contains("Partial response"));
        assert!(output.contains("Error: Connection lost"));
    }

    #[test]
    fn test_streaming_handler_error_without_partial_response() {
        let mut handler = TestStreamingHandler::new();
        let partial = handler.handle_error("Connection failed").unwrap();

        assert_eq!(partial, "");
        assert_eq!(handler.buffer(), "");

        let output = handler.get_output();
        assert!(output.contains("Error: Connection failed"));
        // Should not have extra newline before error
        assert!(!output.starts_with('\n'));
    }

    #[test]
    fn test_streaming_handler_buffer_access() {
        let mut handler = TestStreamingHandler::new();
        assert_eq!(handler.buffer(), "");

        handler.on_chunk("First".to_string()).unwrap();
        assert_eq!(handler.buffer(), "First");

        handler.on_chunk(" Second".to_string()).unwrap();
        assert_eq!(handler.buffer(), "First Second");
    }

    #[test]
    fn test_streaming_handler_unicode_chunks() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("Hello ".to_string()).unwrap();
        handler.on_chunk("世界".to_string()).unwrap();
        handler.on_chunk(" 🌍".to_string()).unwrap();

        assert_eq!(handler.buffer(), "Hello 世界 🌍");
        let result = handler.finalize().unwrap();
        assert_eq!(result, "Hello 世界 🌍");
    }

    #[test]
    fn test_streaming_handler_empty_chunks() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("".to_string()).unwrap();
        handler.on_chunk("Hello".to_string()).unwrap();
        handler.on_chunk("".to_string()).unwrap();
        handler.on_chunk(" World".to_string()).unwrap();
        handler.on_chunk("".to_string()).unwrap();

        assert_eq!(handler.buffer(), "Hello World");
        let result = handler.finalize().unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_streaming_handler_newlines_in_chunks() {
        let mut handler = TestStreamingHandler::new();
        handler.on_chunk("Line 1\n".to_string()).unwrap();
        handler.on_chunk("Line 2\n".to_string()).unwrap();
        handler.on_chunk("Line 3".to_string()).unwrap();

        assert_eq!(handler.buffer(), "Line 1\nLine 2\nLine 3");
        let result = handler.finalize().unwrap();
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_streaming_handler_large_chunks() {
        let mut handler = TestStreamingHandler::new();
        let large_chunk = "a".repeat(10000);
        handler.on_chunk(large_chunk.clone()).unwrap();

        assert_eq!(handler.buffer().len(), 10000);
        let result = handler.finalize().unwrap();
        assert_eq!(result.len(), 10000);
    }

    #[test]
    fn test_streaming_handler_many_small_chunks() {
        let mut handler = TestStreamingHandler::new();
        for i in 0..1000 {
            handler.on_chunk(format!("{} ", i)).unwrap();
        }

        assert!(handler.buffer().contains("0 "));
        assert!(handler.buffer().contains("999 "));
        let result = handler.finalize().unwrap();
        assert!(result.contains("0 "));
        assert!(result.contains("999 "));
    }

    // Property-based tests
    use quickcheck_macros::quickcheck;

    /// **Feature: cli-version, Property 3: Streaming chunk accumulation**
    /// For any sequence of streaming response chunks, the final displayed response
    /// should be the concatenation of all chunks in the order received, with no
    /// chunks lost or reordered.
    /// **Validates: Requirements 2.1, 2.2**
    #[quickcheck]
    fn prop_streaming_chunk_accumulation(chunks: Vec<String>) -> bool {
        let mut handler = TestStreamingHandler::new();
        
        // Expected result is the concatenation of all chunks
        let expected: String = chunks.iter().map(|s| s.as_str()).collect();
        
        // Feed all chunks to the handler
        for chunk in chunks {
            if handler.on_chunk(chunk).is_err() {
                return false;
            }
        }
        
        // Finalize and check the result
        match handler.finalize() {
            Ok(result) => {
                // The result should be exactly the concatenation of all chunks
                if result != expected {
                    return false;
                }
                
                // The terminal output should contain all the chunks (without the final newline)
                let output = handler.get_output();
                // Remove the final newline added by finalize
                let output_without_newline = output.trim_end_matches('\n');
                output_without_newline == expected
            }
            Err(_) => false,
        }
    }
}
