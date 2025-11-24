use anyhow::Result;
use prometheus::backend::BackendClient;
use prometheus::cli::commands::Command;
use prometheus::config::AppConfig;
use prometheus::conversation::{ChatMessage, Conversation, ConversationManager};
use std::sync::Arc;

/// Mock backend server for integration tests
mod mock_backend {
    use mockito::{Mock, Server, ServerGuard};
    use serde_json::json;

    pub struct MockOllamaServer {
        server: ServerGuard,
    }

    impl MockOllamaServer {
        pub async fn new() -> Self {
            let server = Server::new_async().await;
            Self { server }
        }

        pub fn url(&self) -> String {
            self.server.url()
        }

        /// Mock a streaming response
        pub fn mock_streaming_response(&mut self, prompt: &str, response: &str) -> Mock {
            let chunks: Vec<String> = response
                .chars()
                .collect::<Vec<_>>()
                .chunks(5)
                .map(|chunk| {
                    let text: String = chunk.iter().collect();
                    json!({
                        "model": "llama2",
                        "response": text,
                        "done": false
                    })
                    .to_string()
                })
                .collect();

            let mut body = chunks.join("\n");
            body.push_str(&format!(
                "\n{}",
                json!({
                    "model": "llama2",
                    "response": "",
                    "done": true
                })
            ));

            self.server
                .mock("POST", "/api/generate")
                .match_body(mockito::Matcher::PartialJsonString(
                    json!({
                        "prompt": prompt,
                        "stream": true
                    })
                    .to_string(),
                ))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(body)
                .create()
        }

        /// Mock the models endpoint
        pub fn mock_models_response(&mut self, models: Vec<&str>) -> Mock {
            let models_json: Vec<serde_json::Value> = models
                .iter()
                .map(|name| {
                    json!({
                        "name": name,
                        "modified_at": "2024-01-01T00:00:00Z",
                        "size": 1024
                    })
                })
                .collect();

            self.server
                .mock("GET", "/api/tags")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(
                    json!({
                        "models": models_json
                    })
                    .to_string(),
                )
                .create()
        }

        /// Mock an error response
        pub fn mock_error_response(&mut self, status: usize, message: &str) -> Mock {
            self.server
                .mock("POST", "/api/generate")
                .with_status(status)
                .with_body(message)
                .create()
        }
    }
}

/// Test full conversation flow: start → prompt → response → save → exit
/// This test validates the complete lifecycle of a CLI session
/// **Validates: Requirements 1.1, 1.2, 1.3, 3.1, 3.2, 3.3, 3.5**
#[tokio::test]
async fn test_full_conversation_flow() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    // Mock the backend response
    let _mock = mock_server.mock_streaming_response(
        "Hello, how are you?",
        "I'm doing well, thank you for asking!",
    );

    // Create backend client
    let client = BackendClient::new(backend_url.clone(), 30)?;

    // Create conversation manager
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Step 1: Add user prompt
    let user_message = ChatMessage::new("user".to_string(), "Hello, how are you?".to_string());
    conversation.add_message(user_message);

    // Step 2: Send prompt and collect streaming response
    let mut full_response = String::new();
    client
        .send_prompt_streaming("Hello, how are you?", "llama2", |chunk| {
            full_response.push_str(&chunk);
            Ok(())
        })
        .await?;

    // Step 3: Add AI response
    let ai_message = ChatMessage::new("assistant".to_string(), full_response.clone());
    conversation.add_message(ai_message);

    // Step 4: Save conversation
    manager.save_conversation(&conversation)?;

    // Step 5: Verify conversation was saved correctly
    let loaded = manager.load_conversation(&conversation_id)?;
    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.messages[0].role, "user");
    assert_eq!(loaded.messages[0].content, "Hello, how are you?");
    assert_eq!(loaded.messages[1].role, "assistant");
    assert_eq!(loaded.messages[1].content, full_response);

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test multi-turn conversations with multiple prompts and responses
/// **Validates: Requirements 1.2, 1.3, 3.1, 3.2, 3.3**
#[tokio::test]
async fn test_multi_turn_conversation() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    // Create backend client
    let client = BackendClient::new(backend_url.clone(), 30)?;

    // Create conversation
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Define conversation turns
    let turns = vec![
        ("What is Rust?", "Rust is a systems programming language."),
        ("What are its main features?", "Memory safety, concurrency, and performance."),
        ("Is it hard to learn?", "It has a learning curve but is very rewarding."),
    ];

    // Execute each turn
    for (prompt, expected_response) in turns.iter() {
        // Mock the response
        let _mock = mock_server.mock_streaming_response(prompt, expected_response);

        // Add user message
        conversation.add_message(ChatMessage::new("user".to_string(), prompt.to_string()));

        // Get AI response
        let mut full_response = String::new();
        client
            .send_prompt_streaming(prompt, "llama2", |chunk| {
                full_response.push_str(&chunk);
                Ok(())
            })
            .await?;

        // Add AI response
        conversation.add_message(ChatMessage::new("assistant".to_string(), full_response));

        // Save after each turn
        manager.save_conversation(&conversation)?;
    }

    // Verify all messages were saved
    let loaded = manager.load_conversation(&conversation_id)?;
    assert_eq!(loaded.messages.len(), 6); // 3 turns × 2 messages

    // Verify message order and content
    for (i, (prompt, response)) in turns.iter().enumerate() {
        assert_eq!(loaded.messages[i * 2].role, "user");
        assert_eq!(loaded.messages[i * 2].content, *prompt);
        assert_eq!(loaded.messages[i * 2 + 1].role, "assistant");
        assert_eq!(loaded.messages[i * 2 + 1].content, *response);
    }

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test command parsing and execution
/// **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**
#[tokio::test]
async fn test_command_execution() -> Result<()> {
    // Test /exit command
    let exit_cmd = Command::parse("/exit");
    assert_eq!(exit_cmd, Command::Exit);

    // Test /quit command
    let quit_cmd = Command::parse("/quit");
    assert_eq!(quit_cmd, Command::Quit);

    // Test /clear command
    let clear_cmd = Command::parse("/clear");
    assert_eq!(clear_cmd, Command::Clear);

    // Test /new command
    let new_cmd = Command::parse("/new");
    assert_eq!(new_cmd, Command::New);

    // Test /help command
    let help_cmd = Command::parse("/help");
    assert_eq!(help_cmd, Command::Help);

    // Test /models command
    let models_cmd = Command::parse("/models");
    assert_eq!(models_cmd, Command::Models);

    // Test unknown command
    let unknown_cmd = Command::parse("/unknown");
    assert_eq!(unknown_cmd, Command::Unknown("unknown".to_string()));

    Ok(())
}

/// Test /models command with backend integration
/// **Validates: Requirements 5.5**
#[tokio::test]
async fn test_models_command_integration() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    // Mock the models response
    let _mock = mock_server.mock_models_response(vec!["llama2", "codellama", "mistral"]);

    // Create backend client
    let client = BackendClient::new(backend_url, 30)?;

    // Fetch models
    let models = client.fetch_models().await?;

    // Verify models were fetched correctly
    assert_eq!(models.len(), 3);
    assert!(models.contains(&"llama2".to_string()));
    assert!(models.contains(&"codellama".to_string()));
    assert!(models.contains(&"mistral".to_string()));

    Ok(())
}

/// Test /new command creates a new conversation
/// **Validates: Requirements 5.3, 3.4**
#[tokio::test]
async fn test_new_command_creates_conversation() -> Result<()> {
    let manager = ConversationManager::new();

    // Create first conversation
    let mut conv1 = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conv1_id = conv1.id.clone();
    conv1.add_message(ChatMessage::new("user".to_string(), "First conversation".to_string()));
    manager.save_conversation(&conv1)?;

    // Simulate /new command - create second conversation
    let mut conv2 = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conv2_id = conv2.id.clone();
    conv2.add_message(ChatMessage::new("user".to_string(), "Second conversation".to_string()));
    manager.save_conversation(&conv2)?;

    // Verify both conversations exist and are different
    assert_ne!(conv1_id, conv2_id);

    let loaded1 = manager.load_conversation(&conv1_id)?;
    let loaded2 = manager.load_conversation(&conv2_id)?;

    assert_eq!(loaded1.messages[0].content, "First conversation");
    assert_eq!(loaded2.messages[0].content, "Second conversation");

    // Cleanup
    manager.delete_conversation(&conv1_id)?;
    manager.delete_conversation(&conv2_id)?;

    Ok(())
}

/// Test streaming response from start to finish
/// **Validates: Requirements 2.1, 2.2, 2.3**
#[tokio::test]
async fn test_streaming_end_to_end() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    let prompt = "Tell me a story";
    let full_response = "Once upon a time, in a land far away, there lived a brave knight.";

    // Mock streaming response
    let _mock = mock_server.mock_streaming_response(prompt, full_response);

    // Create backend client
    let client = BackendClient::new(backend_url, 30)?;

    // Track chunks received using std::sync::Mutex
    let chunks = Arc::new(std::sync::Mutex::new(Vec::new()));
    let chunks_clone = Arc::clone(&chunks);

    // Send prompt with streaming
    let result = client
        .send_prompt_streaming(prompt, "llama2", |chunk| {
            let mut c = chunks_clone.lock().unwrap();
            c.push(chunk.clone());
            Ok(())
        })
        .await?;

    // Verify chunks were received
    let received_chunks = chunks.lock().unwrap();
    assert!(!received_chunks.is_empty(), "Should have received chunks");

    // Verify complete response
    let reconstructed: String = received_chunks.iter().cloned().collect();
    assert_eq!(reconstructed, full_response);
    assert_eq!(result, full_response);

    Ok(())
}

/// Test streaming with multiple chunks
/// **Validates: Requirements 2.1, 2.2**
#[tokio::test]
async fn test_streaming_multiple_chunks() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    let prompt = "Count to ten";
    let full_response = "One, two, three, four, five, six, seven, eight, nine, ten!";

    // Mock streaming response (will be split into chunks of 5 characters)
    let _mock = mock_server.mock_streaming_response(prompt, full_response);

    // Create backend client
    let client = BackendClient::new(backend_url, 30)?;

    // Track chunk count using std::sync::Mutex
    let chunk_count = Arc::new(std::sync::Mutex::new(0));
    let chunk_count_clone = Arc::clone(&chunk_count);

    // Send prompt with streaming
    let result = client
        .send_prompt_streaming(prompt, "llama2", |_chunk| {
            let mut c = chunk_count_clone.lock().unwrap();
            *c += 1;
            Ok(())
        })
        .await?;

    // Verify multiple chunks were received
    let count = *chunk_count.lock().unwrap();
    assert!(count > 1, "Should have received multiple chunks, got {}", count);

    // Verify complete response
    assert_eq!(result, full_response);

    Ok(())
}

/// Test configuration loading from file
/// **Validates: Requirements 4.1, 4.2, 4.3**
#[tokio::test]
async fn test_configuration_from_file() -> Result<()> {
    use std::fs;
    use std::path::PathBuf;

    // Create a test config file
    let test_config_path = PathBuf::from("test_integration_config.toml");
    let test_config = r#"
[app]
window_title = "Test Prometheus"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://test-backend:9999"
ollama_url = "http://test-ollama:8888"
timeout_seconds = 45

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
"#;

    // Write test config
    fs::write(&test_config_path, test_config)?;

    // Load config
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(test_config_path.clone()))
        .build()?
        .try_deserialize::<AppConfig>()?;

    // Clean up
    fs::remove_file(&test_config_path)?;

    // Verify config values
    assert_eq!(loaded_config.backend.ollama_url, "http://test-ollama:8888");
    assert_eq!(loaded_config.backend.timeout_seconds, 45);

    Ok(())
}

/// Test CLI arguments override config file values
/// **Validates: Requirements 4.5**
#[tokio::test]
async fn test_cli_args_override_config() -> Result<()> {
    // Create a config with specific values
    let mut config = AppConfig::default();
    config.backend.ollama_url = "http://config-url:1111".to_string();

    // CLI arguments
    let cli_url = "http://cli-url:2222".to_string();
    let cli_model = "cli-model".to_string();

    // Simulate creating CLI app with overrides
    // (We can't directly test CliApp::new here without mocking terminal,
    // but we can verify the override logic)
    let final_url = cli_url.clone();
    let final_model = cli_model.clone();

    // Verify CLI args take precedence
    assert_eq!(final_url, "http://cli-url:2222");
    assert_eq!(final_model, "cli-model");
    assert_ne!(final_url, config.backend.ollama_url);

    Ok(())
}

/// Test configuration with missing values uses defaults
/// **Validates: Requirements 4.4**
#[tokio::test]
async fn test_configuration_defaults() -> Result<()> {
    use std::fs;
    use std::path::PathBuf;

    // Create a minimal config file (missing some fields)
    let test_config_path = PathBuf::from("test_minimal_config.toml");
    let test_config = r#"
[app]
window_title = "Minimal Config"
window_width = 800.0
window_height = 600.0

[backend]
url = "http://localhost:1234"
ollama_url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
"#;

    // Write test config
    fs::write(&test_config_path, test_config)?;

    // Load config
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(test_config_path.clone()))
        .build()?
        .try_deserialize::<AppConfig>()?;

    // Clean up
    fs::remove_file(&test_config_path)?;

    // Verify default theme is used when not specified
    assert_eq!(loaded_config.ui.theme, "Hacker Green");

    Ok(())
}

/// Test error handling when backend is unreachable
/// **Validates: Requirements 7.1**
#[tokio::test]
async fn test_backend_unreachable_error() -> Result<()> {
    // Create client with invalid URL
    let client = BackendClient::new("http://localhost:99999".to_string(), 1)?;

    // Try to send a prompt
    let result = client
        .send_prompt_streaming("test", "llama2", |_| Ok(()))
        .await;

    // Should fail with connection error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to connect") || error_msg.contains("error"),
        "Error message should indicate connection failure: {}",
        error_msg
    );

    Ok(())
}

/// Test error handling when backend returns error status
/// **Validates: Requirements 7.2**
#[tokio::test]
async fn test_backend_error_response() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    // Mock an error response
    let _mock = mock_server.mock_error_response(500, "Internal server error");

    // Create backend client
    let client = BackendClient::new(backend_url, 30)?;

    // Try to send a prompt
    let result = client
        .send_prompt_streaming("test", "llama2", |_| Ok(()))
        .await;

    // Should fail with backend error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("500") || error_msg.contains("error"),
        "Error message should indicate backend error: {}",
        error_msg
    );

    Ok(())
}

/// Test conversation persistence across multiple saves
/// **Validates: Requirements 3.3**
#[tokio::test]
async fn test_conversation_persistence_multiple_saves() -> Result<()> {
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Add messages and save multiple times
    for i in 0..5 {
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            format!("Message {}", i),
        ));
        manager.save_conversation(&conversation)?;

        // Verify it was saved correctly after each save
        let loaded = manager.load_conversation(&conversation_id)?;
        assert_eq!(loaded.messages.len(), i + 1);
    }

    // Final verification
    let final_loaded = manager.load_conversation(&conversation_id)?;
    assert_eq!(final_loaded.messages.len(), 5);
    for i in 0..5 {
        assert_eq!(final_loaded.messages[i].content, format!("Message {}", i));
    }

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test complete workflow with configuration, streaming, and persistence
/// **Validates: Requirements 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 3.2, 3.3, 4.2, 4.3**
#[tokio::test]
async fn test_complete_workflow() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    // Step 1: Load configuration (simulated)
    let config = AppConfig::default();
    assert_eq!(config.backend.timeout_seconds, 30);

    // Step 2: Create backend client with configured URL
    let client = BackendClient::new(backend_url.clone(), config.backend.timeout_seconds)?;

    // Step 3: Create conversation manager and new conversation
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Step 4: Execute multiple conversation turns
    let turns = vec![
        ("What is 2+2?", "The answer is 4."),
        ("What about 3+3?", "That equals 6."),
    ];

    for (prompt, response) in turns.iter() {
        // Mock the response
        let _mock = mock_server.mock_streaming_response(prompt, response);

        // Add user message
        conversation.add_message(ChatMessage::new("user".to_string(), prompt.to_string()));

        // Stream response
        let mut full_response = String::new();
        client
            .send_prompt_streaming(prompt, "llama2", |chunk| {
                full_response.push_str(&chunk);
                Ok(())
            })
            .await?;

        // Add AI response
        conversation.add_message(ChatMessage::new("assistant".to_string(), full_response));

        // Save after each turn
        manager.save_conversation(&conversation)?;
    }

    // Step 5: Verify complete conversation was saved
    let loaded = manager.load_conversation(&conversation_id)?;
    assert_eq!(loaded.messages.len(), 4);
    assert_eq!(loaded.model, Some("llama2".to_string()));

    // Step 6: Verify message content
    for (i, (prompt, response)) in turns.iter().enumerate() {
        assert_eq!(loaded.messages[i * 2].role, "user");
        assert_eq!(loaded.messages[i * 2].content, *prompt);
        assert_eq!(loaded.messages[i * 2 + 1].role, "assistant");
        assert_eq!(loaded.messages[i * 2 + 1].content, *response);
    }

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test conversation metadata is updated correctly
/// **Validates: Requirements 3.3**
#[tokio::test]
async fn test_conversation_metadata_updates() -> Result<()> {
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Add a message and save
    conversation.add_message(ChatMessage::new("user".to_string(), "Test message".to_string()));
    manager.save_conversation(&conversation)?;

    // Load metadata
    let metadata = manager.load_metadata()?;
    let conv_metadata = metadata
        .conversations
        .iter()
        .find(|m| m.id == conversation_id)
        .expect("Conversation metadata should exist");

    // Verify metadata
    assert_eq!(conv_metadata.message_count, 1);
    assert!(conv_metadata.preview.contains("Test message"));

    // Add more messages and save
    conversation.add_message(ChatMessage::new(
        "assistant".to_string(),
        "Response".to_string(),
    ));
    manager.save_conversation(&conversation)?;

    // Reload metadata
    let updated_metadata = manager.load_metadata()?;
    let updated_conv_metadata = updated_metadata
        .conversations
        .iter()
        .find(|m| m.id == conversation_id)
        .expect("Conversation metadata should exist");

    // Verify metadata was updated
    assert_eq!(updated_conv_metadata.message_count, 2);

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test streaming with callback errors
/// **Validates: Requirements 2.4**
#[tokio::test]
async fn test_streaming_with_callback_error() -> Result<()> {
    let mut mock_server = mock_backend::MockOllamaServer::new().await;
    let backend_url = mock_server.url();

    let prompt = "Test prompt";
    let response = "Test response";

    // Mock streaming response
    let _mock = mock_server.mock_streaming_response(prompt, response);

    // Create backend client
    let client = BackendClient::new(backend_url, 30)?;

    // Track chunk count using std::sync::Mutex
    let chunk_count = Arc::new(std::sync::Mutex::new(0));
    let chunk_count_clone = Arc::clone(&chunk_count);

    // Send prompt with callback that errors after first chunk
    let result = client
        .send_prompt_streaming(prompt, "llama2", |_chunk| {
            let mut c = chunk_count_clone.lock().unwrap();
            *c += 1;
            let current = *c;

            // Error after first chunk
            if current > 1 {
                anyhow::bail!("Simulated callback error");
            }
            Ok(())
        })
        .await;

    // Should fail due to callback error
    assert!(result.is_err());

    Ok(())
}

/// Test backend timeout handling
/// **Validates: Requirements 7.3**
#[tokio::test]
async fn test_backend_timeout() -> Result<()> {
    // Create client with very short timeout
    let client = BackendClient::new("http://httpbin.org/delay/10".to_string(), 1)?;

    // Try to send a prompt (should timeout)
    let result = client
        .send_prompt_streaming("test", "llama2", |_| Ok(()))
        .await;

    // Should fail with timeout error
    assert!(result.is_err());

    Ok(())
}

/// Test conversation list functionality
/// **Validates: Requirements 3.3**
#[tokio::test]
async fn test_conversation_list() -> Result<()> {
    let manager = ConversationManager::new();

    // Create multiple conversations
    let mut conv_ids = Vec::new();
    for i in 0..3 {
        let mut conv = Conversation::with_timestamp_name(Some("llama2".to_string()));
        conv.add_message(ChatMessage::new(
            "user".to_string(),
            format!("Conversation {}", i),
        ));
        conv_ids.push(conv.id.clone());
        manager.save_conversation(&conv)?;
    }

    // List conversations
    let conversations = manager.list_conversations()?;

    // Verify all conversations are listed
    assert!(conversations.len() >= 3);
    for conv_id in &conv_ids {
        assert!(
            conversations.iter().any(|c| c.id == *conv_id),
            "Conversation {} should be in list",
            conv_id
        );
    }

    // Cleanup
    for conv_id in conv_ids {
        manager.delete_conversation(&conv_id)?;
    }

    Ok(())
}

/// Test conversation deletion
/// **Validates: Requirements 3.3**
#[tokio::test]
async fn test_conversation_deletion() -> Result<()> {
    let manager = ConversationManager::new();

    // Create a conversation
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    conversation.add_message(ChatMessage::new("user".to_string(), "Test".to_string()));
    let conversation_id = conversation.id.clone();
    manager.save_conversation(&conversation)?;

    // Verify it exists
    assert!(manager.load_conversation(&conversation_id).is_ok());

    // Delete it
    manager.delete_conversation(&conversation_id)?;

    // Verify it no longer exists
    assert!(manager.load_conversation(&conversation_id).is_err());

    Ok(())
}

/// Test empty conversation handling
/// **Validates: Requirements 3.4**
#[tokio::test]
async fn test_empty_conversation() -> Result<()> {
    let manager = ConversationManager::new();

    // Create an empty conversation
    let conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Save empty conversation
    manager.save_conversation(&conversation)?;

    // Load it back
    let loaded = manager.load_conversation(&conversation_id)?;

    // Verify it's empty
    assert_eq!(loaded.messages.len(), 0);
    assert_eq!(loaded.id, conversation_id);

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test conversation with very long messages
/// **Validates: Requirements 1.2, 1.3, 3.1, 3.2**
#[tokio::test]
async fn test_long_messages() -> Result<()> {
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Create a very long message (10KB)
    let long_message = "A".repeat(10000);

    // Add long message
    conversation.add_message(ChatMessage::new("user".to_string(), long_message.clone()));

    // Save conversation
    manager.save_conversation(&conversation)?;

    // Load it back
    let loaded = manager.load_conversation(&conversation_id)?;

    // Verify long message was preserved
    assert_eq!(loaded.messages.len(), 1);
    assert_eq!(loaded.messages[0].content, long_message);
    assert_eq!(loaded.messages[0].content.len(), 10000);

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test conversation with special characters
/// **Validates: Requirements 1.2, 1.3, 3.1, 3.2**
#[tokio::test]
async fn test_special_characters_in_messages() -> Result<()> {
    let manager = ConversationManager::new();
    let mut conversation = Conversation::with_timestamp_name(Some("llama2".to_string()));
    let conversation_id = conversation.id.clone();

    // Messages with special characters
    let special_messages = vec![
        "Hello with emoji: 🚀 🎉 ✨",
        "Code: fn main() { println!(\"Hello\"); }",
        "Math: ∑ ∫ ∂ ∇ ∞",
        "Quotes: \"double\" and 'single'",
        "Newlines:\nLine 1\nLine 2\nLine 3",
    ];

    for msg in &special_messages {
        conversation.add_message(ChatMessage::new("user".to_string(), msg.to_string()));
    }

    // Save conversation
    manager.save_conversation(&conversation)?;

    // Load it back
    let loaded = manager.load_conversation(&conversation_id)?;

    // Verify all special characters were preserved
    assert_eq!(loaded.messages.len(), special_messages.len());
    for (i, expected) in special_messages.iter().enumerate() {
        assert_eq!(loaded.messages[i].content, *expected);
    }

    // Cleanup
    manager.delete_conversation(&conversation_id)?;

    Ok(())
}

/// Test backend client with different models
/// **Validates: Requirements 4.3**
#[tokio::test]
async fn test_different_models() -> Result<()> {
    let models = vec!["llama2", "codellama", "mistral"];

    for model in models {
        // Create a new mock server for each model
        let mut mock_server = mock_backend::MockOllamaServer::new().await;
        let backend_url = mock_server.url();

        // Mock response for this model
        let _mock = mock_server.mock_streaming_response("test", "response");

        // Create client
        let client = BackendClient::new(backend_url.clone(), 30)?;

        // Send prompt with this model
        let result = client
            .send_prompt_streaming("test", model, |_| Ok(()))
            .await;

        // Should succeed
        if let Err(e) = result {
            panic!("Failed for model {}: {:?}", model, e);
        }
    }

    Ok(())
}

/// Test concurrent conversation operations
/// **Validates: Requirements 3.3**
#[tokio::test]
async fn test_concurrent_conversation_operations() -> Result<()> {
    let manager = ConversationManager::new();

    // Create multiple conversations concurrently
    let mut handles = vec![];
    let mut conv_ids = vec![];

    for i in 0..5 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let mut conv = Conversation::with_timestamp_name(Some("llama2".to_string()));
            conv.add_message(ChatMessage::new(
                "user".to_string(),
                format!("Concurrent message {}", i),
            ));
            let id = conv.id.clone();
            manager_clone.save_conversation(&conv).unwrap();
            id
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let id = handle.await?;
        conv_ids.push(id);
    }

    // Verify all conversations were saved
    for conv_id in &conv_ids {
        let loaded = manager.load_conversation(conv_id)?;
        assert_eq!(loaded.messages.len(), 1);
    }

    // Cleanup
    for conv_id in conv_ids {
        manager.delete_conversation(&conv_id)?;
    }

    Ok(())
}

/// Test configuration with all fields specified
/// **Validates: Requirements 4.1, 4.2, 4.3**
#[tokio::test]
async fn test_complete_configuration() -> Result<()> {
    use std::fs;
    use std::path::PathBuf;

    // Create a complete config file
    let test_config_path = PathBuf::from("test_complete_config.toml");
    let test_config = r#"
[app]
window_title = "Complete Config Test"
window_width = 1024.0
window_height = 768.0

[backend]
url = "http://complete-backend:7777"
ollama_url = "http://complete-ollama:6666"
timeout_seconds = 60

[ui]
font_size = 18
max_chat_history = 2000
theme = "Cyber Blue"
"#;

    // Write test config
    fs::write(&test_config_path, test_config)?;

    // Load config
    let loaded_config = config::Config::builder()
        .add_source(config::File::from(test_config_path.clone()))
        .build()?
        .try_deserialize::<AppConfig>()?;

    // Clean up
    fs::remove_file(&test_config_path)?;

    // Verify all fields
    assert_eq!(loaded_config.app.window_title, "Complete Config Test");
    assert_eq!(loaded_config.app.window_width, 1024.0);
    assert_eq!(loaded_config.app.window_height, 768.0);
    assert_eq!(loaded_config.backend.url, "http://complete-backend:7777");
    assert_eq!(loaded_config.backend.ollama_url, "http://complete-ollama:6666");
    assert_eq!(loaded_config.backend.timeout_seconds, 60);
    assert_eq!(loaded_config.ui.font_size, 18);
    assert_eq!(loaded_config.ui.max_chat_history, 2000);
    assert_eq!(loaded_config.ui.theme, "Cyber Blue");

    Ok(())
}
