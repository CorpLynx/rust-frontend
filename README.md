# Prometheus

A modern, cross-platform AI chat application built with Rust and Iced. Prometheus provides a powerful interface for interacting with AI backend services, featuring advanced search capabilities, conversation management, and a sleek cyberpunk-inspired design.

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Development](#development)
- [What Was Built](#what-was-built)
- [Next Steps](#next-steps)

## 📚 Additional Documentation

- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Quick reference card for common tasks
- **[FEATURES.md](FEATURES.md)** - Detailed description of new features added in Phase 9
- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - Step-by-step guide for using the new features
- **[STYLE_GUIDE.md](STYLE_GUIDE.md)** - Lo-fi hacker aesthetic design guide
- **[LAYOUT_UPDATE.md](LAYOUT_UPDATE.md)** - ChatGPT-style layout documentation
- **[OLLAMA_STYLE.md](OLLAMA_STYLE.md)** - Ollama-inspired design changes
- **[FINAL_LAYOUT.md](FINAL_LAYOUT.md)** - Final aligned message layout
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and technical details
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and roadmap
- **[SUMMARY.md](SUMMARY.md)** - Summary of latest changes
- **[BUILD_STATUS.md](BUILD_STATUS.md)** - Current build status and recent changes

## 🎯 Overview

Prometheus is a desktop GUI application that serves as a powerful frontend for AI chat services. Named after the Greek Titan who brought fire (knowledge) to humanity, Prometheus brings advanced AI capabilities to your desktop. It communicates with backend APIs via HTTP requests, features intelligent search indexing, and provides a modern user interface built with the Iced GUI framework.

**Design:** Features a **lo-fi hacker aesthetic** with neon green text, cyan accents, and a cyberpunk-inspired dark interface. The layout follows an **Ollama-style** design with user messages aligned right (lighter) and AI messages aligned left (darker), plus a floating input box at the bottom. Dark mode only for a consistent, focused interface.

**Key Technologies:**
- **Rust** - Systems programming language
- **Iced 0.12** - Cross-platform GUI framework
- **Tokio** - Asynchronous runtime
- **Reqwest** - HTTP client for backend communication
- **Serde** - Serialization/deserialization

## ✨ Features

### Core Functionality
- ✅ **Chat Interface** - Clean, scrollable chat history display
- ✅ **Input Field** - Text input for user prompts with Enter key support
- ✅ **Send Button** - Submit prompts to the AI backend
- ✅ **Loading States** - Visual feedback during request processing
- ✅ **Error Handling** - User-friendly error messages with detailed logging
- ✅ **Chat History** - Automatic persistence to `chat_history.json`
- ✅ **History Loading** - Automatic restoration of previous conversations on startup
- ✅ **Timestamps** - Each message includes a timestamp
- ✅ **Configurable Backend** - Easy configuration via `config.toml`
- ✅ **Dark Mode Only** - Consistent hacker aesthetic, always-on dark interface
- ✅ **Auto-scroll** - Automatically scrolls to the latest message
- ✅ **Copy Messages** - Copy any message to clipboard with the 📋 button
- ✅ **Clear Chat** - Clear all chat history with one click

### Technical Features
- ✅ **Asynchronous Operations** - Non-blocking HTTP requests using Tokio
- ✅ **Error Logging** - Detailed error logs saved to `logs/error.log`
- ✅ **Flexible JSON Parsing** - Supports multiple response formats from backend
- ✅ **Configurable UI** - Customizable window size, font size, and more
- ✅ **History Limits** - Configurable maximum chat history size

## 📁 Project Structure

```
prometheus/
├── Cargo.toml          # Rust project configuration and dependencies
├── Cargo.lock          # Locked dependency versions
├── config.toml         # Application configuration file
├── README.md           # This file - Main documentation
├── BUILD_STATUS.md     # Build status and recent changes
├── FEATURES.md         # Detailed feature descriptions (v0.2.0)
├── USAGE_GUIDE.md      # User guide for new features
├── CHANGELOG.md        # Version history and roadmap
├── SUMMARY.md          # Quick summary of latest changes
├── .gitignore          # Git ignore patterns
├── src/
│   ├── main.rs         # Application entry point
│   ├── app.rs          # Main application logic and GUI
│   └── config.rs       # Configuration loading and management
├── target/             # Build artifacts (generated)
├── logs/               # Error logs (generated)
└── chat_history.json   # Saved chat history (generated)
```

### File Purposes

#### `Cargo.toml`
Rust project manifest file that defines:
- Project metadata (name, version, edition)
- All dependencies and their versions
- Build configuration

**Key Dependencies:**
- `iced` - GUI framework with tokio support
- `reqwest` - HTTP client with JSON support
- `serde` / `serde_json` - Serialization
- `tokio` - Async runtime
- `config` - Configuration file loading
- `chrono` - Timestamp generation
- `log` / `env_logger` - Logging
- `arboard` - Clipboard access for copy functionality

#### `config.toml`
Configuration file for customizing the application:
- Window settings (title, size)
- Backend URL and timeout
- UI preferences (font size, history limits)

#### `src/main.rs`
Application entry point that:
- Initializes the logger
- Creates the logs directory
- Loads configuration
- Launches the Iced application with proper window settings

#### `src/config.rs`
Configuration management module that:
- Defines configuration structures (`AppConfig`, `AppSettings`, `BackendSettings`, `UISettings`)
- Loads configuration from `config.toml`
- Provides default values if config file is missing or invalid

#### `src/app.rs`
Main application logic containing:
- **ChatMessage** - Structure representing chat messages with role, content, and timestamp
- **Message** - Enum for application messages/events
- **ChatApp** - Main application struct implementing the Iced `Application` trait
- **GUI Components:**
  - Chat history display (scrollable)
  - Text input field
  - Send button
  - Error message display
- **Backend Communication:**
  - HTTP request handling
  - JSON parsing with flexible field support
  - Error handling and user feedback
- **History Management:**
  - Saving chat history to JSON
  - Loading history on startup
  - History size limiting

## 🔧 Prerequisites

### Required
1. **Rust** (latest stable version)
   - Install from: https://www.rust-lang.org/tools/install
   - Verify: `rustc --version` and `cargo --version`

2. **Microsoft C++ Build Tools** (Windows)
   - Required for compiling Rust on Windows
   - Install "Desktop development with C++" workload from Visual Studio Installer
   - Download: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

### Optional
- **Backend Server** - AI backend API running at the configured URL (default: `http://localhost:8000/generate`)

## 🚀 Installation

1. **Clone or navigate to the project directory:**
   ```powershell
   cd path/to/prometheus
   ```

2. **Build the project:**
   ```powershell
   cargo build
   ```

3. **Run the application:**
   ```powershell
   cargo run
   ```

The first build will download and compile all dependencies, which may take several minutes.

## ⚙️ Configuration

Edit `config.toml` to customize the application:

```toml
[app]
window_title = "Prometheus"     # Window title bar text
window_width = 800              # Initial window width in pixels
window_height = 600             # Initial window height in pixels

[backend]
url = "http://localhost:8000/generate"  # Backend API endpoint
timeout_seconds = 30                    # Request timeout in seconds

[ui]
font_size = 16                  # Base font size for UI elements
max_chat_history = 1000         # Maximum number of messages to keep in memory
```

### Backend API Requirements

The application expects the backend to:

**Accept POST requests** to the configured URL with JSON body:
```json
{
  "prompt": "user's prompt text here"
}
```

**Return JSON response** with one of these field names:
- `response`
- `text`
- `content`
- `message`

Example response:
```json
{
  "response": "AI generated response text"
}
```

## 💻 Usage

### Starting the Application

```powershell
cargo run
```

### Using the Interface

1. **Enter a prompt** in the text input field at the bottom
2. **Click "Send"** or press **Enter** to submit
3. **View responses** in the scrollable chat history area
4. **Error messages** (if any) appear in red above the chat area
5. **Copy messages** by clicking the ⎘ button next to any message
6. **Start new chat** by clicking the "New Chat" button in the header

### Chat History

- Chat history is **automatically saved** to `chat_history.json`
- History is **automatically loaded** when the application starts
- Messages include timestamps and role indicators (You: / AI:)

### Logs

Error logs are written to `logs/error.log` with timestamps for debugging purposes.

## 🛠️ Development

### Building

```powershell
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Running Tests

Currently no tests are implemented. This is a potential next step.

### Code Structure

The application follows a modular structure:
- **main.rs** - Entry point and initialization
- **config.rs** - Configuration management
- **app.rs** - Application logic, GUI, and backend communication

### Current Warnings

The project compiles with 2 non-critical warnings:
1. Unused `LoadHistory` variant (can be removed or kept for future use)
2. Lifetime syntax suggestion in `view()` function (cosmetic)

See `BUILD_STATUS.md` for details.

## 📝 What Was Built

### Phase 1: Project Setup
- ✅ Created Rust project structure
- ✅ Configured `Cargo.toml` with all required dependencies
- ✅ Set up basic project files

### Phase 2: Configuration System
- ✅ Created `config.toml` with default settings
- ✅ Implemented `config.rs` for loading and managing configuration
- ✅ Added support for window settings, backend URL, and UI preferences

### Phase 3: Core Application Logic
- ✅ Implemented `main.rs` with proper initialization
- ✅ Created `app.rs` with full Iced Application implementation
- ✅ Implemented chat message structure with timestamps
- ✅ Added message handling system

### Phase 4: GUI Implementation
- ✅ Built chat history display (scrollable)
- ✅ Created text input field with Enter key support
- ✅ Added Send button with loading state
- ✅ Implemented error message display
- ✅ Added basic styling (colors, fonts, spacing)

### Phase 5: Backend Integration
- ✅ Implemented asynchronous HTTP requests using Reqwest
- ✅ Added flexible JSON response parsing
- ✅ Implemented error handling for network and server errors
- ✅ Added timeout configuration

### Phase 6: History Management
- ✅ Implemented automatic chat history saving to JSON
- ✅ Added automatic history loading on startup
- ✅ Implemented history size limiting
- ✅ Added timestamp generation for messages

### Phase 7: Error Handling & Logging
- ✅ Implemented user-friendly error messages
- ✅ Added error logging to file
- ✅ Created logs directory management
- ✅ Added comprehensive error handling throughout

### Phase 8: Build Fixes
- ✅ Resolved linker issues (Visual Studio C++ Build Tools)
- ✅ Fixed dependency issues (added `serde_json`)
- ✅ Fixed type errors (changed error types to `String`)
- ✅ Fixed Iced API usage issues
- ✅ Resolved compilation errors

### Phase 9: UI Enhancements
- ✅ Implemented dark mode toggle with theme switching
- ✅ Added auto-scroll to bottom when new messages arrive
- ✅ Added copy-to-clipboard functionality for messages
- ✅ Added clear chat button to reset conversation
- ✅ Fixed unused `LoadHistory` variant warning
- ✅ Fixed lifetime syntax warning in `view()` function
- ✅ Added `arboard` dependency for clipboard support

## 🎯 Next Steps

### Immediate Improvements

1. **Testing**
   - Add unit tests for configuration loading
   - Add integration tests for backend communication
   - Test error handling scenarios

3. **Error Handling Enhancements**
   - Add retry logic for failed requests
   - Implement exponential backoff
   - Add connection status indicator

### Feature Enhancements

4. **UI Improvements**
   - Implement custom themes (beyond light/dark)
   - Add message deletion
   - Add message editing
   - Add syntax highlighting for code blocks
   - Add markdown rendering support

5. **Chat Features**
   - Add conversation export (markdown, plain text)
   - Implement conversation management (new, save, load)
   - Add message search functionality
   - Implement message editing

6. **Backend Features**
   - Add support for streaming responses
   - Implement multiple backend endpoints
   - Add model selection dropdown
   - Add request/response logging
   - Implement API key authentication

7. **Configuration**
   - Add GUI settings panel
   - Implement theme customization
   - Add font family selection
   - Add window position saving

8. **Advanced Features**
   - Add conversation templates/prompts
   - Implement conversation history search
   - Add keyboard shortcuts
   - Implement drag-and-drop file support
   - Add image support in chat

### Technical Improvements

9. **Code Quality**
   - Add comprehensive documentation comments
   - Refactor large functions into smaller ones
   - Add error types instead of using `String`
   - Implement proper error propagation

10. **Performance**
    - Optimize chat history rendering for large conversations
    - Implement virtual scrolling for very long histories
    - Add request caching
    - Optimize JSON parsing

11. **Cross-Platform**
    - Test on Linux
    - Test on macOS
    - Ensure consistent behavior across platforms

12. **Deployment**
    - Create release builds
    - Add installation scripts
    - Create distribution packages
    - Add auto-update mechanism

### Documentation

13. **Documentation**
    - Add API documentation
    - Create user guide
    - Add developer guide
    - Document backend API requirements in detail

## 🐛 Known Issues

None currently. The application builds and runs successfully.

## 📄 License

[Add your license here]

## 👤 Author

[Add your information here]

## 🙏 Acknowledgments

- Iced framework team for the excellent GUI framework
- Rust community for the amazing ecosystem
- All dependency maintainers

---

**Last Updated:** Current Session  
**Status:** ✅ Fully Functional - Ready for Use

