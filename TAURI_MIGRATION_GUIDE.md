# Tauri Migration Guide for Prometheus

## Overview

This guide outlines how to migrate your Iced-based Prometheus chat app to Tauri, giving you a modern web-based UI (like Ollama) while keeping your Rust backend logic.

## Why Tauri?

- **Small bundle size**: ~10MB vs 100MB+ for Electron
- **Keep your Rust code**: Backend logic stays in Rust
- **Modern UI**: Use React/Vue/Svelte or plain HTML/CSS/JS
- **Native performance**: Uses system webview, not bundled Chromium
- **Easy styling**: Can replicate Ollama's look exactly with CSS

## Architecture Comparison

### Current (Iced)
```
┌─────────────────────────┐
│   Iced GUI Application  │
│  (All-in-one Rust app)  │
└─────────────────────────┘
```

### After Tauri Migration
```
┌──────────────────────────┐
│   Web Frontend (UI)      │
│   HTML/CSS/JS/React      │
└───────────┬──────────────┘
            │ IPC
┌───────────▼──────────────┐
│   Rust Backend (Tauri)   │
│   - Ollama API calls     │
│   - Conversation mgmt    │
│   - Search engine        │
│   - Config management    │
└──────────────────────────┘
```

## Migration Steps

### 1. Install Tauri CLI

```bash
cargo install tauri-cli
```

### 2. Initialize Tauri in Your Project

```bash
cargo tauri init
```

When prompted:
- App name: `Prometheus`
- Window title: `Prometheus Chat`
- Web assets path: `../ui/dist` (or wherever your frontend builds to)
- Dev server URL: `http://localhost:5173` (if using Vite)
- Frontend dev command: `npm run dev` (or your build tool)
- Frontend build command: `npm run build`

### 3. Project Structure After Migration

```
prometheus/
├── src/                    # Rust backend (Tauri commands)
│   ├── main.rs            # Tauri app entry
│   ├── commands/          # Tauri command handlers
│   │   ├── chat.rs        # Chat/Ollama commands
│   │   ├── conversation.rs
│   │   └── search.rs
│   ├── config.rs          # Keep existing
│   ├── conversation.rs    # Keep existing
│   ├── search/            # Keep existing
│   └── markdown.rs        # Keep existing
├── ui/                    # Frontend (new)
│   ├── src/
│   │   ├── App.jsx        # Main UI component
│   │   ├── components/
│   │   │   ├── ChatWindow.jsx
│   │   │   ├── Sidebar.jsx
│   │   │   ├── Settings.jsx
│   │   │   └── MessageBubble.jsx
│   │   └── styles/
│   │       └── ollama-theme.css
│   ├── package.json
│   └── vite.config.js
├── src-tauri/             # Tauri config (generated)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
└── Cargo.toml             # Root workspace
```

### 4. Convert Rust Logic to Tauri Commands

Your existing Rust code becomes Tauri "commands" that the frontend can call.

#### Example: Chat Command

**src/commands/chat.rs**
```rust
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[tauri::command]
pub async fn send_message(
    prompt: String,
    model: String,
    backend_url: String,
) -> Result<String, String> {
    // Your existing send_request logic here
    // Return the response or error
    Ok("Response from Ollama".to_string())
}

#[tauri::command]
pub async fn get_models(ollama_url: String) -> Result<Vec<String>, String> {
    // Your existing fetch_models logic
    Ok(vec!["llama2".to_string(), "mistral".to_string()])
}

#[tauri::command]
pub fn get_chat_history() -> Result<Vec<ChatMessage>, String> {
    // Load from file or state
    Ok(vec![])
}
```

#### Updated main.rs

**src/main.rs**
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod conversation;
mod search;
mod markdown;

use commands::chat::{send_message, get_models, get_chat_history};
use commands::conversation::{load_conversations, save_conversation};
use commands::search::{search_conversations};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            send_message,
            get_models,
            get_chat_history,
            load_conversations,
            save_conversation,
            search_conversations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5. Create Frontend UI

#### Option A: React + Vite (Recommended)

```bash
cd ui
npm create vite@latest . -- --template react
npm install @tauri-apps/api
```

**ui/src/App.jsx**
```jsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './styles/ollama-theme.css';

function App() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState('');
  const [models, setModels] = useState([]);
  const [selectedModel, setSelectedModel] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Load models on startup
    invoke('get_models', { ollamaUrl: 'http://localhost:11434' })
      .then(setModels)
      .catch(console.error);
    
    // Load chat history
    invoke('get_chat_history')
      .then(setMessages)
      .catch(console.error);
  }, []);

  const sendMessage = async () => {
    if (!input.trim()) return;
    
    setLoading(true);
    const userMsg = { role: 'user', content: input, timestamp: new Date().toISOString() };
    setMessages(prev => [...prev, userMsg]);
    setInput('');

    try {
      const response = await invoke('send_message', {
        prompt: input,
        model: selectedModel || 'llama2',
        backendUrl: 'http://localhost:11434'
      });
      
      const aiMsg = { role: 'assistant', content: response, timestamp: new Date().toISOString() };
      setMessages(prev => [...prev, aiMsg]);
    } catch (error) {
      console.error('Error:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="app">
      <div className="sidebar">
        <h2>Prometheus</h2>
        <select value={selectedModel} onChange={e => setSelectedModel(e.target.value)}>
          {models.map(m => <option key={m} value={m}>{m}</option>)}
        </select>
      </div>
      
      <div className="chat-container">
        <div className="messages">
          {messages.map((msg, i) => (
            <div key={i} className={`message ${msg.role}`}>
              <div className="message-content">{msg.content}</div>
            </div>
          ))}
        </div>
        
        <div className="input-area">
          <input
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyPress={e => e.key === 'Enter' && sendMessage()}
            placeholder="Type a message..."
            disabled={loading}
          />
          <button onClick={sendMessage} disabled={loading}>
            {loading ? 'Sending...' : 'Send'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;
```

**ui/src/styles/ollama-theme.css**
```css
/* Ollama-inspired dark theme */
:root {
  --bg-primary: #0a0a0a;
  --bg-secondary: #1a1a1a;
  --bg-tertiary: #2a2a2a;
  --text-primary: #e0e0e0;
  --text-secondary: #a0a0a0;
  --accent: #00ff88;
  --user-msg-bg: #1e3a5f;
  --ai-msg-bg: #1a1a1a;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.app {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 250px;
  background: var(--bg-secondary);
  padding: 20px;
  border-right: 1px solid var(--bg-tertiary);
}

.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message {
  display: flex;
  max-width: 70%;
}

.message.user {
  align-self: flex-end;
}

.message.assistant {
  align-self: flex-start;
}

.message-content {
  padding: 12px 16px;
  border-radius: 12px;
  background: var(--bg-secondary);
}

.message.user .message-content {
  background: var(--user-msg-bg);
}

.input-area {
  display: flex;
  gap: 10px;
  padding: 20px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--bg-tertiary);
}

.input-area input {
  flex: 1;
  padding: 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--bg-tertiary);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
}

.input-area button {
  padding: 12px 24px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
}

.input-area button:hover {
  opacity: 0.9;
}

.input-area button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

### 6. Update Cargo.toml

**Cargo.toml**
```toml
[package]
name = "prometheus"
version = "0.2.0"
edition = "2021"

[dependencies]
# Remove iced
# iced = { version = "0.12", features = ["tokio"] }

# Add Tauri
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Keep your existing dependencies
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
config = "0.14"
anyhow = "1.0"
log = "0.4"
env_logger = "0.11"
chrono = "0.4"
toml = "0.8"
once_cell = "1.19"
uuid = { version = "1.6", features = ["v4", "serde"] }
regex = "1.10"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

### 7. Build and Run

```bash
# Development mode (hot reload)
cargo tauri dev

# Production build
cargo tauri build
```

## What You Keep

✅ All your Rust backend logic:
- `config.rs` - Configuration management
- `conversation.rs` - Conversation storage
- `search/` - Search engine
- `markdown.rs` - Markdown parsing
- Ollama API integration

## What Changes

🔄 UI Layer:
- Replace Iced widgets with HTML/CSS/JS
- Use Tauri IPC to call Rust functions
- More flexible styling (can match Ollama exactly)

## Benefits

1. **Smaller bundle**: ~10MB vs 100MB+ Electron
2. **Better UI flexibility**: Use any web framework or plain JS
3. **Keep Rust performance**: Backend stays fast
4. **Easy to style**: CSS is easier than Iced styling
5. **Hot reload**: Faster development iteration
6. **Cross-platform**: Works on macOS, Windows, Linux

## Next Steps

1. Install Tauri CLI
2. Initialize Tauri in your project
3. Create a simple frontend (start with plain HTML/CSS/JS)
4. Convert one feature at a time (start with chat)
5. Gradually add more features
6. Style to match Ollama's aesthetic

Want me to help you get started with the actual migration?
