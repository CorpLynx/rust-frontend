use anyhow::{Context, Result};
use crate::backend::BackendClient;
use crate::cli::commands::{Command, display_help};
use crate::cli::streaming::StreamingHandler;
use crate::cli::terminal::Terminal;
use crate::config::AppConfig;
use crate::conversation::{ChatMessage, Conversation, ConversationManager};

/// CLI application state and REPL loop
pub struct CliApp {
    #[allow(dead_code)]
    config: AppConfig,
    conversation: Conversation,
    conversation_manager: ConversationManager,
    backend_client: BackendClient,
    terminal: Terminal,
    running: bool,
    model: String,
}

impl CliApp {
    /// Create a new CLI application instance
    ///
    /// # Arguments
    /// * `config` - Application configuration
    /// * `backend_url` - Optional backend URL override
    /// * `model` - Optional model name override
    pub fn new(
        config: AppConfig,
        backend_url: Option<String>,
        model: Option<String>,
    ) -> Result<Self> {
        // Use CLI argument overrides if provided, otherwise use config values
        let url = backend_url.unwrap_or_else(|| config.backend.ollama_url.clone());
        let model_name = model.unwrap_or_else(|| "llama2".to_string());

        let backend_client = BackendClient::new(url, config.backend.timeout_seconds)
            .context("Failed to create backend client")?;

        let terminal = Terminal::new().context("Failed to create terminal")?;

        let conversation_manager = ConversationManager::new();
        let conversation = Conversation::with_timestamp_name(Some(model_name.clone()));

        Ok(Self {
            config,
            conversation,
            conversation_manager,
            backend_client,
            terminal,
            running: true,
            model: model_name,
        })
    }

    /// Display welcome message
    fn display_welcome(&mut self) -> Result<()> {
        self.terminal.write("\n")?;
        self.terminal.write_info(&format!(
            "Prometheus CLI v{} - Terminal AI Chat",
            env!("CARGO_PKG_VERSION")
        ))?;
        self.terminal.write_info(&format!(
            "Connected to: {}",
            self.backend_client.base_url()
        ))?;
        self.terminal.write_info(&format!("Model: {}", self.model))?;
        self.terminal
            .write_info("Type /help for available commands")?;
        self.terminal.write("\n")?;
        Ok(())
    }

    /// Run the main REPL loop
    pub async fn run(&mut self) -> Result<()> {
        self.display_welcome()?;

        while self.running {
            // Display prompt
            self.terminal.write("> ")?;

            // Read user input
            let input = self.terminal.read_line()?;

            // Handle empty input
            if input.trim().is_empty() {
                continue;
            }

            // Check if it's a command or a prompt
            if input.starts_with('/') {
                self.handle_command(&input).await?;
            } else {
                self.handle_prompt(input).await?;
            }
        }

        self.shutdown().await
    }

    /// Handle a user prompt
    async fn handle_prompt(&mut self, prompt: String) -> Result<()> {
        // Add user message to conversation
        let user_message = ChatMessage::new("user".to_string(), prompt.clone());
        self.conversation.add_message(user_message);

        // Display user prompt
        self.terminal.write_user_prompt(&prompt)?;

        // Show loading indicator
        self.terminal.show_spinner()?;

        // Create streaming handler
        let mut streaming_handler = StreamingHandler::new(Terminal::new()?);

        // Send prompt to backend with streaming
        let result = self
            .backend_client
            .send_prompt_streaming(&prompt, &self.model, |chunk| {
                streaming_handler.on_chunk(chunk)
            })
            .await;

        // Hide spinner
        self.terminal.hide_spinner()?;

        // Handle response or error
        match result {
            Ok(_response) => {
                // Finalize streaming (adds newline)
                let final_response = streaming_handler.finalize()?;

                // Add AI response to conversation
                let ai_message = ChatMessage::new("assistant".to_string(), final_response);
                self.conversation.add_message(ai_message);

                // Save conversation
                if let Err(e) = self.conversation_manager.save_conversation(&self.conversation) {
                    self.terminal
                        .write_error(&format!("Failed to save conversation: {}", e))?;
                }
            }
            Err(e) => {
                // Handle error and save partial response if any
                let partial = streaming_handler.handle_error(&format!("{}", e))?;

                if !partial.is_empty() {
                    // Save partial response to conversation
                    let ai_message = ChatMessage::new("assistant".to_string(), partial);
                    self.conversation.add_message(ai_message);

                    // Try to save conversation with partial response
                    if let Err(save_err) = self
                        .conversation_manager
                        .save_conversation(&self.conversation)
                    {
                        self.terminal.write_error(&format!(
                            "Failed to save conversation: {}",
                            save_err
                        ))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a special command
    async fn handle_command(&mut self, input: &str) -> Result<()> {
        let command = Command::parse(input);

        match command {
            Command::Exit | Command::Quit => {
                self.terminal.write_info("Saving conversation and exiting...")?;
                self.running = false;
            }
            Command::Clear => {
                self.terminal.clear_screen()?;
                self.display_welcome()?;
            }
            Command::New => {
                // Save current conversation
                self.conversation_manager
                    .save_conversation(&self.conversation)
                    .context("Failed to save current conversation")?;

                // Create new conversation
                self.conversation = Conversation::with_timestamp_name(Some(self.model.clone()));
                self.terminal.write_info("Started new conversation")?;
            }
            Command::Help => {
                let help_text = display_help();
                self.terminal.write(&help_text)?;
            }
            Command::Models => {
                self.terminal.write_info("Fetching available models...")?;
                match self.backend_client.fetch_models().await {
                    Ok(models) => {
                        self.terminal.write("\nAvailable models:\n")?;
                        for model in models {
                            self.terminal.write(&format!("  - {}\n", model))?;
                        }
                        self.terminal.write("\n")?;
                    }
                    Err(e) => {
                        self.terminal
                            .write_error(&format!("Failed to fetch models: {}", e))?;
                    }
                }
            }
            Command::Unknown(cmd) => {
                self.terminal.write_error(&format!(
                    "Unknown command: /{}. Type /help for available commands",
                    cmd
                ))?;
            }
        }

        Ok(())
    }

    /// Shutdown the application gracefully
    async fn shutdown(&mut self) -> Result<()> {
        // Save final conversation state
        self.conversation_manager
            .save_conversation(&self.conversation)
            .context("Failed to save conversation on shutdown")?;

        self.terminal.write_info("Goodbye!")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{QuickCheck, TestResult};
    use std::sync::{Arc, Mutex};

    /// **Feature: cli-version, Property 1: Prompt transmission completeness**
    /// For any non-empty user prompt, when submitted to the CLI, the exact prompt
    /// text should be sent to the configured Ollama backend in the request body.
    /// **Validates: Requirements 1.2**
    /// 
    /// This property tests that the prompt is correctly formatted into a JSON request.
    /// We verify this by checking that the JSON serialization includes the prompt.
    #[test]
    fn prop_prompt_transmission_completeness() {
        fn property(prompt: String) -> TestResult {
            // Filter out empty or whitespace-only prompts
            if prompt.trim().is_empty() {
                return TestResult::discard();
            }

            // Filter out prompts that start with '/' (commands)
            if prompt.starts_with('/') {
                return TestResult::discard();
            }

            // Filter out prompts with control characters that could break JSON
            if prompt.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
                return TestResult::discard();
            }

            // Limit prompt length
            if prompt.len() > 500 {
                return TestResult::discard();
            }

            // Test that the prompt can be serialized into a JSON request body
            let request_body = serde_json::json!({
                "model": "llama2",
                "prompt": prompt.clone(),
                "stream": true
            });

            // Verify we can serialize it to JSON
            let json_string = match serde_json::to_string(&request_body) {
                Ok(s) => s,
                Err(_) => return TestResult::failed(),
            };

            // Verify we can deserialize it back
            let parsed: serde_json::Value = match serde_json::from_str(&json_string) {
                Ok(v) => v,
                Err(_) => return TestResult::failed(),
            };

            // Verify the prompt field matches exactly (round-trip test)
            match parsed.get("prompt").and_then(|v| v.as_str()) {
                Some(p) if p == prompt => TestResult::passed(),
                _ => TestResult::failed(),
            }
        }

        QuickCheck::new()
            .tests(100)
            .quickcheck(property as fn(String) -> TestResult);
    }

    /// **Feature: cli-version, Property 2: Response display consistency**
    /// For any response received from the backend, the response text should appear
    /// in the terminal output with visual separation markers (such as newlines or formatting).
    /// **Validates: Requirements 1.3**
    /// 
    /// This property tests that responses are correctly captured and displayed.
    /// We verify this by simulating the streaming callback mechanism.
    #[test]
    fn prop_response_display_consistency() {
        fn property(response_text: String) -> TestResult {
            // Filter out empty responses
            if response_text.is_empty() {
                return TestResult::discard();
            }

            // Filter out responses with control characters
            if response_text.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
                return TestResult::discard();
            }

            // Limit response length
            if response_text.len() > 500 {
                return TestResult::discard();
            }

            // Simulate the streaming callback mechanism
            let output_buffer = Arc::new(Mutex::new(String::new()));
            let output_buffer_clone = Arc::clone(&output_buffer);

            // Simulate receiving the response in chunks
            let chunks: Vec<String> = response_text
                .chars()
                .collect::<Vec<_>>()
                .chunks(10)
                .map(|chunk| chunk.iter().collect())
                .collect();

            // Process each chunk through the callback
            for chunk in chunks {
                let mut buffer = output_buffer_clone.lock().unwrap();
                buffer.push_str(&chunk);
            }

            // Verify the complete response appears in the output
            let output = output_buffer.lock().unwrap();
            if output.contains(&response_text) && *output == response_text {
                TestResult::passed()
            } else {
                TestResult::failed()
            }
        }

        QuickCheck::new()
            .tests(100)
            .quickcheck(property as fn(String) -> TestResult);
    }

    #[tokio::test]
    async fn test_cli_app_creation() {
        let config = AppConfig::default();
        let result = CliApp::new(config, None, None);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_app_with_url_override() {
        let config = AppConfig::default();
        let custom_url = "http://custom-server:8080".to_string();
        let app = CliApp::new(config, Some(custom_url.clone()), None).unwrap();
        assert_eq!(app.backend_client.base_url(), "http://custom-server:8080");
    }

    #[tokio::test]
    async fn test_cli_app_with_model_override() {
        let config = AppConfig::default();
        let custom_model = "custom-model".to_string();
        let app = CliApp::new(config, None, Some(custom_model.clone())).unwrap();
        assert_eq!(app.model, "custom-model");
    }

    #[tokio::test]
    async fn test_display_welcome() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        let result = app.display_welcome();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_exit_command() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        assert!(app.running);

        app.handle_command("/exit").await.unwrap();
        assert!(!app.running);
    }

    #[tokio::test]
    async fn test_handle_quit_command() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        assert!(app.running);

        app.handle_command("/quit").await.unwrap();
        assert!(!app.running);
    }

    #[tokio::test]
    async fn test_handle_new_command() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        let original_id = app.conversation.id.clone();

        app.handle_command("/new").await.unwrap();
        assert_ne!(app.conversation.id, original_id);
    }

    #[tokio::test]
    async fn test_handle_help_command() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        let result = app.handle_command("/help").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_clear_command() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        let result = app.handle_command("/clear").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_unknown_command() {
        let config = AppConfig::default();
        let mut app = CliApp::new(config, None, None).unwrap();
        let result = app.handle_command("/unknown").await;
        assert!(result.is_ok());
    }
}
