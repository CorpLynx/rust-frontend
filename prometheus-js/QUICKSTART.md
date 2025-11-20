# Quick Start Guide

Get Prometheus JS running in 3 steps:

## 1. Install Dependencies

```bash
cd prometheus-js
npm install
```

## 2. Make Sure Ollama is Running

```bash
# Start Ollama (if not already running)
ollama serve

# In another terminal, verify you have models
ollama list

# If no models, pull one
ollama pull llama2
```

## 3. Run Prometheus

```bash
npm start
```

That's it! You should see the Prometheus ASCII art and be ready to chat.

## First Time Setup

The app will automatically:
- Create a `conversations/` directory for saving chats
- Create a `logs/` directory for error logs
- Load default configuration from `config.json`
- Fetch available models from Ollama
- Create a new conversation

## Tips

- Press **Tab** to switch between input field and send button
- Press **Enter** to send your message
- Press **Esc** or **q** to quit
- Scroll with your mouse or arrow keys in the chat area

## Customization

Edit `config.json` to change:
- Theme colors
- Backend URL
- Timeout settings
- UI preferences

## Troubleshooting

**"Failed to fetch models"**
- Make sure Ollama is running: `ollama serve`
- Check the URL in config.json matches your Ollama instance

**"No model selected"**
- Pull a model: `ollama pull llama2`
- Restart the app

**Display looks weird**
- Use a modern terminal (iTerm2, Windows Terminal, etc.)
- Ensure your terminal supports Unicode
- Try resizing your terminal window

## Next Steps

- Check out the full [README.md](README.md) for more details
- Explore the code in `src/` to customize behavior
- Try different themes in `config.json`
