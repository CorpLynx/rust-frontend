# Quick Reference Card

## 🚀 Getting Started

```bash
# Build the project
cargo build

# Run the application
cargo run

# Build optimized release version
cargo build --release
```

## 🎮 Controls

| Action | How To |
|--------|--------|
| Send message | Type and press **Enter** or click **Send** |
| Toggle dark mode | Click **🌙** (light) or **☀️** (dark) button |
| Copy message | Click **📋** button next to any message |
| Clear all chat | Click **Clear Chat** button in header |
| Scroll chat | Automatic (or use mouse wheel) |

## 📋 Features at a Glance

### ✅ Core Features
- Chat interface with AI backend
- Message history persistence
- Configurable backend URL
- Error handling and logging
- Timestamps on all messages

### ✨ New in v0.2.0
- 🌙 Dark mode toggle
- 📜 Auto-scroll to latest message
- 📋 Copy messages to clipboard
- 🗑️ Clear chat button

## ⚙️ Configuration

Edit `config.toml`:

```toml
[app]
window_title = "AI Chat"
window_width = 800
window_height = 600

[backend]
url = "http://localhost:8000/generate"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
```

## 🔧 Backend API Format

**Request:**
```json
POST /generate
{
  "prompt": "your message here"
}
```

**Response:**
```json
{
  "response": "AI response text"
}
```

Supported response fields: `response`, `text`, `content`, or `message`

## 📁 Important Files

| File | Purpose |
|------|---------|
| `config.toml` | App configuration |
| `chat_history.json` | Saved conversations |
| `logs/error.log` | Error logs |
| `src/app.rs` | Main application code |

## 🐛 Troubleshooting

| Problem | Solution |
|---------|----------|
| Backend connection fails | Check URL in `config.toml` and ensure backend is running |
| Copy doesn't work | Check clipboard permissions |
| Build fails | Run `cargo clean && cargo build` |
| Window doesn't appear | Check graphics drivers |

## 📚 Documentation

| Document | Description |
|----------|-------------|
| `README.md` | Complete documentation |
| `FEATURES.md` | Detailed feature descriptions |
| `USAGE_GUIDE.md` | Step-by-step usage guide |
| `CHANGELOG.md` | Version history |
| `BUILD_STATUS.md` | Current build status |
| `SUMMARY.md` | Latest changes summary |

## 🎯 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Enter** | Send message |

*More shortcuts coming in future versions!*

## 💡 Tips

1. **Use dark mode** in low-light conditions
2. **Copy important responses** before clearing chat
3. **Check logs** if you encounter errors
4. **Configure backend URL** in `config.toml`
5. **Chat history persists** between sessions

## 🔄 Version Info

- **Current Version:** 0.2.0
- **Rust Edition:** 2021
- **Iced Version:** 0.12
- **Build Status:** ✅ Successful

## 📞 Getting Help

1. Check `logs/error.log` for errors
2. Review documentation files
3. Verify `config.toml` settings
4. Ensure backend is running
5. Try rebuilding the project

---

**Quick Tip:** Press **Enter** to send messages quickly! 🚀
