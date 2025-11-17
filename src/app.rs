use crate::config::{AppConfig, ColorTheme};
use crate::conversation::{Conversation, ConversationManager, ConversationMetadata};
use crate::markdown::{parse_message, MessageSegment};
use iced::{
    alignment, executor,
    widget::{
        button, column, container, pick_list, row, scrollable, text, text_input, Column, Row,
    },
    Alignment, Application, Command, Element, Length, Subscription, Theme,
    Color, Background, Border, time,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Global state for streaming responses
static STREAM_BUFFER: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new(String::new())));
static STREAM_COMPLETE: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

impl ChatMessage {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone)]
pub enum Message {
    PromptChanged(String),
    SendPrompt,
    ResponseReceived(Result<String, String>),
    StreamChunk(String),
    StreamComplete,
    StreamError(String),
    StreamPoll,
    StopGeneration,
    HistoryLoaded(Result<Vec<ChatMessage>, String>),
    ClearChat,
    Tick,
    ToggleSidebar,
    FetchModels,
    ModelsReceived(Result<Vec<String>, String>),
    ModelSelected(String),
    ToggleSettings,
    BackendUrlChanged(String),
    OllamaUrlChanged(String),
    ThemeSelected(String),
    SaveSettings,
    // Conversation management
    NewConversation,
    LoadConversation(String),
    ConversationsLoaded(Result<Vec<ConversationMetadata>, String>),
    SaveConversation,
    ConversationSaved(Result<(), String>),
    DeleteConversation(String),
    // Code block actions
    CopyCodeBlock(String),
    CodeBlockCopied,
}

pub struct ChatApp {
    config: AppConfig,
    prompt_input: String,
    chat_history: Vec<ChatMessage>,
    is_loading: bool,
    streaming_response: String,
    last_stream_len: usize,
    error_message: Option<String>,
    scroll_id: scrollable::Id,
    animation_offset: usize,
    sidebar_open: bool,
    sidebar_width: f32,
    available_models: Vec<String>,
    selected_model: Option<String>,
    models_loading: bool,
    settings_open: bool,
    temp_backend_url: String,
    temp_ollama_url: String,
    available_themes: Vec<String>,
    temp_theme: String,
    current_theme: ColorTheme,
    // Conversation management
    conversation_manager: ConversationManager,
    active_conversation_id: Option<String>,
    conversations: Vec<ConversationMetadata>,
    // Code block state
    copied_code_block: Option<usize>,
}

impl ChatApp {
    fn create(config: AppConfig) -> Self {
        let available_themes = vec![
            "Hacker Green".to_string(),
            "Cyber Blue".to_string(),
            "Neon Purple".to_string(),
            "Matrix Red".to_string(),
        ];
        
        let current_theme = ColorTheme::from_string(&config.ui.theme);
        
        Self {
            config,
            prompt_input: String::new(),
            chat_history: Vec::new(),
            is_loading: false,
            streaming_response: String::new(),
            last_stream_len: 0,
            error_message: None,
            scroll_id: scrollable::Id::unique(),
            animation_offset: 0,
            sidebar_open: false,
            sidebar_width: 0.0,
            available_models: Vec::new(),
            selected_model: None,
            models_loading: false,
            settings_open: false,
            temp_backend_url: String::new(),
            temp_ollama_url: String::new(),
            available_themes: available_themes.clone(),
            temp_theme: String::new(),
            current_theme,
            conversation_manager: ConversationManager::new(),
            active_conversation_id: None,
            conversations: Vec::new(),
            copied_code_block: None,
        }
    }

    fn load_conversations() -> Command<Message> {
        Command::perform(
            async move {
                let manager = ConversationManager::new();
                manager.list_conversations()
                    .map_err(|e| format!("Failed to load conversations: {}", e))
            },
            |result| Message::ConversationsLoaded(result),
        )
    }

    fn fetch_models(ollama_url: String) -> Command<Message> {
        Command::perform(
            async move {
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

                // Use /api/tags endpoint (standard Ollama)
                let url = format!("{}/api/tags", ollama_url.trim_end_matches('/'));
                
                let response = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| {
                        format!("Failed to fetch models from {}: {}", url, e)
                    })?;

                if !response.status().is_success() {
                    return Err(format!(
                        "API error: {} - {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    ));
                }

                // Get response text for debugging
                let response_text = response.text().await
                    .map_err(|e| format!("Failed to read response: {}", e))?;

                // Try to parse as JSON
                let json: serde_json::Value = serde_json::from_str(&response_text)
                    .map_err(|e| format!("Failed to parse JSON: {}. Response: {}", e, response_text))?;

                // Try multiple parsing strategies
                let model_names: Vec<String> = if let Some(models_array) = json.as_array() {
                    // Direct array of strings: ["model1", "model2"]
                    models_array
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                } else if let Some(data) = json.get("data").and_then(|v| v.as_array()) {
                    // OpenAI-style: {"data": [{"id": "model1"}, ...]}
                    data.iter()
                        .filter_map(|v| {
                            v.get("id")
                                .and_then(|id| id.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                } else if let Some(models) = json.get("models").and_then(|v| v.as_array()) {
                    // Ollama-style: {"models": [{"name": "model1"}, ...]}
                    models
                        .iter()
                        .filter_map(|v| {
                            if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                                Some(name.to_string())
                            } else if let Some(id) = v.get("id").and_then(|n| n.as_str()) {
                                Some(id.to_string())
                            } else if let Some(s) = v.as_str() {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    // Unknown format
                    return Err(format!("Unexpected response format: {}", response_text));
                };

                if model_names.is_empty() {
                    Err(format!("No models found in response: {}", response_text))
                } else {
                    Ok(model_names)
                }
            },
            |result: Result<Vec<String>, String>| Message::ModelsReceived(result),
        )
    }

    fn send_request(prompt: String, model: String, backend_url: String, timeout: u64) -> Command<Message> {
        // Clear the stream buffer and completion flag
        if let Ok(mut buffer) = STREAM_BUFFER.lock() {
            buffer.clear();
        }
        if let Ok(mut complete) = STREAM_COMPLETE.lock() {
            *complete = false;
        }
        
        // Spawn async task that updates the buffer
        tokio::spawn(async move {
            use futures::StreamExt;
            
            let client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout))
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create HTTP client: {}", e);
                    return;
                }
            };

            let full_url = format!("{}/api/generate", backend_url.trim_end_matches('/'));

            let request_body = serde_json::json!({
                "model": model,
                "prompt": prompt,
                "stream": true
            });

            let response = match client
                .post(&full_url)
                .json(&request_body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    error!("Network error: {}", e);
                    return;
                }
            };

            if !response.status().is_success() {
                error!("Server error: {}", response.status());
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            buffer.push_str(text);
                            
                            while let Some(newline_pos) = buffer.find('\n') {
                                let line = buffer[..newline_pos].trim().to_string();
                                buffer = buffer[newline_pos + 1..].to_string();
                                
                                if !line.is_empty() {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                                        if let Some(response_text) = json.get("response").and_then(|v| v.as_str()) {
                                            // Update the global buffer
                                            if let Ok(mut stream_buf) = STREAM_BUFFER.lock() {
                                                stream_buf.push_str(response_text);
                                            }
                                        }
                                        
                                        // Check if done
                                        if json.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                                            if let Ok(mut complete) = STREAM_COMPLETE.lock() {
                                                *complete = true;
                                            }
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        return;
                    }
                }
            }
        });
        
        Command::none()
    }

    fn log_error(&self, error: &str) {
        error!("{}", error);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/error.log")
        {
            let log_entry = format!(
                "[{}] ERROR: {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                error
            );
            let _ = file.write_all(log_entry.as_bytes());
        }
    }

    fn save_history(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.chat_history) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("chat_history.json")
            {
                let _ = file.write_all(json.as_bytes());
                info!("Chat history saved");
            }
        }
    }

    fn load_history() -> Command<Message> {
        Command::perform(
            async move {
                match std::fs::read_to_string("chat_history.json") {
                    Ok(content) => {
                        match serde_json::from_str::<Vec<ChatMessage>>(&content) {
                            Ok(history) => Ok(history),
                            Err(e) => Err(format!("Failed to parse history: {}", e)),
                        }
                    }
                    Err(_) => Ok(Vec::new()), // File doesn't exist, return empty history
                }
            },
            |result: Result<Vec<ChatMessage>, String>| Message::HistoryLoaded(result),
        )
    }
}

impl Application for ChatApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = AppConfig::load().unwrap_or_default();
        let ollama_url = config.backend.ollama_url.clone();
        let app = Self::create(config);
        (
            app,
            Command::batch(vec![
                Self::load_history(),
                Self::fetch_models(ollama_url),
                Self::load_conversations(),
            ]),
        )
    }

    fn title(&self) -> String {
        self.config.app.window_title.clone()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PromptChanged(value) => {
                self.prompt_input = value;
                self.error_message = None;
                Command::none()
            }
            Message::SendPrompt => {
                if self.is_loading {
                    return Command::none();
                }

                let prompt = self.prompt_input.trim().to_string();
                if prompt.is_empty() {
                    return Command::none();
                }

                // Add user message to history
                let user_message = ChatMessage::new("user".to_string(), prompt.clone());
                self.chat_history.push(user_message);

                // Limit chat history size
                if self.chat_history.len() > self.config.ui.max_chat_history {
                    self.chat_history.remove(0);
                }

                // Clear input and set loading state
                self.prompt_input.clear();
                self.is_loading = true;
                self.streaming_response.clear();
                self.error_message = None;

                // Save history
                self.save_history();

                // Send request
                info!("Sending prompt to backend: {}", prompt);
                let model = self.selected_model.clone().unwrap_or_else(|| "default".to_string());
                Command::batch(vec![
                    Self::send_request(
                        prompt,
                        model,
                        self.config.backend.url.clone(),
                        self.config.backend.timeout_seconds,
                    ),
                    scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END),
                ])
            }
            Message::StreamPoll => {
                if self.is_loading {
                    // Check if stream is complete
                    if let Ok(complete) = STREAM_COMPLETE.lock() {
                        if *complete {
                            return Command::perform(async {}, |_| Message::StreamComplete);
                        }
                    }
                    
                    // Check for new content
                    if let Ok(buffer) = STREAM_BUFFER.lock() {
                        let current_len = buffer.len();
                        if current_len > self.last_stream_len {
                            // New content available
                            self.streaming_response = buffer.clone();
                            self.last_stream_len = current_len;
                            return scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END);
                        }
                    }
                }
                Command::none()
            }
            Message::StopGeneration => {
                info!("Stopping generation");
                self.is_loading = false;
                
                // Save whatever we have so far
                if !self.streaming_response.is_empty() {
                    let ai_message = ChatMessage::new("assistant".to_string(), self.streaming_response.clone());
                    self.chat_history.push(ai_message);
                    self.save_history();
                }
                
                // Clear state
                self.streaming_response.clear();
                self.last_stream_len = 0;
                
                // Clear global buffers
                if let Ok(mut buffer) = STREAM_BUFFER.lock() {
                    buffer.clear();
                }
                if let Ok(mut complete) = STREAM_COMPLETE.lock() {
                    *complete = false;
                }
                
                // Auto-save conversation if we have content
                if !self.chat_history.is_empty() {
                    return Command::perform(async {}, |_| Message::SaveConversation);
                }
                
                Command::none()
            }
            Message::StreamChunk(chunk) => {
                self.streaming_response.push_str(&chunk);
                scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END)
            }
            Message::StreamComplete => {
                self.is_loading = false;
                if !self.streaming_response.is_empty() {
                    let ai_message = ChatMessage::new("assistant".to_string(), self.streaming_response.clone());
                    self.chat_history.push(ai_message);
                    self.streaming_response.clear();
                    self.last_stream_len = 0;

                    if self.chat_history.len() > self.config.ui.max_chat_history {
                        self.chat_history.remove(0);
                    }

                    self.save_history();
                    
                    // Auto-save conversation
                    return Command::batch(vec![
                        Command::perform(async {}, |_| Message::SaveConversation),
                        scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END),
                    ]);
                }
                // Clear the buffer
                if let Ok(mut buffer) = STREAM_BUFFER.lock() {
                    buffer.clear();
                }
                scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END)
            }
            Message::StreamError(error) => {
                self.is_loading = false;
                self.streaming_response.clear();
                self.last_stream_len = 0;
                let error_msg = format!("Error: {}", error);
                error!("{}", error_msg);
                self.log_error(&error_msg);
                self.error_message = Some(error_msg);
                Command::none()
            }
            Message::ResponseReceived(result) => {
                self.is_loading = false;

                match result {
                    Ok(response) => {
                        info!("Received response from backend");
                        let ai_message = ChatMessage::new("assistant".to_string(), response);
                        self.chat_history.push(ai_message);

                        // Limit chat history size
                        if self.chat_history.len() > self.config.ui.max_chat_history {
                            self.chat_history.remove(0);
                        }

                        self.save_history();
                        self.error_message = None;
                        
                        // Auto-scroll to bottom after receiving response
                        return scrollable::snap_to(self.scroll_id.clone(), scrollable::RelativeOffset::END);
                    }
                    Err(e) => {
                        let error_msg = format!("Error: {}", e);
                        error!("{}", error_msg);
                        self.log_error(&error_msg);
                        self.error_message = Some(error_msg);
                    }
                }

                Command::none()
            }
            Message::HistoryLoaded(result) => {
                match result {
                    Ok(history) => {
                        self.chat_history = history;
                        info!("Loaded {} messages from history", self.chat_history.len());
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to load history: {}", e);
                        error!("{}", error_msg);
                        self.log_error(&error_msg);
                    }
                }
                Command::none()
            }
            Message::ClearChat => {
                self.chat_history.clear();
                self.save_history();
                self.error_message = None;
                info!("Chat history cleared");
                Command::none()
            }
            Message::Tick => {
                // Animate the empty state text
                if self.chat_history.is_empty() {
                    self.animation_offset = (self.animation_offset + 1) % 50;
                }
                
                // Animate sidebar slide
                let target_width = if self.sidebar_open { 250.0 } else { 0.0 };
                if (self.sidebar_width - target_width).abs() > 1.0 {
                    let diff = target_width - self.sidebar_width;
                    self.sidebar_width += diff * 0.2; // Smooth easing
                } else {
                    self.sidebar_width = target_width;
                }
                
                Command::none()
            }
            Message::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                Command::none()
            }
            Message::FetchModels => {
                self.models_loading = true;
                Self::fetch_models(self.config.backend.ollama_url.clone())
            }
            Message::ModelsReceived(result) => {
                self.models_loading = false;
                match result {
                    Ok(models) => {
                        info!("Fetched {} models from Ollama", models.len());
                        self.available_models = models;
                        // Set first model as default if none selected
                        if self.selected_model.is_none() && !self.available_models.is_empty() {
                            self.selected_model = Some(self.available_models[0].clone());
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to fetch models: {}", e);
                        error!("{}", error_msg);
                        self.log_error(&error_msg);
                        // Keep any existing models
                    }
                }
                Command::none()
            }
            Message::ModelSelected(model) => {
                info!("Model selected: {} (not creating new chat)", model);
                self.selected_model = Some(model);
                Command::none()
            }
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                if self.settings_open {
                    // Load current values into temp fields
                    self.temp_backend_url = self.config.backend.url.clone();
                    self.temp_ollama_url = self.config.backend.ollama_url.clone();
                    self.temp_theme = self.config.ui.theme.clone();
                }
                Command::none()
            }
            Message::BackendUrlChanged(url) => {
                self.temp_backend_url = url;
                Command::none()
            }
            Message::OllamaUrlChanged(url) => {
                self.temp_ollama_url = url;
                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.temp_theme = theme;
                Command::none()
            }
            Message::SaveSettings => {
                let ollama_changed = self.temp_ollama_url != self.config.backend.ollama_url;
                let theme_changed = self.temp_theme != self.config.ui.theme;
                
                // Update config
                self.config.backend.url = self.temp_backend_url.clone();
                self.config.backend.ollama_url = self.temp_ollama_url.clone();
                self.config.ui.theme = self.temp_theme.clone();
                
                // Update current theme if changed
                if theme_changed {
                    self.current_theme = ColorTheme::from_string(&self.temp_theme);
                }
                
                // Save to file
                match self.config.save() {
                    Ok(_) => {
                        info!("Settings saved successfully");
                        self.settings_open = false;
                        self.error_message = None;
                        
                        // Refetch models if Ollama URL changed
                        if ollama_changed {
                            return Self::fetch_models(self.config.backend.ollama_url.clone());
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to save settings: {}", e);
                        error!("{}", error_msg);
                        self.log_error(&error_msg);
                        self.error_message = Some(error_msg);
                    }
                }
                Command::none()
            }
            Message::NewConversation => {
                // Create new conversation
                info!("NewConversation message received - creating new chat");
                let conversation = Conversation::with_timestamp_name(self.selected_model.clone());
                self.active_conversation_id = Some(conversation.id.clone());
                self.chat_history.clear();
                self.streaming_response.clear();
                self.last_stream_len = 0;
                self.is_loading = false;
                self.error_message = None;
                
                // Clear the global stream buffer
                if let Ok(mut buffer) = STREAM_BUFFER.lock() {
                    buffer.clear();
                }
                
                info!("Created new conversation: {}", conversation.id);
                Command::none()
            }
            Message::LoadConversation(id) => {
                self.active_conversation_id = Some(id.clone());
                self.streaming_response.clear();
                self.last_stream_len = 0;
                self.is_loading = false;
                
                // Clear the global stream buffer
                if let Ok(mut buffer) = STREAM_BUFFER.lock() {
                    buffer.clear();
                }
                
                let manager = self.conversation_manager.clone();
                Command::perform(
                    async move {
                        manager.load_conversation(&id)
                            .map_err(|e| format!("Failed to load conversation: {}", e))
                    },
                    |result| match result {
                        Ok(conv) => Message::HistoryLoaded(Ok(conv.messages)),
                        Err(e) => Message::HistoryLoaded(Err(e)),
                    },
                )
            }
            Message::ConversationsLoaded(result) => {
                match result {
                    Ok(conversations) => {
                        info!("Loaded {} conversations", conversations.len());
                        self.conversations = conversations;
                        
                        // Only create new conversation if we don't have one active (startup only)
                        if self.active_conversation_id.is_none() {
                            let conversation = Conversation::with_timestamp_name(self.selected_model.clone());
                            self.active_conversation_id = Some(conversation.id.clone());
                            self.chat_history.clear();
                            info!("Starting with new conversation: {}", conversation.id);
                        } else {
                            info!("Keeping existing active conversation");
                        }
                    }
                    Err(e) => {
                        error!("Failed to load conversations: {}", e);
                        self.error_message = Some(e);
                    }
                }
                Command::none()
            }
            Message::SaveConversation => {
                if let Some(id) = &self.active_conversation_id {
                    let manager = self.conversation_manager.clone();
                    let mut conversation = Conversation::new(
                        format!("Chat {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
                        self.selected_model.clone(),
                    );
                    conversation.id = id.clone();
                    conversation.messages = self.chat_history.clone();
                    
                    Command::perform(
                        async move {
                            manager.save_conversation(&conversation)
                                .map_err(|e| format!("Failed to save conversation: {}", e))
                        },
                        |result| Message::ConversationSaved(result),
                    )
                } else {
                    Command::none()
                }
            }
            Message::ConversationSaved(result) => {
                match result {
                    Ok(_) => {
                        info!("Conversation saved successfully");
                        // Reload conversations list
                        return Self::load_conversations();
                    }
                    Err(e) => {
                        error!("Failed to save conversation: {}", e);
                        self.error_message = Some(e);
                    }
                }
                Command::none()
            }
            Message::DeleteConversation(id) => {
                let manager = self.conversation_manager.clone();
                Command::perform(
                    async move {
                        manager.delete_conversation(&id)
                            .map_err(|e| format!("Failed to delete conversation: {}", e))
                    },
                    |result| match result {
                        Ok(_) => Message::ConversationsLoaded(Ok(Vec::new())), // Trigger reload
                        Err(e) => Message::ConversationsLoaded(Err(e)),
                    },
                )
            }
            Message::CopyCodeBlock(code) => {
                use arboard::Clipboard;
                match Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(&code) {
                            error!("Failed to copy to clipboard: {}", e);
                            self.error_message = Some(format!("Failed to copy: {}", e));
                        } else {
                            info!("Code copied to clipboard");
                        }
                    }
                    Err(e) => {
                        error!("Failed to access clipboard: {}", e);
                        self.error_message = Some(format!("Clipboard error: {}", e));
                    }
                }
                Command::perform(async {}, |_| Message::CodeBlockCopied)
            }
            Message::CodeBlockCopied => {
                // Reset copied state after 2 seconds
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Get theme colors
        let (pr, pg, pb) = self.current_theme.primary_color();
        let (sr, sg, sb) = self.current_theme.secondary_color();
        
        let text_color = Color::from_rgb(pr, pg, pb); // Theme primary color
        let muted_color = Color::from_rgb(sr, sg, sb); // Theme secondary color

        let chat_display: Element<_> = if self.chat_history.is_empty() {
            // ASCII art animation - rotating frames with glowing ONLINE text
            let glow_color = Color::from_rgb(pr, pg, pb); // Theme primary color
            
            let frames_top = vec![
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
                "╔═══════════════════════════╗\n║    Frontend build v0.2    ║\n║                           ║",
            ];
            
            let frames_online = vec![
                "║      [  ONLINE  ]         ║",
                "║      [ >ONLINE  ]         ║",
                "║      [ >>ONLINE ]         ║",
                "║      [ >>>ONLINE]         ║",
                "║      [  ONLINE>>>]        ║",
                "║      [  ONLINE>> ]        ║",
                "║      [  ONLINE>  ]        ║",
            ];
            
            let frames_bottom = vec![
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
                "║                           ║\n║   > awaiting input...     ║\n╚═══════════════════════════╝",
            ];

            let frame_index = (self.animation_offset / 5) % frames_online.len();

            container(
                column![
                    text(frames_top[frame_index])
                        .size(self.config.ui.font_size - 2)
                        .style(iced::theme::Text::Color(muted_color)),
                    text(frames_online[frame_index])
                        .size(self.config.ui.font_size - 2)
                        .style(iced::theme::Text::Color(glow_color)),
                    text(frames_bottom[frame_index])
                        .size(self.config.ui.font_size - 2)
                        .style(iced::theme::Text::Color(muted_color)),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        } else {
            let mut chat_column = Column::new()
                .spacing(12)
                .padding(20)
                .width(Length::Fill);

            for (msg_idx, message) in self.chat_history.iter().enumerate() {
                let is_user = message.role.as_str() == "user";

                // Parse message content
                let segments = parse_message(&message.content);
                
                // Build message content from segments - simplified approach
                let mut message_content = Column::new().spacing(8);
                let mut code_block_idx = 0;
                
                for segment in segments {
                    match segment {
                        MessageSegment::Text(t) => {
                            if !t.trim().is_empty() {
                                message_content = message_content.push(
                                    text(t)
                                        .size(self.config.ui.font_size)
                                        .style(iced::theme::Text::Color(text_color))
                                );
                            }
                        }
                        MessageSegment::CodeBlock { language, code } => {
                            let lang_label = language.as_deref().unwrap_or("code");
                            
                            let copy_button = button(
                                text("Copy")
                                    .size(self.config.ui.font_size - 4)
                            )
                            .on_press(Message::CopyCodeBlock(code.clone()))
                            .padding(4)
                            .style(iced::theme::Button::Custom(Box::new(CodeCopyButtonStyle::new(&self.current_theme))));
                            
                            let code_header = row![
                                text(lang_label)
                                    .size(self.config.ui.font_size - 4)
                                    .style(iced::theme::Text::Color(muted_color)),
                                copy_button
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center);
                            
                            let code_block = container(
                                column![
                                    code_header,
                                    text(&code)
                                        .size(self.config.ui.font_size - 2)
                                        .style(iced::theme::Text::Color(text_color))
                                        .font(iced::Font::MONOSPACE)
                                ]
                                .spacing(8)
                            )
                            .padding(12)
                            .width(Length::Fill)
                            .style(iced::theme::Container::Custom(Box::new(CodeBlockStyle::new(&self.current_theme))));
                            
                            message_content = message_content.push(code_block);
                            code_block_idx += 1;
                        }
                        MessageSegment::InlineCode(code) => {
                            message_content = message_content.push(
                                container(
                                    text(&code)
                                        .size(self.config.ui.font_size - 2)
                                        .style(iced::theme::Text::Color(text_color))
                                        .font(iced::Font::MONOSPACE)
                                )
                                .padding(2)
                                .style(iced::theme::Container::Custom(Box::new(InlineCodeStyle::new(&self.current_theme))))
                            );
                        }
                        MessageSegment::Bold(t) => {
                            message_content = message_content.push(
                                text(t)
                                    .size(self.config.ui.font_size)
                                    .style(iced::theme::Text::Color(text_color))
                                    .font(iced::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    })
                            );
                        }
                        MessageSegment::Italic(t) => {
                            message_content = message_content.push(
                                text(t)
                                    .size(self.config.ui.font_size)
                                    .style(iced::theme::Text::Color(text_color))
                                    .font(iced::Font {
                                        style: iced::font::Style::Italic,
                                        ..Default::default()
                                    })
                            );
                        }
                        MessageSegment::ListItem(item) => {
                            message_content = message_content.push(
                                row![
                                    text("•")
                                        .size(self.config.ui.font_size)
                                        .style(iced::theme::Text::Color(muted_color)),
                                    text(item)
                                        .size(self.config.ui.font_size)
                                        .style(iced::theme::Text::Color(text_color))
                                ]
                                .spacing(8)
                            );
                        }
                    }
                }

                // Ollama-style: user messages on right, AI on left
                let message_bubble = container(message_content)
                    .padding(16)
                    .max_width(600)
                    .style(if is_user {
                        iced::theme::Container::Custom(Box::new(UserMessageStyle))
                    } else {
                        iced::theme::Container::Custom(Box::new(AIMessageStyle))
                    });

                // Align user messages to the right, AI to the left
                let message_row = if is_user {
                    container(message_bubble)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Right)
                } else {
                    container(message_bubble)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Left)
                };

                chat_column = chat_column.push(message_row);
            }

            // Show streaming response or loading indicator when waiting for response
            if self.is_loading {
                if self.streaming_response.is_empty() {
                    // Show loading dots
                    let dots = match (self.animation_offset / 10) % 4 {
                        0 => "",
                        1 => ".",
                        2 => "..",
                        _ => "...",
                    };
                    let loading_text = format!("getting response{}", dots);
                    
                    let loading_bubble = container(
                        text(loading_text)
                            .size(self.config.ui.font_size)
                            .style(iced::theme::Text::Color(muted_color))
                    )
                    .padding(16)
                    .max_width(600)
                    .style(iced::theme::Container::Custom(Box::new(AIMessageStyle)));

                    let loading_row = container(loading_bubble)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Left);

                    chat_column = chat_column.push(loading_row);
                } else {
                    // Show streaming response with markdown parsing - simplified
                    let segments = parse_message(&self.streaming_response);
                    let mut streaming_content = Column::new().spacing(8);
                    
                    for segment in segments {
                        match segment {
                            MessageSegment::Text(t) => {
                                if !t.trim().is_empty() {
                                    streaming_content = streaming_content.push(
                                        text(t)
                                            .size(self.config.ui.font_size)
                                            .style(iced::theme::Text::Color(text_color))
                                    );
                                }
                            }
                            MessageSegment::CodeBlock { language, code } => {
                                
                                let lang_label = language.as_deref().unwrap_or("code");
                                
                                let code_block = container(
                                    column![
                                        text(lang_label)
                                            .size(self.config.ui.font_size - 4)
                                            .style(iced::theme::Text::Color(muted_color)),
                                        text(&code)
                                            .size(self.config.ui.font_size - 2)
                                            .style(iced::theme::Text::Color(text_color))
                                            .font(iced::Font::MONOSPACE)
                                    ]
                                    .spacing(8)
                                )
                                .padding(12)
                                .width(Length::Fill)
                                .style(iced::theme::Container::Custom(Box::new(CodeBlockStyle::new(&self.current_theme))));
                                
                                streaming_content = streaming_content.push(code_block);
                            }
                            MessageSegment::InlineCode(code) => {
                                streaming_content = streaming_content.push(
                                    container(
                                        text(&code)
                                            .size(self.config.ui.font_size - 2)
                                            .style(iced::theme::Text::Color(text_color))
                                            .font(iced::Font::MONOSPACE)
                                    )
                                    .padding(2)
                                    .style(iced::theme::Container::Custom(Box::new(InlineCodeStyle::new(&self.current_theme))))
                                );
                            }
                            MessageSegment::Bold(t) => {
                                streaming_content = streaming_content.push(
                                    text(t)
                                        .size(self.config.ui.font_size)
                                        .style(iced::theme::Text::Color(text_color))
                                        .font(iced::Font {
                                            weight: iced::font::Weight::Bold,
                                            ..Default::default()
                                        })
                                );
                            }
                            MessageSegment::Italic(t) => {
                                streaming_content = streaming_content.push(
                                    text(t)
                                        .size(self.config.ui.font_size)
                                        .style(iced::theme::Text::Color(text_color))
                                        .font(iced::Font {
                                            style: iced::font::Style::Italic,
                                            ..Default::default()
                                        })
                                );
                            }
                            MessageSegment::ListItem(item) => {
                                streaming_content = streaming_content.push(
                                    row![
                                        text("•")
                                            .size(self.config.ui.font_size)
                                            .style(iced::theme::Text::Color(muted_color)),
                                        text(item)
                                            .size(self.config.ui.font_size)
                                            .style(iced::theme::Text::Color(text_color))
                                    ]
                                    .spacing(8)
                                );
                            }
                        }
                    }
                    
                    let streaming_bubble = container(streaming_content)
                        .padding(16)
                        .max_width(600)
                        .style(iced::theme::Container::Custom(Box::new(AIMessageStyle)));

                    let streaming_row = container(streaming_bubble)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Left);

                    chat_column = chat_column.push(streaming_row);
                }
            }

            scrollable(chat_column)
                .id(self.scroll_id.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollbarStyle::new(&self.current_theme))))
                .into()
        };

        // Model selector dropdown
        let model_selector = pick_list(
            &self.available_models[..],
            self.selected_model.as_ref(),
            Message::ModelSelected,
        )
        .placeholder("Select Model")
        .width(Length::Fixed(150.0))
        .style(iced::theme::PickList::Custom(
            std::rc::Rc::new(ModelSelectorStyle),
            std::rc::Rc::new(ModelSelectorMenuStyle),
        ));

        // Send/Stop button - smaller size
        let action_button = if self.is_loading {
            button(text("■").size(self.config.ui.font_size + 2))
                .on_press(Message::StopGeneration)
                .padding(10)
                .width(Length::Fixed(40.0))
                .style(iced::theme::Button::Custom(Box::new(StopButtonStyle)))
        } else {
            button(text("↑").size(self.config.ui.font_size + 2))
                .on_press(Message::SendPrompt)
                .padding(10)
                .width(Length::Fixed(40.0))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle::new(&self.current_theme))))
        };

        // Responsive floating input box - centered and adapts to window size
        let input_box = container(
            row![
                model_selector,
                text_input("Ask anything...", &self.prompt_input)
                    .id(iced::widget::text_input::Id::unique())
                    .on_input(Message::PromptChanged)
                    .on_submit(Message::SendPrompt)
                    .size(self.config.ui.font_size)
                    .width(Length::Fill)
                    .padding(14)
                    .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme)))),
                action_button
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fill),
        )
        .padding(12)
        .max_width(800)
        .style(iced::theme::Container::Custom(Box::new(HackerInputContainerStyle)))
        .center_x()
        .width(Length::Fill);

        let input_row = container(input_box)
            .padding([10, 20, 20, 20])
            .width(Length::Fill)
            .center_x();

        let error_color = Color::from_rgb(1.0, 0.2, 0.4); // Hot pink/red

        let error_display = if let Some(error) = &self.error_message {
            container(
                text(format!("⚠ ERROR: {}", error))
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(error_color)),
            )
            .padding(10)
            .style(iced::theme::Container::Custom(Box::new(HackerErrorStyle)))
            .width(Length::Fixed(750.0))
            .center_x()
            .into()
        } else {
            Element::from(container(text("")).width(Length::Fill).height(Length::Shrink))
        };

        let header_text_color = Color::from_rgb(pr, pg, pb); // Theme primary color

        // Burger menu button
        let burger_button = button(
            text("[≡]")
                .size(self.config.ui.font_size)
        )
        .on_press(Message::ToggleSidebar)
        .padding(8)
        .style(iced::theme::Button::Text);

        let new_chat_button = button(
            text("[+]")
                .size(self.config.ui.font_size)
        )
        .on_press(Message::NewConversation)
        .padding(8)
        .style(iced::theme::Button::Text);

        let settings_button = button(
            text("[*]")
                .size(self.config.ui.font_size)
        )
        .on_press(Message::ToggleSettings)
        .padding(8)
        .style(iced::theme::Button::Text);

        // Ollama-style minimal header with burger menu, new chat on left, settings on right
        let header = container(
            row![
                container(
                    row![
                        burger_button,
                        new_chat_button
                    ]
                    .spacing(5)
                    .align_items(Alignment::Center)
                )
                .width(Length::FillPortion(2))
                .align_x(alignment::Horizontal::Left),
                container(
                    text("Frontend build")
                        .size(self.config.ui.font_size)
                        .style(iced::theme::Text::Color(header_text_color))
                )
                .width(Length::FillPortion(8))
                .center_x(),
                container(settings_button)
                    .width(Length::FillPortion(2))
                    .align_x(alignment::Horizontal::Right),
            ]
            .spacing(0)
            .align_items(Alignment::Center)
        )
        .padding(15)
        .width(Length::Fill);

        // Main chat area
        let main_content = column![
            header,
            error_display,
            chat_display,
            input_row
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        // Sidebar content
        let sidebar_content = if self.sidebar_width > 1.0 {
            let mut conversations_column = Column::new()
                .spacing(8)
                .padding(10)
                .width(Length::Fill);

            // Add header
            conversations_column = conversations_column.push(
                text("Conversations")
                    .size(self.config.ui.font_size)
                    .style(iced::theme::Text::Color(header_text_color))
            );

            // Add each conversation
            for conv in &self.conversations {
                let is_active = self.active_conversation_id.as_ref() == Some(&conv.id);
                
                // Delete button for this conversation
                let delete_btn = button(text("×").size(self.config.ui.font_size + 2))
                    .on_press(Message::DeleteConversation(conv.id.clone()))
                    .padding(4)
                    .style(iced::theme::Button::Text);
                
                let conv_content = column![
                    row![
                        text(&conv.name)
                            .size(self.config.ui.font_size - 2)
                            .style(iced::theme::Text::Color(if is_active { header_text_color } else { text_color }))
                            .width(Length::Fill),
                        delete_btn
                    ]
                    .align_items(Alignment::Center)
                    .spacing(4),
                    text(&conv.preview)
                        .size(self.config.ui.font_size - 4)
                        .style(iced::theme::Text::Color(muted_color))
                ]
                .spacing(4);
                
                let conv_item = container(conv_content)
                    .padding(8)
                    .width(Length::Fill)
                    .style(if is_active {
                        iced::theme::Container::Custom(Box::new(ActiveConversationStyle))
                    } else {
                        iced::theme::Container::Custom(Box::new(ConversationItemStyle))
                    });

                let conv_button = button(conv_item)
                    .on_press(Message::LoadConversation(conv.id.clone()))
                    .width(Length::Fill)
                    .style(iced::theme::Button::Text);

                conversations_column = conversations_column.push(conv_button);
            }

            container(
                scrollable(conversations_column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollbarStyle::new(&self.current_theme))))
            )
            .width(Length::Fixed(self.sidebar_width))
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(SidebarStyle)))
        } else {
            container(text(""))
                .width(Length::Fixed(0.0))
                .height(Length::Fill)
        };

        // Layout with sidebar
        let layout = row![
            sidebar_content,
            main_content
        ]
        .spacing(0);

        let base_view = container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(HackerContainerStyle)));

        // Settings modal overlay
        if self.settings_open {
            let settings_panel = container(
                column![
                    text("[ SETTINGS ]")
                        .size(self.config.ui.font_size + 4)
                        .style(iced::theme::Text::Color(header_text_color)),
                    column![
                        text("Backend URL:")
                            .size(self.config.ui.font_size - 2)
                            .style(iced::theme::Text::Color(muted_color)),
                        text_input("http://localhost:8000/generate", &self.temp_backend_url)
                            .on_input(Message::BackendUrlChanged)
                            .size(self.config.ui.font_size)
                            .padding(12)
                            .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme)))),
                    ]
                    .spacing(8),
                    column![
                        text("Ollama URL:")
                            .size(self.config.ui.font_size - 2)
                            .style(iced::theme::Text::Color(muted_color)),
                        text_input("http://localhost:11434", &self.temp_ollama_url)
                            .on_input(Message::OllamaUrlChanged)
                            .size(self.config.ui.font_size)
                            .padding(12)
                            .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme)))),
                    ]
                    .spacing(8),
                    column![
                        text("Theme:")
                            .size(self.config.ui.font_size - 2)
                            .style(iced::theme::Text::Color(muted_color)),
                        pick_list(
                            &self.available_themes[..],
                            Some(self.temp_theme.clone()),
                            Message::ThemeSelected,
                        )
                        .placeholder("Select Theme")
                        .width(Length::Fill)
                        .padding(12)
                        .style(iced::theme::PickList::Custom(
                            std::rc::Rc::new(ModelSelectorStyle),
                            std::rc::Rc::new(ModelSelectorMenuStyle),
                        )),
                    ]
                    .spacing(8),
                    row![
                        button(text("Cancel").size(self.config.ui.font_size - 2))
                            .on_press(Message::ToggleSettings)
                            .padding(10)
                            .style(iced::theme::Button::Text),
                        button(text("Save").size(self.config.ui.font_size - 2))
                            .on_press(Message::SaveSettings)
                            .padding(10)
                            .style(iced::theme::Button::Primary),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ]
                .spacing(20)
                .padding(30)
                .width(Length::Fixed(500.0)),
            )
            .style(iced::theme::Container::Custom(Box::new(SettingsPanelStyle)))
            .center_x()
            .center_y();

            let modal_overlay = container(settings_panel)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(SettingsModalStyle)))
                .center_x()
                .center_y();

            // Return modal overlay (which contains the settings panel)
            // The base view is still rendered underneath
            modal_overlay.into()
        } else {
            base_view.into()
        }
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            "hacker".to_string(),
            iced::theme::Palette {
                background: Color::from_rgb(0.05, 0.05, 0.08),
                text: Color::from_rgb(0.0, 1.0, 0.6),
                primary: Color::from_rgb(0.0, 0.8, 1.0),
                success: Color::from_rgb(0.0, 1.0, 0.6),
                danger: Color::from_rgb(1.0, 0.2, 0.4),
            },
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;
        
        // Animate when chat is empty OR sidebar is animating
        let target_width = if self.sidebar_open { 250.0 } else { 0.0 };
        let needs_animation = self.chat_history.is_empty() || (self.sidebar_width - target_width).abs() > 1.0;
        let needs_stream_poll = self.is_loading;
        
        let tick_sub = if needs_animation {
            time::every(Duration::from_millis(16)).map(|_| Message::Tick) // ~60fps
        } else {
            Subscription::none()
        };
        
        let stream_sub = if needs_stream_poll {
            time::every(Duration::from_millis(8)).map(|_| Message::StreamPoll) // Poll at 120fps for ultra-smooth streaming
        } else {
            Subscription::none()
        };
        
        Subscription::batch(vec![tick_sub, stream_sub])
    }
}

// Custom styles for hacker aesthetic
struct HackerContainerStyle;

impl iced::widget::container::StyleSheet for HackerContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.08))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.0),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct SidebarStyle;

impl iced::widget::container::StyleSheet for SidebarStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.03, 0.03, 0.06))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ConversationItemStyle;

impl iced::widget::container::StyleSheet for ConversationItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.0),
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

struct ActiveConversationStyle;

impl iced::widget::container::StyleSheet for ActiveConversationStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.1))),
            border: Border {
                color: Color::from_rgba(0.0, 1.0, 0.6, 0.8),
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerInputContainerStyle;

impl iced::widget::container::StyleSheet for HackerInputContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 1.0))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.5),
                width: 1.5,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

// User message style - lighter, on the right
struct UserMessageStyle;

impl iced::widget::container::StyleSheet for UserMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.12))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.3),
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

// AI message style - darker blue background (fixed color)
struct AIMessageStyle;

impl iced::widget::container::StyleSheet for AIMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.04))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.15),
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerErrorStyle;

impl iced::widget::container::StyleSheet for HackerErrorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 0.2, 0.4, 0.1))),
            border: Border {
                color: Color::from_rgb(1.0, 0.2, 0.4),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

struct HackerInputStyle {
    primary_color: (f32, f32, f32),
    secondary_color: (f32, f32, f32),
}

impl HackerInputStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
            secondary_color: theme.secondary_color(),
        }
    }
}

impl iced::widget::text_input::StyleSheet for HackerInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 0.0)),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgb(pr, pg, pb),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 0.0)),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            icon_color: Color::from_rgb(pr, pg, pb),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        let (sr, sg, sb) = self.secondary_color;
        Color::from_rgba(sr, sg, sb, 0.5)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.0, 1.0, 0.6)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.8, 1.0, 0.3)
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(_style)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.7, 0.5, 0.3)
    }
}

struct RoundedButtonStyle {
    primary_color: (f32, f32, f32),
    secondary_color: (f32, f32, f32),
}

impl RoundedButtonStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
            secondary_color: theme.secondary_color(),
        }
    }
}

impl iced::widget::button::StyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let (sr, sg, sb) = self.secondary_color;
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgb(sr, sg, sb))),
            border: Border {
                color: Color::from_rgba(sr, sg, sb, 0.0),
                width: 0.0,
                radius: 8.0.into(), // Rounded corners
            },
            text_color: Color::from_rgb(0.05, 0.05, 0.08),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgb(pr, pg, pb))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgb(0.05, 0.05, 0.08),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let (sr, sg, sb) = self.secondary_color;
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgb(sr * 0.8, sg * 0.8, sb * 0.8))),
            border: Border {
                color: Color::from_rgba(sr, sg, sb, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgb(0.05, 0.05, 0.08),
            ..Default::default()
        }
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let (sr, sg, sb) = self.secondary_color;
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(sr, sg, sb, 0.3))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgba(0.05, 0.05, 0.08, 0.5),
            ..Default::default()
        }
    }
}

struct StopButtonStyle;

impl iced::widget::button::StyleSheet for StopButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgb(1.0, 0.2, 0.4))),
            border: Border {
                color: Color::from_rgba(1.0, 0.2, 0.4, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgb(1.0, 0.3, 0.5))),
            border: Border {
                color: Color::from_rgba(1.0, 0.2, 0.4, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.1, 0.3))),
            border: Border {
                color: Color::from_rgba(1.0, 0.2, 0.4, 0.0),
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            ..Default::default()
        }
    }
}

struct ModelSelectorStyle;

impl iced::widget::pick_list::StyleSheet for ModelSelectorStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::pick_list::Appearance {
        iced::widget::pick_list::Appearance {
            text_color: Color::from_rgb(0.0, 1.0, 0.6),
            placeholder_color: Color::from_rgba(0.0, 0.7, 0.5, 0.7),
            handle_color: Color::from_rgb(0.0, 1.0, 0.6),
            background: Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.12)),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.5),
                width: 1.5,
                radius: 8.0.into(),
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::pick_list::Appearance {
        iced::widget::pick_list::Appearance {
            text_color: Color::from_rgb(0.0, 1.0, 0.6),
            placeholder_color: Color::from_rgba(0.0, 0.7, 0.5, 0.7),
            handle_color: Color::from_rgb(0.0, 1.0, 0.6),
            background: Background::Color(Color::from_rgba(0.0, 1.0, 0.6, 0.2)),
            border: Border {
                color: Color::from_rgb(0.0, 1.0, 0.6),
                width: 1.5,
                radius: 8.0.into(),
            },
        }
    }
}

struct ModelSelectorMenuStyle;

impl iced::widget::overlay::menu::StyleSheet for ModelSelectorMenuStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::overlay::menu::Appearance {
        iced::widget::overlay::menu::Appearance {
            text_color: Color::from_rgb(0.0, 1.0, 0.6),
            background: Background::Color(Color::from_rgb(0.08, 0.08, 0.12)),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.5),
                width: 1.5,
                radius: 8.0.into(),
            },
            selected_text_color: Color::from_rgb(0.0, 1.0, 0.6),
            selected_background: Background::Color(Color::from_rgba(0.0, 0.8, 1.0, 0.2)),
        }
    }
}

struct SettingsModalStyle;

impl iced::widget::container::StyleSheet for SettingsModalStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.08, 0.95))),
            border: Border::default(),
            ..Default::default()
        }
    }
}

struct SettingsPanelStyle;

impl iced::widget::container::StyleSheet for SettingsPanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.08))),
            border: Border {
                color: Color::from_rgba(0.0, 0.8, 1.0, 0.5),
                width: 1.5,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

// Code block styles
struct CodeBlockStyle {
    primary_color: (f32, f32, f32),
}

impl CodeBlockStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for CodeBlockStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.3),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct InlineCodeStyle {
    primary_color: (f32, f32, f32),
}

impl InlineCodeStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for InlineCodeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(pr, pg, pb, 0.1))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.0),
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

struct CodeCopyButtonStyle {
    primary_color: (f32, f32, f32),
}

impl CodeCopyButtonStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::button::StyleSheet for CodeCopyButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(pr, pg, pb, 0.2))),
            text_color: Color::from_rgb(pr, pg, pb),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.5),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::button::Appearance {
            background: Some(Background::Color(Color::from_rgba(pr, pg, pb, 0.3))),
            text_color: Color::from_rgb(pr, pg, pb),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.8),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

struct CustomScrollbarStyle {
    primary_color: (f32, f32, f32),
    secondary_color: (f32, f32, f32),
}

impl CustomScrollbarStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
            secondary_color: theme.secondary_color(),
        }
    }
}

impl iced::widget::scrollable::StyleSheet for CustomScrollbarStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
        let (r, g, b) = self.primary_color;
        
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.1))),
                border: Border {
                    radius: 6.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: Color::from_rgba(r, g, b, 0.6),
                    border: Border {
                        radius: 6.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_mouse_over_scrollbar: bool) -> iced::widget::scrollable::Appearance {
        let (r, g, b) = self.primary_color;
        
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.2))),
                border: Border {
                    radius: 6.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: Color::from_rgba(r, g, b, 0.8),
                    border: Border {
                        radius: 6.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }
}

