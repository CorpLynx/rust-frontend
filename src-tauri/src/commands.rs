use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use futures_util::StreamExt;
use crate::persona::{Persona, PersonaManager};
use crate::config::{AppConfig, RemoteEndpoint};
use crate::network::{ConnectionManager, ConnectionTestResult};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[tauri::command]
pub async fn get_models(
    connection_manager: State<'_, Arc<ConnectionManager>>,
) -> Result<Vec<String>, String> {
    // Get the active endpoint URL based on connection mode
    // Requirements: 3.1, 3.2
    let ollama_url = connection_manager.get_active_endpoint()?;
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/api/tags", ollama_url);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let response_text = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

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
        Err("No models found".to_string())
    } else {
        Ok(model_names)
    }
}

#[tauri::command]
pub async fn send_message_stream(
    app: AppHandle,
    prompt: String,
    model: String,
    request_id: String,
    system_prompt: Option<String>,
    connection_manager: State<'_, Arc<ConnectionManager>>,
) -> Result<(), String> {
    // Get the active endpoint URL based on connection mode
    // Requirements: 3.1, 3.2
    let ollama_url = connection_manager.get_active_endpoint()?;
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/api/generate", ollama_url);

    // Prepend system prompt to user message if provided
    // Requirements: 3.1, 3.3
    let final_prompt = if let Some(sys_prompt) = system_prompt {
        format!("{}\n\n{}", sys_prompt, prompt)
    } else {
        prompt
    };

    let request_body = serde_json::json!({
        "model": model,
        "prompt": final_prompt,
        "stream": true
    });

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        let _ = app.emit("stream-error", serde_json::json!({
            "request_id": request_id,
            "error": format!("Server error: {}", response.status())
        }));
        return Err(format!("Server error: {}", response.status()));
    }

    // Process streaming response
    let mut stream = response.bytes_stream();
    let mut line_buffer = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                line_buffer.push_str(&text);

                // Process complete JSON lines
                while let Some(newline_pos) = line_buffer.find('\n') {
                    let line = line_buffer[..newline_pos].trim().to_string();
                    line_buffer = line_buffer[newline_pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(json) => {
                            if let Some(token) = json.get("response").and_then(|v| v.as_str()) {
                                let _ = app.emit("stream-token", serde_json::json!({
                                    "request_id": request_id,
                                    "token": token
                                }));
                            }

                            // Check if this is the final response
                            if json.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                                let _ = app.emit("stream-done", serde_json::json!({
                                    "request_id": request_id
                                }));
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON line: {} - Error: {}", line, e);
                        }
                    }
                }
            }
            Err(e) => {
                let _ = app.emit("stream-error", serde_json::json!({
                    "request_id": request_id,
                    "error": format!("Stream error: {}", e)
                }));
                return Err(format!("Stream error: {}", e));
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_chat_history() -> Result<Vec<ChatMessage>, String> {
    // Try to load from file
    match std::fs::read_to_string("chat_history.json") {
        Ok(content) => {
            serde_json::from_str::<Vec<ChatMessage>>(&content)
                .map_err(|e| format!("Failed to parse history: {}", e))
        }
        Err(_) => Ok(Vec::new()), // Return empty if file doesn't exist
    }
}

#[tauri::command]
pub fn new_conversation() -> Result<(), String> {
    // Clear chat history
    std::fs::write("chat_history.json", "[]")
        .map_err(|e| format!("Failed to clear history: {}", e))
}

// Persona management commands

/// Get all available personas
/// Requirements: 2.1, 3.5
#[tauri::command]
pub fn get_personas(persona_manager: State<PersonaManager>) -> Result<Vec<Persona>, String> {
    Ok(persona_manager.get_all_personas().to_vec())
}

/// Set the active persona by ID (or None to deselect)
/// Requirements: 2.1, 3.5
#[tauri::command]
pub fn set_active_persona(
    persona_manager: State<PersonaManager>,
    persona_id: Option<String>,
) -> Result<(), String> {
    persona_manager.set_active_persona(persona_id)
}

/// Get the currently active persona
/// Requirements: 3.5
#[tauri::command]
pub fn get_active_persona(persona_manager: State<PersonaManager>) -> Result<Option<Persona>, String> {
    Ok(persona_manager.get_active_persona())
}

// Endpoint management commands

/// Add a new remote endpoint
/// Requirements: 1.2, 1.3
#[tauri::command]
pub fn add_remote_endpoint(
    config: State<Arc<RwLock<AppConfig>>>,
    name: String,
    host: String,
    port: u16,
    use_https: bool,
    api_key: Option<String>,
) -> Result<String, String> {
    // Create the endpoint with validation
    let endpoint = RemoteEndpoint::new(name, host, port, use_https, api_key)
        .map_err(|e| format!("Validation failed: {:?}", e))?;
    
    // Add to config
    let mut config = config.write()
        .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
    
    let endpoint_id = config.backend.add_remote_endpoint(endpoint)
        .map_err(|e| format!("Failed to add endpoint: {:?}", e))?;
    
    // Save configuration
    config.save()
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(endpoint_id)
}

/// Remove a remote endpoint by ID
/// Requirements: 5.3
#[tauri::command]
pub fn remove_remote_endpoint(
    config: State<Arc<RwLock<AppConfig>>>,
    endpoint_id: String,
) -> Result<(), String> {
    let mut config = config.write()
        .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
    
    config.backend.remove_remote_endpoint(&endpoint_id)?;
    
    // Save configuration
    config.save()
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}

/// Update an existing remote endpoint
/// Requirements: 5.4
#[tauri::command]
pub fn update_remote_endpoint(
    config: State<Arc<RwLock<AppConfig>>>,
    endpoint_id: String,
    name: String,
    host: String,
    port: u16,
    use_https: bool,
    api_key: Option<String>,
) -> Result<(), String> {
    // Create the updated endpoint with validation
    let mut updated_endpoint = RemoteEndpoint::new(name, host, port, use_https, api_key)
        .map_err(|e| format!("Validation failed: {:?}", e))?;
    
    // Set the ID to match the existing endpoint
    updated_endpoint.id = endpoint_id.clone();
    
    // Update in config
    let mut config = config.write()
        .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
    
    config.backend.update_remote_endpoint(&endpoint_id, updated_endpoint)?;
    
    // Save configuration
    config.save()
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}

/// List all remote endpoints
/// Requirements: 5.1
#[tauri::command]
pub fn list_remote_endpoints(
    config: State<Arc<RwLock<AppConfig>>>,
) -> Result<Vec<RemoteEndpoint>, String> {
    let config = config.read()
        .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
    
    Ok(config.backend.list_remote_endpoints().to_vec())
}

/// Test connection to a remote endpoint
/// Requirements: 4.1, 4.2
#[tauri::command]
pub async fn test_remote_endpoint(
    connection_manager: State<'_, Arc<ConnectionManager>>,
    endpoint_id: String,
    config: State<'_, Arc<RwLock<AppConfig>>>,
) -> Result<ConnectionTestResult, String> {
    // Get the endpoint URL
    let endpoint_url = {
        let config = config.read()
            .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
        
        let endpoint = config.backend.get_remote_endpoint(&endpoint_id)
            .ok_or_else(|| format!("Endpoint with ID {} not found", endpoint_id))?;
        
        endpoint.url()
    };
    
    // Test the connection
    connection_manager.test_connection(&endpoint_url)
        .await
        .map_err(|e| e.to_string())
}

// Connection mode management commands

/// Set the connection mode (Local or Remote)
/// Requirements: 2.1, 2.4, 2.5
#[tauri::command]
pub async fn set_connection_mode(
    connection_manager: State<'_, Arc<ConnectionManager>>,
    mode: crate::config::ConnectionMode,
) -> Result<(), String> {
    connection_manager.switch_mode(mode).await
}

/// Get the current connection mode
/// Requirements: 2.1
#[tauri::command]
pub fn get_connection_mode(
    config: State<Arc<RwLock<AppConfig>>>,
) -> Result<crate::config::ConnectionMode, String> {
    let config = config.read()
        .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
    
    Ok(config.backend.get_connection_mode().clone())
}

/// Set the active remote endpoint
/// Requirements: 2.3, 5.2
#[tauri::command]
pub async fn set_active_remote_endpoint(
    connection_manager: State<'_, Arc<ConnectionManager>>,
    endpoint_id: String,
) -> Result<(), String> {
    connection_manager.set_active_remote_endpoint(&endpoint_id).await
}

/// Get the currently active endpoint (returns the URL)
/// Requirements: 2.2, 2.3
#[tauri::command]
pub fn get_active_endpoint(
    config: State<Arc<RwLock<AppConfig>>>,
) -> Result<String, String> {
    let config = config.read()
        .map_err(|e| format!("Failed to acquire config lock: {}", e))?;
    
    config.backend.get_active_endpoint_url()
}
