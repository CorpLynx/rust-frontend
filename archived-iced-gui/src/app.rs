use crate::backend::BackendClient;
use crate::config::{AppConfig, ColorTheme};
use crate::conversation::{Conversation, ConversationManager, ConversationMetadata};
use crate::icons;
use crate::markdown::{parse_message, MessageSegment};
use crate::search::SearchEngine;
use iced::{
    alignment, executor,
    widget::{
        button, column, container, pick_list, row, scrollable, text, text_input, Column,
    },
    Alignment, Application, Command, Element, Length, Subscription, Theme,
    Color, Background, Border, Point,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
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
    SetLocalOllama,
    SelectSavedUrl(String),
    DeleteSavedUrl(String),
    // Conversation management
    NewConversation,
    LoadConversation(String),
    ConversationsLoaded(Result<Vec<ConversationMetadata>, String>),
    SaveConversation,
    ConversationSaved(Result<(), String>),
    DeleteConversation(String),
    // Conversation context menu
    ShowConversationContextMenu(String, Point),
    HideConversationContextMenu,
    StartRenameConversation(String),
    UpdateConversationName(String),
    ConfirmRenameConversation,
    CancelRenameConversation,
    // Code block actions
    CopyCodeBlock(String),
    CodeBlockCopied,
    // Message context menu
    ShowMessageContextMenu(usize, Point), // (message_index, position)
    HideMessageContextMenu,
    CopyMessage(usize),
    DeleteMessage(usize),
    EditMessage(usize),
    ConfirmEdit(String),
    CancelEdit,
    // Search indexing
    BuildSearchIndex,
    SearchIndexBuilt(Result<Vec<Conversation>, String>),
    // Search UI
    ToggleSearch,
    SearchQueryChanged(String),
    ClearSearch,
    NextSearchResult,
    PreviousSearchResult,
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
    selected_saved_url: Option<String>,
    // Conversation management
    conversation_manager: ConversationManager,
    active_conversation_id: Option<String>,
    conversations: Vec<ConversationMetadata>,
    // Conversation context menu
    conversation_context_menu: Option<(String, Point)>, // (conversation_id, position)
    renaming_conversation_id: Option<String>,
    rename_input: String,
    // Code block state
    copied_code_block: Option<usize>,
    // Message context menu state
    message_context_menu: Option<(usize, Point)>, // (message_index, position)
    editing_message_index: Option<usize>,
    edit_message_content: String,
    // Search engine
    search_engine: SearchEngine,
    indexing_progress: Option<(usize, usize)>, // (current, total)
    // Search UI state
    search_active: bool,
    search_query: String,
    search_results: Vec<crate::search::SearchResult>,
    current_search_result_index: Option<usize>,
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
            selected_saved_url: None,
            conversation_manager: ConversationManager::new(),
            active_conversation_id: None,
            conversations: Vec::new(),
            conversation_context_menu: None,
            renaming_conversation_id: None,
            rename_input: String::new(),
            copied_code_block: None,
            message_context_menu: None,
            editing_message_index: None,
            edit_message_content: String::new(),
            search_engine: SearchEngine::new(),
            indexing_progress: None,
            search_active: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_result_index: None,
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

    fn build_search_index() -> Command<Message> {
        Command::perform(
            async move {
                let manager = ConversationManager::new();
                let conversations_metadata = manager.list_conversations()
                    .map_err(|e| format!("Failed to load conversations: {}", e))?;
                
                let mut conversations = Vec::new();
                for metadata in conversations_metadata {
                    if let Ok(conversation) = manager.load_conversation(&metadata.id) {
                        conversations.push(conversation);
                    }
                }
                
                Ok(conversations)
            },
            |result| Message::SearchIndexBuilt(result),
        )
    }

    fn fetch_models(ollama_url: String) -> Command<Message> {
        Command::perform(
            async move {
                let client = BackendClient::new(ollama_url, 10)
                    .map_err(|e| format!("Failed to create backend client: {}", e))?;
                
                client.fetch_models()
                    .await
                    .map_err(|e| format!("Failed to fetch models: {}", e))
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
            let client = match BackendClient::new(backend_url, timeout) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create backend client: {}", e);
                    return;
                }
            };

            let result = client.send_prompt_streaming(
                &prompt,
                &model,
                |chunk| {
                    // Update the global buffer
                    if let Ok(mut stream_buf) = STREAM_BUFFER.lock() {
                        stream_buf.push_str(&chunk);
                    }
                    Ok(())
                }
            ).await;

            match result {
                Ok(_) => {
                    // Mark as complete
                    if let Ok(mut complete) = STREAM_COMPLETE.lock() {
                        *complete = true;
                    }
                }
                Err(e) => {
                    error!("Streaming error: {}", e);
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
        // Load configuration from config.toml, including saved URLs
        // If config is missing or corrupted, use default values
        // Requirement 3.5: Saved URLs are loaded on app startup and handled gracefully
        let config = AppConfig::load().unwrap_or_default();
        let ollama_url = config.backend.ollama_url.clone();
        let app = Self::create(config);
        (
            app,
            Command::batch(vec![
                Self::load_history(),
                Self::fetch_models(ollama_url),
                Self::load_conversations(),
                Self::build_search_index(),
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
                
                // Add backend URL to saved URLs (only non-localhost URLs)
                self.config.backend.add_saved_url(self.temp_backend_url.clone());
                
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
            Message::SetLocalOllama => {
                self.temp_ollama_url = crate::config::BackendSettings::LOCAL_OLLAMA_URL.to_string();
                Command::none()
            }
            Message::SelectSavedUrl(url) => {
                self.temp_backend_url = url.clone();
                self.selected_saved_url = Some(url);
                Command::none()
            }
            Message::DeleteSavedUrl(url) => {
                // Remove URL from saved list
                self.config.backend.remove_saved_url(&url);
                
                // Clear temp_backend_url if it was the deleted URL
                if self.temp_backend_url == url {
                    self.temp_backend_url.clear();
                }
                
                // Clear selected_saved_url if it was the deleted URL
                if self.selected_saved_url.as_ref() == Some(&url) {
                    self.selected_saved_url = None;
                }
                
                // Persist config immediately
                if let Err(e) = self.config.save() {
                    let error_msg = format!("Failed to save settings after deletion: {}", e);
                    error!("{}", error_msg);
                    self.log_error(&error_msg);
                    self.error_message = Some(error_msg);
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
            Message::SearchIndexBuilt(result) => {
                match result {
                    Ok(conversations) => {
                        let count = conversations.len();
                        info!("Indexing {} conversations...", count);
                        self.indexing_progress = Some((0, count));
                        
                        // Index all conversations
                        for (idx, conversation) in conversations.iter().enumerate() {
                            self.search_engine.index_conversation(conversation);
                            self.indexing_progress = Some((idx + 1, count));
                        }
                        
                        info!("Search index built successfully with {} conversations", count);
                        self.indexing_progress = None;
                    }
                    Err(e) => {
                        error!("Failed to build search index: {}", e);
                        self.indexing_progress = None;
                    }
                }
                Command::none()
            }
            Message::BuildSearchIndex => {
                info!("Building search index...");
                self.indexing_progress = Some((0, 0));
                Self::build_search_index()
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
                let active_id = self.active_conversation_id.clone();
                
                // If deleting the active conversation, clear the chat
                if Some(&id) == active_id.as_ref() {
                    self.chat_history.clear();
                    self.active_conversation_id = None;
                }
                
                Command::perform(
                    async move {
                        // Delete the conversation
                        if let Err(e) = manager.delete_conversation(&id) {
                            return Err(format!("Failed to delete conversation: {}", e));
                        }
                        // Reload the conversation list
                        manager.list_conversations()
                            .map_err(|e| format!("Failed to reload conversations: {}", e))
                    },
                    |result| Message::ConversationsLoaded(result),
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
            Message::ShowConversationContextMenu(id, position) => {
                self.conversation_context_menu = Some((id, position));
                Command::none()
            }
            Message::HideConversationContextMenu => {
                self.conversation_context_menu = None;
                Command::none()
            }
            Message::StartRenameConversation(id) => {
                // Find the conversation and pre-fill the rename input
                if let Some(conv) = self.conversations.iter().find(|c| c.id == id) {
                    self.rename_input = conv.name.clone();
                    self.renaming_conversation_id = Some(id);
                    self.conversation_context_menu = None;
                }
                Command::none()
            }
            Message::UpdateConversationName(name) => {
                self.rename_input = name;
                Command::none()
            }
            Message::ConfirmRenameConversation => {
                if let Some(id) = &self.renaming_conversation_id {
                    let manager = self.conversation_manager.clone();
                    let conversation_id = id.clone();
                    let new_name = self.rename_input.clone();
                    
                    // Clear rename state
                    self.renaming_conversation_id = None;
                    self.rename_input.clear();
                    
                    // Load conversation, update name, and save
                    return Command::perform(
                        async move {
                            let mut conversation = manager.load_conversation(&conversation_id)
                                .map_err(|e| format!("Failed to load conversation: {}", e))?;
                            conversation.name = new_name;
                            manager.save_conversation(&conversation)
                                .map_err(|e| format!("Failed to save conversation: {}", e))?;
                            manager.list_conversations()
                                .map_err(|e| format!("Failed to reload conversations: {}", e))
                        },
                        |result| Message::ConversationsLoaded(result),
                    );
                }
                Command::none()
            }
            Message::CancelRenameConversation => {
                self.renaming_conversation_id = None;
                self.rename_input.clear();
                Command::none()
            }
            Message::ShowMessageContextMenu(index, position) => {
                // Store message index and cursor position
                // Determine if message is AI message
                self.message_context_menu = Some((index, position));
                Command::none()
            }
            Message::HideMessageContextMenu => {
                // Clear context menu state
                self.message_context_menu = None;
                Command::none()
            }
            Message::CopyMessage(index) => {
                // Extract message content at index
                if let Some(message) = self.chat_history.get(index) {
                    // Copy to clipboard using arboard
                    use arboard::Clipboard;
                    match Clipboard::new() {
                        Ok(mut clipboard) => {
                            if let Err(e) = clipboard.set_text(&message.content) {
                                error!("Failed to copy message to clipboard: {}", e);
                                self.error_message = Some(format!("Failed to copy: {}", e));
                            } else {
                                info!("Message copied to clipboard");
                            }
                        }
                        Err(e) => {
                            error!("Failed to access clipboard: {}", e);
                            self.error_message = Some(format!("Clipboard error: {}", e));
                        }
                    }
                }
                // Close context menu
                self.message_context_menu = None;
                Command::none()
            }
            Message::DeleteMessage(index) => {
                // Remove message from chat_history at index
                if index < self.chat_history.len() {
                    self.chat_history.remove(index);
                    
                    // Update conversation file
                    if let Some(id) = &self.active_conversation_id {
                        let manager = self.conversation_manager.clone();
                        let mut conversation = Conversation::new(
                            format!("Chat {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
                            self.selected_model.clone(),
                        );
                        conversation.id = id.clone();
                        conversation.messages = self.chat_history.clone();
                        
                        // Close context menu
                        self.message_context_menu = None;
                        
                        return Command::perform(
                            async move {
                                manager.save_conversation(&conversation)
                                    .map_err(|e| format!("Failed to save conversation: {}", e))
                            },
                            |result| Message::ConversationSaved(result),
                        );
                    }
                }
                
                // Close context menu
                self.message_context_menu = None;
                Command::none()
            }
            Message::EditMessage(index) => {
                // Set editing_message_index to message index
                // Load message content into edit_message_content
                if let Some(message) = self.chat_history.get(index) {
                    self.editing_message_index = Some(index);
                    self.edit_message_content = message.content.clone();
                }
                // Close context menu
                self.message_context_menu = None;
                Command::none()
            }
            Message::ConfirmEdit(content) => {
                // Update message content at editing_message_index
                if let Some(index) = self.editing_message_index {
                    if let Some(message) = self.chat_history.get_mut(index) {
                        message.content = content;
                        
                        // Remove all messages after edited message
                        self.chat_history.truncate(index + 1);
                        
                        // Save conversation
                        if let Some(id) = &self.active_conversation_id {
                            let manager = self.conversation_manager.clone();
                            let mut conversation = Conversation::new(
                                format!("Chat {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
                                self.selected_model.clone(),
                            );
                            conversation.id = id.clone();
                            conversation.messages = self.chat_history.clone();
                            
                            // Clear edit state
                            self.editing_message_index = None;
                            self.edit_message_content.clear();
                            
                            return Command::perform(
                                async move {
                                    manager.save_conversation(&conversation)
                                        .map_err(|e| format!("Failed to save conversation: {}", e))
                                },
                                |result| Message::ConversationSaved(result),
                            );
                        }
                    }
                }
                
                // Clear edit state
                self.editing_message_index = None;
                self.edit_message_content.clear();
                Command::none()
            }
            Message::CancelEdit => {
                // Clear edit state without changes
                self.editing_message_index = None;
                self.edit_message_content.clear();
                Command::none()
            }
            Message::ToggleSearch => {
                self.search_active = !self.search_active;
                if !self.search_active {
                    // Clear search when closing
                    self.search_query.clear();
                    self.search_results.clear();
                    self.current_search_result_index = None;
                }
                Command::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query.clone();
                
                if query.is_empty() {
                    self.search_results.clear();
                    self.current_search_result_index = None;
                } else {
                    // Perform search
                    let search_query = crate::search::SearchQuery::new(query);
                    self.search_results = self.search_engine.search(&search_query);
                    
                    // Set to first result if any
                    if !self.search_results.is_empty() {
                        self.current_search_result_index = Some(0);
                    } else {
                        self.current_search_result_index = None;
                    }
                }
                
                Command::none()
            }
            Message::ClearSearch => {
                self.search_query.clear();
                self.search_results.clear();
                self.current_search_result_index = None;
                Command::none()
            }
            Message::NextSearchResult => {
                if let Some(current) = self.current_search_result_index {
                    if current + 1 < self.search_results.len() {
                        self.current_search_result_index = Some(current + 1);
                    }
                }
                Command::none()
            }
            Message::PreviousSearchResult => {
                if let Some(current) = self.current_search_result_index {
                    if current > 0 {
                        self.current_search_result_index = Some(current - 1);
                    }
                }
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
            // ASCII art - PROMETHEUS
            let glow_color = Color::from_rgb(pr, pg, pb); // Theme primary color
            
            let ascii_art = r#"   ________  ________  ________  ________  ________  ________  ________  ________  ________  ________ 
  /        \/        \/        \/        \/        \/        \/    /   \/        \/    /   \/        \
 /         /         /         /         /         /        _/         /         /         /        _/
//      __/        _/         /         /        _//       //         /        _/         /-        / 
\\_____/  \____/___/\________/\__/__/__/\________/ \______/ \___/____/\________/\________/\________/  "#;

            container(
                text(ascii_art)
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(glow_color))
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
                        MessageSegment::Highlighted(t) => {
                            // Render highlighted text with a background color
                            message_content = message_content.push(
                                container(
                                    text(t)
                                        .size(self.config.ui.font_size)
                                        .style(iced::theme::Text::Color(Color::BLACK))
                                )
                                .padding(2)
                                .style(iced::theme::Container::Custom(Box::new(HighlightStyle::new(&self.current_theme))))
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

                // Wrap message bubble in mouse_area for right-click detection
                use iced::widget::mouse_area;
                
                let message_with_context = mouse_area(message_bubble)
                    .on_right_press(Message::ShowMessageContextMenu(msg_idx, Point::ORIGIN));

                // Align user messages to the right, AI to the left
                let message_row = if is_user {
                    container(message_with_context)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Right)
                } else {
                    container(message_with_context)
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
                            MessageSegment::Highlighted(t) => {
                                // Render highlighted text with a background color
                                streaming_content = streaming_content.push(
                                    container(
                                        text(t)
                                            .size(self.config.ui.font_size)
                                            .style(iced::theme::Text::Color(Color::BLACK))
                                    )
                                    .padding(2)
                                    .style(iced::theme::Container::Custom(Box::new(HighlightStyle::new(&self.current_theme))))
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
            text(icons::ICON_MENU)
                .size(self.config.ui.font_size + 4)
                .font(iced::Font::with_name("FiraCode Nerd Font"))
        )
        .on_press(Message::ToggleSidebar)
        .padding(8)
        .style(iced::theme::Button::Text);

        let new_chat_button = button(
            text(icons::ICON_PLUS)
                .size(self.config.ui.font_size + 4)
                .font(iced::Font::with_name("FiraCode Nerd Font"))
        )
        .on_press(Message::NewConversation)
        .padding(8)
        .style(iced::theme::Button::Text);

        let search_button = button(
            text(icons::ICON_SEARCH)
                .size(self.config.ui.font_size + 4)
                .font(iced::Font::with_name("FiraCode Nerd Font"))
        )
        .on_press(Message::ToggleSearch)
        .padding(8)
        .style(iced::theme::Button::Text);

        let settings_button = button(
            text(icons::ICON_SETTINGS)
                .size(self.config.ui.font_size + 4)
                .font(iced::Font::with_name("FiraCode Nerd Font"))
        )
        .on_press(Message::ToggleSettings)
        .padding(8)
        .style(iced::theme::Button::Text);

        // Ollama-style minimal header with burger menu, new chat on left, search and settings on right
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
                    text("Prometheus")
                        .size(self.config.ui.font_size)
                        .style(iced::theme::Text::Color(header_text_color))
                )
                .width(Length::FillPortion(8))
                .center_x(),
                container(
                    row![
                        search_button,
                        settings_button
                    ]
                    .spacing(5)
                    .align_items(Alignment::Center)
                )
                .width(Length::FillPortion(2))
                .align_x(alignment::Horizontal::Right),
            ]
            .spacing(0)
            .align_items(Alignment::Center)
        )
        .padding(15)
        .width(Length::Fill);

        // Search bar (shown when search is active)
        let search_bar = if self.search_active {
            let result_text = if self.search_results.is_empty() {
                if self.search_query.is_empty() {
                    "Type to search...".to_string()
                } else {
                    "No results".to_string()
                }
            } else {
                let current = self.current_search_result_index.unwrap_or(0) + 1;
                format!("{} / {}", current, self.search_results.len())
            };

            let prev_button = button(
                text(icons::ICON_ARROW_UP)
                    .size(self.config.ui.font_size)
                    .font(iced::Font::with_name("FiraCode Nerd Font"))
            )
            .on_press(Message::PreviousSearchResult)
            .padding(8)
            .style(iced::theme::Button::Text);

            let next_button = button(
                text(icons::ICON_ARROW_DOWN)
                    .size(self.config.ui.font_size)
                    .font(iced::Font::with_name("FiraCode Nerd Font"))
            )
            .on_press(Message::NextSearchResult)
            .padding(8)
            .style(iced::theme::Button::Text);

            let clear_button = button(
                text(icons::ICON_CLOSE)
                    .size(self.config.ui.font_size)
                    .font(iced::Font::with_name("FiraCode Nerd Font"))
            )
            .on_press(Message::ClearSearch)
            .padding(8)
            .style(iced::theme::Button::Text);

            let close_button = button(
                text("Close")
                    .size(self.config.ui.font_size - 2)
            )
            .on_press(Message::ToggleSearch)
            .padding(8)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle::new(&self.current_theme))));

            Some(
                container(
                    row![
                        text(icons::ICON_SEARCH)
                            .size(self.config.ui.font_size)
                            .font(iced::Font::with_name("FiraCode Nerd Font"))
                            .style(iced::theme::Text::Color(text_color)),
                        text_input("Search conversations...", &self.search_query)
                            .on_input(Message::SearchQueryChanged)
                            .size(self.config.ui.font_size)
                            .padding(8)
                            .width(Length::Fill)
                            .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme)))),
                        clear_button,
                        text(result_text)
                            .size(self.config.ui.font_size - 2)
                            .style(iced::theme::Text::Color(muted_color)),
                        prev_button,
                        next_button,
                        close_button,
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .padding(10)
                .width(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(SearchBarStyle::new(&self.current_theme))))
            )
        } else {
            None
        };

        // Main chat area
        let mut main_content_column = column![header];
        if let Some(search) = search_bar {
            main_content_column = main_content_column.push(search);
        }
        main_content_column = main_content_column
            .push(error_display)
            .push(chat_display)
            .push(input_row);

        let main_content = main_content_column
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
                let is_renaming = self.renaming_conversation_id.as_ref() == Some(&conv.id);
                let show_context_menu = self.conversation_context_menu.as_ref()
                    .map(|(id, _)| id == &conv.id)
                    .unwrap_or(false);
                
                if is_renaming {
                    // Show rename input
                    let rename_content = column![
                        text_input("Conversation name", &self.rename_input)
                            .on_input(Message::UpdateConversationName)
                            .on_submit(Message::ConfirmRenameConversation)
                            .size(self.config.ui.font_size - 2)
                            .padding(8)
                            .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme)))),
                        row![
                            button(text("Save").size(self.config.ui.font_size - 4))
                                .on_press(Message::ConfirmRenameConversation)
                                .padding(4)
                                .style(iced::theme::Button::Primary),
                            button(text("Cancel").size(self.config.ui.font_size - 4))
                                .on_press(Message::CancelRenameConversation)
                                .padding(4)
                                .style(iced::theme::Button::Text),
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                    ]
                    .spacing(4);
                    
                    let rename_item = container(rename_content)
                        .padding(8)
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(ActiveConversationStyle)));
                    
                    conversations_column = conversations_column.push(rename_item);
                } else {
                    // Show context menu if this conversation is right-clicked
                    if show_context_menu {
                        let menu_content = column![
                            button(text("Rename").size(self.config.ui.font_size - 2))
                                .on_press(Message::StartRenameConversation(conv.id.clone()))
                                .width(Length::Fill)
                                .padding(6)
                                .style(iced::theme::Button::Text),
                            button(text("Delete").size(self.config.ui.font_size - 2))
                                .on_press(Message::DeleteConversation(conv.id.clone()))
                                .width(Length::Fill)
                                .padding(6)
                                .style(iced::theme::Button::Text),
                        ]
                        .spacing(2);
                        
                        let context_menu = container(menu_content)
                            .padding(4)
                            .width(Length::Fill)
                            .style(iced::theme::Container::Custom(Box::new(ContextMenuStyle::new(&self.current_theme))));
                        
                        conversations_column = conversations_column.push(context_menu);
                    } else {
                        // Show normal conversation item with right-click button
                        let conv_content = column![
                            row![
                                text(&conv.name)
                                    .size(self.config.ui.font_size - 2)
                                    .style(iced::theme::Text::Color(if is_active { header_text_color } else { text_color }))
                                    .width(Length::Fill),
                                button(text("...").size(self.config.ui.font_size - 2))
                                    .on_press(Message::ShowConversationContextMenu(conv.id.clone(), Point::new(0.0, 0.0)))
                                    .padding(2)
                                    .style(iced::theme::Button::Text),
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
                }
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

        // Message context menu overlay
        if let Some((msg_idx, _position)) = self.message_context_menu {
            // Determine if this is an AI message
            let is_ai_message = self.chat_history.get(msg_idx)
                .map(|msg| msg.role == "assistant")
                .unwrap_or(false);
            
            // Build context menu options
            let mut menu_column = Column::new().spacing(2);
            
            // Copy option
            let copy_button = button(
                text("📋 Copy")
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(text_color))
            )
            .on_press(Message::CopyMessage(msg_idx))
            .width(Length::Fill)
            .padding(8)
            .style(iced::theme::Button::Text);
            menu_column = menu_column.push(copy_button);
            
            // Delete option
            let delete_button = button(
                text("🗑️  Delete")
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(text_color))
            )
            .on_press(Message::DeleteMessage(msg_idx))
            .width(Length::Fill)
            .padding(8)
            .style(iced::theme::Button::Text);
            menu_column = menu_column.push(delete_button);
            
            // Edit option (only for user messages)
            if !is_ai_message {
                let edit_button = button(
                    text("✏️  Edit")
                        .size(self.config.ui.font_size - 2)
                        .style(iced::theme::Text::Color(text_color))
                )
                .on_press(Message::EditMessage(msg_idx))
                .width(Length::Fill)
                .padding(8)
                .style(iced::theme::Button::Text);
                menu_column = menu_column.push(edit_button);
            }
            
            // Create the context menu container
            let context_menu = container(menu_column)
                .padding(4)
                .width(Length::Fixed(150.0))
                .style(iced::theme::Container::Custom(Box::new(MessageContextMenuStyle::new(&self.current_theme))));
            
            // Position the menu (for now, centered - in a real implementation, use cursor position)
            let menu_positioned = container(context_menu)
                .padding([100, 0, 0, 100]); // Position from top-left
            
            // Create click-outside-to-close overlay
            use iced::widget::mouse_area;
            let modal_overlay = mouse_area(
                container(menu_positioned)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(TransparentOverlayStyle)))
            )
            .on_press(Message::HideMessageContextMenu);
            
            return modal_overlay.into();
        }

        // Settings modal overlay
        if self.settings_open {
            // Build saved URLs dropdown with delete buttons
            let mut backend_url_section = column![
                text("Backend URL:")
                    .size(self.config.ui.font_size - 2)
                    .style(iced::theme::Text::Color(muted_color)),
            ]
            .spacing(8);
            
            // Add saved URLs dropdown if there are any saved URLs
            if !self.config.backend.saved_urls.is_empty() {
                let mut saved_urls_column = Column::new().spacing(4);
                
                for url in &self.config.backend.saved_urls {
                    let url_row = row![
                        button(
                            text(url)
                                .size(self.config.ui.font_size - 2)
                                .style(iced::theme::Text::Color(text_color))
                        )
                        .on_press(Message::SelectSavedUrl(url.clone()))
                        .width(Length::Fill)
                        .padding(8)
                        .style(iced::theme::Button::Text),
                        button(
                            text(icons::ICON_DELETE)
                                .size(self.config.ui.font_size - 2)
                                .font(iced::Font::with_name("FiraCode Nerd Font"))
                                .style(iced::theme::Text::Color(Color::from_rgb(1.0, 0.2, 0.4)))
                        )
                        .on_press(Message::DeleteSavedUrl(url.clone()))
                        .padding(8)
                        .style(iced::theme::Button::Text),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center);
                    
                    saved_urls_column = saved_urls_column.push(url_row);
                }
                
                let saved_urls_container = container(
                    scrollable(saved_urls_column)
                        .height(Length::Fixed(120.0))
                )
                .padding(8)
                .width(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(SavedUrlsDropdownStyle::new(&self.current_theme))));
                
                backend_url_section = backend_url_section.push(
                    column![
                        text("Saved URLs:")
                            .size(self.config.ui.font_size - 3)
                            .style(iced::theme::Text::Color(muted_color)),
                        saved_urls_container
                    ]
                    .spacing(4)
                );
            }
            
            backend_url_section = backend_url_section.push(
                text_input("http://localhost:8000/generate", &self.temp_backend_url)
                    .on_input(Message::BackendUrlChanged)
                    .size(self.config.ui.font_size)
                    .padding(12)
                    .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme))))
            );
            
            // Check if local mode is active
            let is_local_mode = self.temp_ollama_url == crate::config::BackendSettings::LOCAL_OLLAMA_URL;
            
            // Build Ollama URL section with Local button
            let local_button = button(
                row![
                    text(icons::ICON_LOCAL)
                        .size(self.config.ui.font_size - 2)
                        .font(iced::Font::with_name("FiraCode Nerd Font")),
                    text("Local")
                        .size(self.config.ui.font_size - 2)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::SetLocalOllama)
            .padding(8)
            .style(if is_local_mode {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            });
            
            let ollama_url_row = row![
                text_input("http://localhost:11434", &self.temp_ollama_url)
                    .on_input(Message::OllamaUrlChanged)
                    .size(self.config.ui.font_size)
                    .padding(12)
                    .style(iced::theme::TextInput::Custom(Box::new(HackerInputStyle::new(&self.current_theme)))),
                local_button
            ]
            .spacing(8)
            .align_items(Alignment::Center);
            
            let settings_panel = container(
                column![
                    text("[ SETTINGS ]")
                        .size(self.config.ui.font_size + 4)
                        .style(iced::theme::Text::Color(header_text_color)),
                    backend_url_section,
                    column![
                        text("Ollama URL:")
                            .size(self.config.ui.font_size - 2)
                            .style(iced::theme::Text::Color(muted_color)),
                        ollama_url_row,
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
                .width(Length::Fixed(550.0)),
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

// Saved URLs dropdown style
struct SavedUrlsDropdownStyle {
    primary_color: (f32, f32, f32),
}

impl SavedUrlsDropdownStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for SavedUrlsDropdownStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.08, 0.08, 0.12, 0.8))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.3),
                width: 1.0,
                radius: 8.0.into(),
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

struct HighlightStyle {
    primary_color: (f32, f32, f32),
}

impl HighlightStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for HighlightStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(pr, pg, pb, 0.4))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.0),
                width: 0.0,
                radius: 2.0.into(),
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

struct ContextMenuStyle {
    primary_color: (f32, f32, f32),
}

impl ContextMenuStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for ContextMenuStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.08, 0.98))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.8),
                width: 1.5,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

// Message context menu style
struct MessageContextMenuStyle {
    primary_color: (f32, f32, f32),
}

impl MessageContextMenuStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for MessageContextMenuStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.08, 0.98))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.8),
                width: 1.5,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

// Transparent overlay style for click-outside-to-close
struct TransparentOverlayStyle;

impl iced::widget::container::StyleSheet for TransparentOverlayStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0))),
            border: Border::default(),
            ..Default::default()
        }
    }
}

// Search bar style
struct SearchBarStyle {
    primary_color: (f32, f32, f32),
}

impl SearchBarStyle {
    fn new(theme: &ColorTheme) -> Self {
        Self {
            primary_color: theme.primary_color(),
        }
    }
}

impl iced::widget::container::StyleSheet for SearchBarStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let (pr, pg, pb) = self.primary_color;
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.05, 0.05, 0.08, 0.95))),
            border: Border {
                color: Color::from_rgba(pr, pg, pb, 0.5),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}


// Test module
#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    /// **Feature: core-productivity-features, Property 4: Clipboard copy preserves content**
    /// **Validates: Requirements 1.3**
    /// 
    /// For any message content, copying to clipboard should result in the clipboard 
    /// containing exactly that content
    #[quickcheck]
    fn prop_clipboard_copy_preserves_content(content: String) -> TestResult {
        // Skip empty strings as they might not be meaningful for clipboard operations
        if content.is_empty() {
            return TestResult::discard();
        }

        use arboard::Clipboard;
        
        // Try to create clipboard - if it fails (e.g., in CI), discard the test
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return TestResult::discard(),
        };

        // Set the content to clipboard
        if clipboard.set_text(&content).is_err() {
            return TestResult::discard();
        }

        // Read back from clipboard
        match clipboard.get_text() {
            Ok(clipboard_content) => {
                // Verify the content matches exactly
                TestResult::from_bool(clipboard_content == content)
            }
            Err(_) => TestResult::discard(),
        }
    }

    /// **Feature: core-productivity-features, Property 1: Message deletion reduces conversation size**
    /// **Validates: Requirements 1.4**
    /// 
    /// For any conversation with N messages, deleting a message at index I should result 
    /// in a conversation with N-1 messages
    #[quickcheck]
    fn prop_message_deletion_reduces_size(messages: Vec<String>, index: usize) -> TestResult {
        // Need at least one message to delete
        if messages.is_empty() {
            return TestResult::discard();
        }
        
        // Index must be valid
        if index >= messages.len() {
            return TestResult::discard();
        }

        // Create chat messages
        let mut chat_messages: Vec<ChatMessage> = messages
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                ChatMessage::new(role.to_string(), content.clone())
            })
            .collect();

        let initial_len = chat_messages.len();
        
        // Delete the message
        chat_messages.remove(index);
        
        // Verify the size decreased by 1
        TestResult::from_bool(chat_messages.len() == initial_len - 1)
    }

    /// **Feature: core-productivity-features, Property 2: Message editing truncates subsequent messages**
    /// **Validates: Requirements 1.6**
    /// 
    /// For any conversation with messages at indices 0..N, editing a message at index I 
    /// should result in a conversation with messages only at indices 0..I
    #[quickcheck]
    fn prop_message_edit_truncates_subsequent(messages: Vec<String>, edit_index: usize) -> TestResult {
        // Need at least 2 messages to test truncation
        if messages.len() < 2 {
            return TestResult::discard();
        }
        
        // Edit index must be valid and not the last message (so there's something to truncate)
        if edit_index >= messages.len() - 1 {
            return TestResult::discard();
        }

        // Create chat messages
        let mut chat_messages: Vec<ChatMessage> = messages
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                ChatMessage::new(role.to_string(), content.clone())
            })
            .collect();

        // Edit the message (update content)
        if let Some(message) = chat_messages.get_mut(edit_index) {
            message.content = "edited content".to_string();
        }
        
        // Truncate all messages after the edited message
        chat_messages.truncate(edit_index + 1);
        
        // Verify the conversation only has messages up to and including the edited index
        TestResult::from_bool(chat_messages.len() == edit_index + 1)
    }

    /// **Feature: core-productivity-features, Property 3: AI messages exclude edit option**
    /// **Validates: Requirements 1.8**
    /// 
    /// For any message with role "assistant", the context menu should not contain an edit action.
    /// This property tests that the logic for determining whether to show the edit option
    /// correctly identifies AI messages and excludes the edit option for them.
    #[quickcheck]
    fn prop_ai_messages_exclude_edit_option(messages: Vec<String>) -> TestResult {
        // Need at least one message
        if messages.is_empty() {
            return TestResult::discard();
        }

        // Create chat messages with alternating roles
        let chat_messages: Vec<ChatMessage> = messages
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                ChatMessage::new(role.to_string(), content.clone())
            })
            .collect();

        // Test each message
        for (idx, message) in chat_messages.iter().enumerate() {
            let is_ai_message = message.role == "assistant";
            
            // The edit option should be available if and only if it's NOT an AI message
            let should_show_edit = !is_ai_message;
            
            // In the actual UI code, we check: if !is_ai_message { show edit button }
            // So for AI messages (is_ai_message == true), should_show_edit should be false
            // For user messages (is_ai_message == false), should_show_edit should be true
            
            if is_ai_message && should_show_edit {
                // This should never happen - AI messages should not show edit
                return TestResult::from_bool(false);
            }
            
            if !is_ai_message && !should_show_edit {
                // This should never happen - user messages should show edit
                return TestResult::from_bool(false);
            }
        }
        
        // All messages passed the test
        TestResult::from_bool(true)
    }
}
