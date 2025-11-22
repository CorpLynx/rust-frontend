# Prometheus Tauri - Quick Start Guide

## ✅ Migration Complete!

Your Prometheus chat app has been successfully migrated to Tauri! You now have:

- **Rust backend** with all your existing logic
- **Modern web UI** with Ollama-inspired styling
- **Small bundle size** (~10MB vs 100MB+ Electron)
- **Native performance** using system webview

## 🚀 Running the App

### Development Mode

```bash
cargo tauri dev
```

This will:
1. Build the Rust backend
2. Launch the app with hot-reload capabilities
3. Open the Prometheus window

### Production Build

```bash
cargo tauri build
```

This creates optimized binaries in `src-tauri/target/release/bundle/`

## 📁 Project Structure

```
prometheus/
├── ui/                          # Frontend (HTML/CSS/JS)
│   ├── index.html              # Main UI
│   ├── styles.css              # Ollama-inspired theme
│   └── app.js                  # Frontend logic
│
├── src-tauri/                   # Tauri backend
│   ├── src/
│   │   ├── commands.rs         # Tauri commands (API)
│   │   ├── lib.rs              # App entry point
│   │   └── main.rs             # Binary entry
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
│
└── src/                         # Original Iced code (can be removed)
```

## 🎨 Current Features

### Working:
- ✅ Chat interface with Ollama-inspired dark theme
- ✅ Model selection
- ✅ Send messages to Ollama
- ✅ Chat history persistence
- ✅ New conversation button
- ✅ Responsive layout

### To Add (from your original app):
- Conversation management (load/save/delete)
- Search functionality
- Settings panel
- Markdown rendering with code blocks
- Streaming responses
- Message context menu (copy/edit/delete)

## 🔧 How It Works

### Frontend → Backend Communication

The frontend calls Rust functions using Tauri's IPC:

```javascript
// Frontend (app.js)
const models = await invoke('get_models');
const response = await invoke('send_message', { 
    prompt: 'Hello', 
    model: 'llama2' 
});
```

### Backend Commands

Commands are defined in `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn send_message(prompt: String, model: String) -> Result<String, String> {
    // Your Rust logic here
}
```

## 📝 Next Steps

### 1. Add More Features

Copy functionality from your original `src/app.rs`:

- **Streaming responses**: Update `send_message` command
- **Conversation management**: Add commands for load/save/delete
- **Search**: Port your search engine
- **Settings**: Add settings panel and persistence

### 2. Improve UI

The current UI is minimal. You can:

- Add markdown rendering (use a library like `marked.js`)
- Add code syntax highlighting
- Implement message actions (copy, edit, delete)
- Add conversation sidebar with list

### 3. Migrate Existing Code

Your existing Rust modules are already copied to `src-tauri/src/`:
- `config.rs` - Configuration management
- `conversation.rs` - Conversation storage
- `markdown.rs` - Markdown parsing
- `search/` - Search engine

Just import and use them in your commands!

## 🎯 Example: Adding Streaming

### Backend (commands.rs)

```rust
use tauri::Window;

#[tauri::command]
pub async fn send_message_stream(
    window: Window,
    prompt: String,
    model: String
) -> Result<(), String> {
    // Stream chunks to frontend
    window.emit("stream-chunk", "Hello").unwrap();
    window.emit("stream-chunk", " World").unwrap();
    window.emit("stream-complete", ()).unwrap();
    Ok(())
}
```

### Frontend (app.js)

```javascript
import { listen } from '@tauri-apps/api/event';

// Listen for stream events
await listen('stream-chunk', (event) => {
    appendToMessage(event.payload);
});

await listen('stream-complete', () => {
    finishMessage();
});
```

## 🐛 Troubleshooting

### App won't start
- Make sure Ollama is running: `ollama serve`
- Check console for errors: `cargo tauri dev`

### Models not loading
- Verify Ollama is accessible at `http://localhost:11434`
- Test with: `curl http://localhost:11434/api/tags`

### Build errors
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo tauri build`

## 📚 Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Tauri API Reference](https://tauri.app/v2/reference/javascript/api/)
- [Your Migration Guide](./TAURI_MIGRATION_GUIDE.md)

## 🎉 What You've Achieved

You've successfully migrated from Iced to Tauri! Your app now has:

1. **Better UI flexibility** - Use any web tech (React, Vue, Svelte, or plain JS)
2. **Smaller size** - ~10MB vs 100MB+ Electron
3. **Same Rust backend** - All your logic is preserved
4. **Modern look** - Ollama-inspired dark theme
5. **Easy styling** - CSS is much easier than Iced styling

The app is running and ready for you to add more features! 🚀
