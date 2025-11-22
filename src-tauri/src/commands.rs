use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use futures_util::StreamExt;
use crate::persona::{Persona, PersonaManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[tauri::command]
pub async fn get_models() -> Result<Vec<String>, String> {
    // Default Ollama URL
    let ollama_url = "http://localhost:11434";
    
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
) -> Result<(), String> {
    let ollama_url = "http://localhost:11434";
    
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
