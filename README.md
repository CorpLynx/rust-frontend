# Prometheus

A modern AI chat application with multiple interfaces: a terminal-based CLI and a cross-platform Tauri desktop application.

## Overview

Prometheus provides flexible ways to interact with AI models through Ollama:

- **Prometheus CLI** - A streamlined terminal REPL interface with real-time streaming, markdown rendering, and minimal dependencies. Perfect for server environments and SSH sessions.
- **Prometheus Tauri** - A modern desktop application with a web-based UI, providing a rich graphical interface with conversation management and advanced features.

Both interfaces share the same configuration format and conversation storage, allowing seamless switching between terminal and desktop environments.

## Features

### CLI Application

- 🚀 **Fast startup** - Launches in under 500ms
- 💬 **Real-time streaming** - See AI responses as they're generated
- 📝 **Markdown rendering** - Syntax-highlighted code blocks, formatted lists, and styled text
- 💾 **Auto-save conversations** - All chats automatically saved with timestamps
- ⚙️ **Flexible configuration** - Config file + CLI argument overrides
- 🎯 **Minimal dependencies** - Single binary, no external runtime required
- ⌨️ **Interrupt handling** - Graceful Ctrl+C support during generation
- 🔧 **Multiple commands** - Built-in commands for common operations

### Tauri Desktop Application

- 🖥️ **Modern UI** - Web-based interface with rich styling and animations
- 📚 **Conversation management** - Browse, search, and organize chat history
- 🎨 **Persona system** - Switch between different AI personas and behaviors
- 🔌 **Connection modes** - Support for local and remote Ollama instances
- ⚙️ **Settings UI** - Graphical configuration without editing files
- 🌐 **Cross-platform** - Runs on Windows, macOS, and Linux
- 🔍 **Search functionality** - Full-text search across conversations

## Installation

### Prerequisites

- Rust 1.70 or later
- Ollama running locally or accessible via network
- Node.js and npm (for Tauri UI development)

### Building from Source

#### CLI Application

```bash
# Clone the repository
git clone <repository-url>
cd prometheus

# Build the CLI binary
cargo build --release -p prometheus-cli

# The binary will be at target/release/prometheus-cli
# Optionally, copy it to your PATH
sudo cp target/release/prometheus-cli /usr/local/bin/
```

#### Tauri Desktop Application

```bash
# Navigate to the Tauri directory
cd src-tauri

# Build the Tauri application
cargo build --release

# Or use the Tauri CLI for development
cargo tauri dev
```

## Quick Start

1. **Start Ollama** (if not already running):
   ```bash
   ollama serve
   ```

2. **Launch Prometheus CLI**:
   ```bash
   prometheus-cli
   ```

3. **Start chatting**:
   ```
   > What is Rust?
   ```

4. **Exit when done**:
   ```
   > /exit
   ```

## Usage

### Basic Usage

```bash
# Use default configuration (config.toml)
prometheus-cli

# Specify custom backend URL
prometheus-cli --url http://localhost:11434

# Use a specific model
prometheus-cli --model llama2

# Combine options
prometheus-cli --url http://192.168.1.100:11434 --model codellama
```

### Command-Line Arguments

| Argument | Short | Description | Example |
|----------|-------|-------------|---------|
| `--url` | `-u` | Ollama backend URL (overrides config) | `-u http://localhost:11434` |
| `--model` | `-m` | Model name to use (overrides config) | `-m llama2` |
| `--config` | `-c` | Configuration file path | `-c /path/to/config.toml` |
| `--help` | `-h` | Display help information | `-h` |
| `--version` | `-V` | Display version information | `-V` |

### Interactive Commands

Commands are prefixed with `/` and are case-insensitive:

| Command | Description | Example |
|---------|-------------|---------|
| `/exit` | Save conversation and exit | `/exit` |
| `/quit` | Alias for /exit | `/quit` |
| `/clear` | Clear terminal screen (preserves history) | `/clear` |
| `/new` | Save current conversation and start new one | `/new` |
| `/help` | Display available commands | `/help` |
| `/models` | List available models from backend | `/models` |

## Configuration

### Configuration File

Prometheus CLI uses the same `config.toml` file as the GUI version. Create or edit `config.toml` in the project root:

```toml
[app]
window_title = "Prometheus v0.2.0"
window_width = 900.0
window_height = 650.0

[backend]
url = "http://localhost:11434"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = [
    "https://api.openai.com/v1",
    "http://192.168.1.100:8000"
]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
```

### Configuration Precedence

Configuration values are applied in the following order (later overrides earlier):

1. **Default values** - Built-in defaults
2. **Config file** - Values from `config.toml`
3. **CLI arguments** - Command-line flags (highest priority)

Example:
```bash
# config.toml has: ollama_url = "http://localhost:11434"
# This command will use http://192.168.1.100:11434 instead
prometheus-cli --url http://192.168.1.100:11434
```

### Configuration Options

#### Backend Settings

- `backend.ollama_url` - Ollama server URL (default: `http://localhost:11434`)
- `backend.timeout_seconds` - Request timeout in seconds (default: `30`)

#### Model Selection

The model can be specified via:
- CLI argument: `--model llama2`
- Default: `llama2` if not specified

## Common Workflows

### Starting a Chat Session

```bash
# Start with default settings
prometheus-cli

# You'll see:
# Prometheus CLI v0.2.0 - Terminal AI Chat
# Connected to: http://localhost:11434
# Model: llama2
# Type /help for available commands
#
# >
```

### Changing Models

```bash
# Method 1: Start with specific model
prometheus-cli --model codellama

# Method 2: Check available models during session
> /models
Available models:
  - llama2
  - codellama
  - mistral
  - llama2:13b

# Then restart with desired model
> /exit
prometheus-cli --model codellama
```

### Managing Conversations

```bash
# Start a new conversation (saves current one)
> /new
Started new conversation

# Conversations are automatically saved to:
# conversations/<timestamp>.json

# Clear screen without losing history
> /clear
```

### Interrupting Long Responses

```bash
# During a long response, press Ctrl+C
> Write a very long essay about...
[AI starts responding...]
^C
Response generation interrupted by user

# Partial response is saved to conversation
# You're returned to the prompt
>
```

### Connecting to Remote Ollama

```bash
# Connect to Ollama on another machine
prometheus-cli --url http://192.168.1.100:11434

# Or use SSH tunnel for secure connection
ssh -L 11434:localhost:11434 user@remote-server
prometheus-cli --url http://localhost:11434
```

## Markdown Rendering

Prometheus CLI renders markdown content with terminal formatting:

### Code Blocks

````
> Show me a Python function

```python
def hello_world():
    print("Hello, World!")
```
````

Rendered with:
- Syntax highlighting (if supported by terminal)
- Bordered boxes
- Distinct visual formatting

### Inline Code

```
> How do I use the `print` function?
```

Inline code appears with different color/style.

### Text Styling

- **Bold text** - Rendered with bold terminal formatting
- *Italic text* - Rendered with italic terminal formatting
- Lists - Proper indentation and bullet points

### Fallback Rendering

If advanced markdown rendering fails, raw markdown is displayed in a readable format.

## Conversation Management

### Automatic Saving

- Conversations are automatically saved after each message
- Files stored in `conversations/` directory
- Filename format: `<timestamp>.json`
- Example: `conversations/2024-11-23T14-30-45.json`

### Conversation Format

```json
{
  "id": "uuid-here",
  "name": "2024-11-23T14:30:45",
  "messages": [
    {
      "role": "user",
      "content": "Hello!",
      "timestamp": "2024-11-23T14:30:45Z"
    },
    {
      "role": "assistant",
      "content": "Hi! How can I help you?",
      "timestamp": "2024-11-23T14:30:46Z"
    }
  ],
  "created_at": "2024-11-23T14:30:45Z",
  "updated_at": "2024-11-23T14:30:46Z",
  "model": "llama2"
}
```

### Manual Conversation Management

```bash
# View saved conversations
ls -lh conversations/

# Read a conversation
cat conversations/2024-11-23T14-30-45.json | jq

# Delete old conversations
rm conversations/2024-11-*.json
```

## Signal Handling

### Ctrl+C Behavior

**At the prompt:**
```
> ^C
Received interrupt signal. Saving conversation and exiting...
Goodbye!
```

**During response generation:**
```
> Write a long essay...
[AI responding...]
^C
Response generation interrupted by user
>
```
- Streaming stops immediately
- Partial response is saved
- Returns to prompt for new input

### SIGTERM Handling

```bash
# Send SIGTERM to process
kill <pid>

# CLI saves conversation and exits cleanly
```

## Troubleshooting

### Connection Issues

**Problem:** `Failed to connect to http://localhost:11434`

**Solutions:**
1. Check if Ollama is running:
   ```bash
   curl http://localhost:11434/api/tags
   ```

2. Start Ollama if not running:
   ```bash
   ollama serve
   ```

3. Verify the URL in config or CLI args:
   ```bash
   prometheus-cli --url http://localhost:11434
   ```

### Timeout Errors

**Problem:** `Request timed out after 30s`

**Solutions:**
1. Increase timeout in config.toml:
   ```toml
   [backend]
   timeout_seconds = 60
   ```

2. Check network connectivity to backend
3. Try a smaller/faster model

### Model Not Found

**Problem:** `Model 'xyz' not found`

**Solutions:**
1. List available models:
   ```bash
   > /models
   ```

2. Pull the model with Ollama:
   ```bash
   ollama pull llama2
   ```

3. Verify model name spelling

### Configuration Loading Failures

**Problem:** `Warning: Failed to load config from config.toml`

**Solutions:**
1. Check if config.toml exists in current directory
2. Verify TOML syntax:
   ```bash
   cat config.toml
   ```

3. Use CLI arguments as override:
   ```bash
   prometheus-cli --url http://localhost:11434 --model llama2
   ```

4. CLI will use defaults if config fails to load

### Conversation Save Failures

**Problem:** `Error: Failed to save conversation`

**Solutions:**
1. Check write permissions:
   ```bash
   ls -ld conversations/
   ```

2. Create directory if missing:
   ```bash
   mkdir -p conversations
   ```

3. Check disk space:
   ```bash
   df -h .
   ```

### Terminal Display Issues

**Problem:** Markdown not rendering correctly

**Solutions:**
1. Ensure terminal supports ANSI colors:
   ```bash
   echo -e "\033[1;32mGreen\033[0m"
   ```

2. Try a different terminal emulator (iTerm2, Alacritty, etc.)
3. Raw markdown is displayed as fallback

### Performance Issues

**Problem:** Slow startup or response

**Solutions:**
1. Check backend response time:
   ```bash
   time curl http://localhost:11434/api/tags
   ```

2. Use a smaller model:
   ```bash
   prometheus-cli --model llama2:7b
   ```

3. Check system resources:
   ```bash
   top
   ```

## Advanced Usage

### Using with Different Backends

While designed for Ollama, Prometheus CLI can work with compatible APIs:

```bash
# OpenAI-compatible endpoint
prometheus-cli --url https://api.openai.com/v1

# Local LLM server
prometheus-cli --url http://localhost:8080
```

### Scripting and Automation

```bash
# Non-interactive usage (pipe input)
echo "What is 2+2?" | prometheus-cli

# Capture output
prometheus-cli --model llama2 < questions.txt > answers.txt
```

### Environment Variables

```bash
# Set default URL via environment
export OLLAMA_URL="http://localhost:11434"
prometheus-cli
```

## Performance

- **Startup time:** < 500ms on typical hardware
- **Memory footprint:** < 50MB during idle
- **Binary size:** ~10-15MB (statically linked)
- **Streaming latency:** Near real-time (depends on backend)

## Comparison: CLI vs Tauri

| Feature | CLI | Tauri Desktop |
|---------|-----|---------------|
| Startup time | < 500ms | ~1-2s |
| Memory usage | < 50MB | ~100-200MB |
| Conversation history | Auto-save | Full browsing & search |
| Markdown rendering | Terminal-based | Rich HTML |
| Multi-window | No | Yes |
| Server deployment | ✅ Ideal | ❌ Not suitable |
| Remote SSH | ✅ Perfect | ❌ Requires X11 |
| Persona management | Via config | Built-in UI |
| Settings | Config file + CLI args | Graphical settings panel |

## Contributing

Contributions are welcome! Please see the main project README for contribution guidelines.

## License

See the main project LICENSE file.

## Support

For issues, questions, or feature requests:
- Open an issue on GitHub
- Check existing documentation
- Review troubleshooting section above

## See Also

- [Archived Iced GUI](archived-iced-gui/README.md) - Legacy Iced-based GUI documentation
- [Architecture](docs/ARCHITECTURE.md) - System design
- [Changelog](docs/CHANGELOG.md) - Version history
- [Repository Structure](STRUCTURE.md) - Workspace organization explanation
