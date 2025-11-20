# Prometheus JS

A modern, terminal-based AI chat application built with Node.js and Blessed. This is a JavaScript port of the Rust version, offering the same core functionality with easier development and faster iteration.

## Features

- 🚀 **Terminal UI** - Clean, responsive interface using Blessed
- 💬 **Streaming Responses** - Real-time AI responses with streaming support
- 📝 **Markdown Rendering** - Code blocks, inline code, bold, italic, lists
- 💾 **Conversation Management** - Save and load chat conversations
- 🎨 **Multiple Themes** - Hacker Green, Cyber Blue, Neon Purple, Matrix Red
- ⚙️ **Configurable** - Easy configuration via JSON file
- 🔌 **Ollama Integration** - Works with local Ollama models

## Prerequisites

- **Node.js** (v18 or higher)
- **Ollama** (running locally or accessible via network)

## Installation

1. Navigate to the project directory:
```bash
cd prometheus-js
```

2. Install dependencies:
```bash
npm install
```

3. Configure your backend (optional):
Edit `config.json` to set your Ollama URL and preferences.

## Usage

Start the application:
```bash
npm start
```

Or use watch mode for development:
```bash
npm run dev
```

### Controls

- **Type** your message in the input box
- **Enter** to send the message
- **Tab** to switch focus between input and send button
- **Esc** or **q** to quit
- **Mouse scroll** to scroll through chat history

## Configuration

Edit `config.json`:

```json
{
  "app": {
    "windowTitle": "Prometheus",
    "theme": "Hacker Green"
  },
  "backend": {
    "url": "http://localhost:1234",
    "ollamaUrl": "http://localhost:11434",
    "timeoutSeconds": 30,
    "savedUrls": []
  },
  "ui": {
    "fontSize": 16,
    "maxChatHistory": 1000
  }
}
```

### Available Themes

- `Hacker Green` (default) - Classic terminal green
- `Cyber Blue` - Bright blue cyberpunk
- `Neon Purple` - Purple neon aesthetic
- `Matrix Red` - Red matrix style

## Project Structure

```
prometheus-js/
├── src/
│   ├── index.js          # Main application
│   ├── config.js         # Configuration management
│   ├── conversation.js   # Conversation storage
│   ├── api.js           # Ollama API client
│   └── markdown.js      # Markdown parser
├── conversations/        # Saved conversations (generated)
├── config.json          # Configuration file
├── package.json         # Dependencies
└── README.md           # This file
```

## Comparison with Rust Version

### Advantages of JS Version:
- ✅ Faster development and iteration
- ✅ Easier to modify and extend
- ✅ Simpler dependency management (npm)
- ✅ More accessible to JavaScript developers
- ✅ Smaller codebase

### Advantages of Rust Version:
- ✅ Better performance
- ✅ Lower memory usage
- ✅ Single compiled binary
- ✅ No runtime dependencies
- ✅ Type safety at compile time

## Development

The codebase is organized into modules:

- **index.js** - Main UI and application logic
- **config.js** - Configuration loading and theme management
- **conversation.js** - Conversation persistence
- **api.js** - Ollama API integration with streaming
- **markdown.js** - Simple markdown parser for terminal

## Troubleshooting

### Models not loading
- Ensure Ollama is running: `ollama serve`
- Check the `ollamaUrl` in `config.json`
- Verify you have models installed: `ollama list`

### Display issues
- Ensure your terminal supports Unicode
- Try a different terminal emulator
- Adjust terminal size (minimum 80x24 recommended)

### Connection errors
- Check that Ollama is accessible at the configured URL
- Verify firewall settings if using remote Ollama

## License

MIT

## Credits

- Built with [Blessed](https://github.com/chjj/blessed) for terminal UI
- Inspired by the Rust version of Prometheus
- Designed for use with [Ollama](https://ollama.ai/)
