# Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Rust AI Chat Frontend                    │
│                         (Iced GUI)                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP POST
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Backend API Server                      │
│                  (http://localhost:8000)                     │
└─────────────────────────────────────────────────────────────┘
```

## Application Flow

```
┌──────────────┐
│   main.rs    │  Entry Point
└──────┬───────┘
       │
       │ 1. Initialize logger
       │ 2. Create logs directory
       │ 3. Load config.toml
       │
       ▼
┌──────────────┐
│  config.rs   │  Configuration
└──────┬───────┘
       │
       │ Load settings:
       │ - Window size
       │ - Backend URL
       │ - UI preferences
       │
       ▼
┌──────────────┐
│   app.rs     │  Main Application
└──────┬───────┘
       │
       │ Initialize:
       │ - ChatApp state
       │ - Load chat history
       │ - Setup GUI
       │
       ▼
┌──────────────────────────────────────────────────────────┐
│                    Application Loop                       │
│                                                           │
│  ┌─────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │   User      │───▶│   Message    │───▶│   Update   │ │
│  │   Input     │    │   Handler    │    │   State    │ │
│  └─────────────┘    └──────────────┘    └────────────┘ │
│         │                                       │        │
│         │                                       │        │
│         ▼                                       ▼        │
│  ┌─────────────┐                        ┌────────────┐ │
│  │    View     │◀───────────────────────│   Render   │ │
│  │   (GUI)     │                        │    GUI     │ │
│  └─────────────┘                        └────────────┘ │
└──────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. main.rs
**Responsibilities:**
- Application initialization
- Logger setup
- Configuration loading
- Window settings
- Launch Iced application

**Key Functions:**
```rust
fn main() -> Result<()>
```

### 2. config.rs
**Responsibilities:**
- Load configuration from `config.toml`
- Provide default values
- Manage settings structures

**Key Structures:**
```rust
struct AppConfig {
    app: AppSettings,
    backend: BackendSettings,
    ui: UISettings,
}
```

### 3. app.rs
**Responsibilities:**
- Main application logic
- GUI rendering
- Message handling
- Backend communication
- History management

**Key Structures:**
```rust
struct ChatApp {
    config: AppConfig,
    prompt_input: String,
    chat_history: Vec<ChatMessage>,
    is_loading: bool,
    error_message: Option<String>,
    dark_mode: bool,
    scroll_id: scrollable::Id,
}

enum Message {
    PromptChanged(String),
    SendPrompt,
    ResponseReceived(Result<String, String>),
    HistoryLoaded(Result<Vec<ChatMessage>, String>),
    ClearChat,
    ToggleDarkMode,
    CopyMessage(usize),
}
```

## Message Flow

### Sending a Message

```
User types message
       │
       ▼
PromptChanged(String)
       │
       ▼
User presses Enter/Send
       │
       ▼
SendPrompt
       │
       ├─▶ Add to chat_history
       ├─▶ Save to chat_history.json
       ├─▶ Clear input field
       ├─▶ Set is_loading = true
       └─▶ Send HTTP request
              │
              ▼
       Backend processes
              │
              ▼
ResponseReceived(Result)
       │
       ├─▶ Set is_loading = false
       ├─▶ Add response to chat_history
       ├─▶ Save to chat_history.json
       └─▶ Auto-scroll to bottom
```

### Feature Flows

#### Dark Mode Toggle
```
User clicks 🌙/☀️
       │
       ▼
ToggleDarkMode
       │
       ▼
dark_mode = !dark_mode
       │
       ▼
Re-render with new theme
```

#### Copy Message
```
User clicks 📋
       │
       ▼
CopyMessage(index)
       │
       ▼
Get message from chat_history[index]
       │
       ▼
Copy to clipboard via arboard
```

#### Clear Chat
```
User clicks Clear Chat
       │
       ▼
ClearChat
       │
       ├─▶ Clear chat_history vector
       ├─▶ Clear chat_history.json
       └─▶ Clear error_message
```

## Data Flow

### Persistence Layer

```
┌─────────────────┐
│   ChatApp       │
│   (Memory)      │
└────────┬────────┘
         │
         │ save_history()
         ▼
┌─────────────────┐
│ chat_history    │
│    .json        │
│   (Disk)        │
└────────┬────────┘
         │
         │ load_history()
         ▼
┌─────────────────┐
│   ChatApp       │
│   (Memory)      │
└─────────────────┘
```

### Backend Communication

```
┌─────────────────┐
│   ChatApp       │
└────────┬────────┘
         │
         │ send_request()
         ▼
┌─────────────────┐
│   Reqwest       │
│   HTTP Client   │
└────────┬────────┘
         │
         │ POST /generate
         ▼
┌─────────────────┐
│   Backend API   │
└────────┬────────┘
         │
         │ JSON Response
         ▼
┌─────────────────┐
│   Parse JSON    │
└────────┬────────┘
         │
         │ ResponseReceived
         ▼
┌─────────────────┐
│   ChatApp       │
└─────────────────┘
```

## GUI Structure

```
Container (Root)
│
└─▶ Column (Main Layout)
    │
    ├─▶ Row (Header)
    │   ├─▶ Text ("AI Chat Interface")
    │   ├─▶ Button (🌙/☀️ Dark Mode Toggle)
    │   └─▶ Button ("Clear Chat")
    │
    ├─▶ Container (Error Display)
    │   └─▶ Text (error message if any)
    │
    ├─▶ Scrollable (Chat History)
    │   └─▶ Column (Messages)
    │       └─▶ For each message:
    │           └─▶ Container (Message Box)
    │               └─▶ Column
    │                   ├─▶ Row (Header)
    │                   │   ├─▶ Text (Role: "You:" or "AI:")
    │                   │   ├─▶ Text (Timestamp)
    │                   │   └─▶ Button (📋 Copy)
    │                   └─▶ Text (Message Content)
    │
    └─▶ Row (Input Area)
        ├─▶ TextInput (Prompt Input)
        └─▶ Button ("Send" / "Sending...")
```

## State Management

### Application State

```rust
ChatApp {
    config: AppConfig,           // Loaded from config.toml
    prompt_input: String,         // Current input text
    chat_history: Vec<ChatMessage>, // All messages
    is_loading: bool,             // Request in progress?
    error_message: Option<String>, // Current error
    dark_mode: bool,              // Theme preference
    scroll_id: scrollable::Id,    // For auto-scroll
}
```

### Message State

```rust
ChatMessage {
    role: String,      // "user" or "assistant"
    content: String,   // Message text
    timestamp: String, // HH:MM:SS format
}
```

## Async Operations

### HTTP Request Flow

```
Main Thread (GUI)
       │
       │ Command::perform()
       ▼
Tokio Runtime (Async)
       │
       ├─▶ Create HTTP client
       ├─▶ Build request
       ├─▶ Send POST request
       ├─▶ Wait for response
       ├─▶ Parse JSON
       └─▶ Return Result
              │
              ▼
Main Thread (GUI)
       │
       ▼
ResponseReceived(Result)
       │
       ▼
Update UI
```

## Error Handling

```
Error Occurs
       │
       ├─▶ Log to console (log crate)
       ├─▶ Write to logs/error.log
       ├─▶ Set error_message in state
       └─▶ Display in GUI (red text)
```

## Dependencies Graph

```
rust-frontend
├── iced (GUI framework)
│   ├── tokio (async runtime)
│   └── wgpu (graphics)
├── reqwest (HTTP client)
│   └── tokio (async runtime)
├── serde + serde_json (serialization)
├── config (config file loading)
├── chrono (timestamps)
├── log + env_logger (logging)
├── arboard (clipboard)
└── anyhow (error handling)
```

## Performance Considerations

### Optimizations
1. **Async HTTP** - Non-blocking requests using Tokio
2. **Efficient Rendering** - Iced's retained mode GUI
3. **History Limits** - Configurable max_chat_history
4. **Lazy Loading** - Messages rendered on-demand
5. **Fast Clipboard** - Non-blocking arboard operations

### Memory Usage
- Chat history: ~1KB per message
- Default limit: 1000 messages = ~1MB
- GUI state: Minimal overhead
- Total: < 10MB typical usage

## Security Considerations

### Data Storage
- Chat history stored locally in plain text
- No encryption (consider adding for sensitive data)
- Logs may contain error details

### Network
- HTTP requests (not HTTPS by default)
- No authentication (configure in backend)
- Timeout protection (30s default)

### Clipboard
- Only copies on user action
- No automatic clipboard access
- Platform-specific permissions may apply

## Future Architecture Improvements

### Planned Enhancements
1. **Plugin System** - Extensible backend support
2. **Database** - SQLite for better history management
3. **Encryption** - Optional chat history encryption
4. **Streaming** - Server-sent events for real-time responses
5. **Multi-window** - Multiple conversation windows
6. **Themes** - Custom theme system

---

**Last Updated:** Current Session  
**Version:** 0.2.0
