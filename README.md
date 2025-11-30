# Prometheus

A modern AI chat application with multiple interfaces: a terminal-based CLI and a cross-platform desktop application.

## Overview

Prometheus provides flexible ways to interact with AI models through Ollama:

- **Prometheus CLI** - A streamlined terminal interface with real-time streaming, markdown rendering, and non-interactive mode for scripting
- **Prometheus Desktop** - A modern desktop application with a web-based UI, conversation management, and advanced features

Both interfaces share the same configuration format and conversation storage, allowing seamless switching between terminal and desktop environments.

## Quick Start

### Installation

**macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/your-username/prometheus/main/install.sh | bash
```

**Linux:**
```bash
curl -sSL https://raw.githubusercontent.com/your-username/prometheus/main/install-linux.sh | bash
```

**Manual installation:**

```bash
# Clone and build
git clone <repository-url>
cd prometheus
cargo build --release -p prometheus-cli

# Install binary
sudo cp target/release/prometheus-cli /usr/local/bin/
```

### First Run

1. **Start Ollama:**
   ```bash
   ollama serve
   ```

2. **Launch Prometheus CLI:**
   ```bash
   prometheus-cli
   ```

3. **Start chatting:**
   ```
   > What is Rust?
   ```

## Features

### CLI Application

- 🚀 **Fast startup** - Launches in under 500ms
- 💬 **Real-time streaming** - See AI responses as they're generated
- 📝 **Markdown rendering** - Syntax-highlighted code blocks and formatted text
- 💾 **Auto-save conversations** - All chats automatically saved with timestamps
- 🤖 **Non-interactive mode** - Perfect for scripts, automation, and command-line workflows
- ⚙️ **Flexible configuration** - Config file + CLI argument overrides
- ⌨️ **Interrupt handling** - Graceful Ctrl+C support during generation
- 🔧 **Built-in commands** - Model switching, conversation management, updates

### Desktop Application

- 🖥️ **Modern UI** - Web-based interface with rich styling
- 📚 **Conversation management** - Browse, search, and organize chat history
- 🎨 **Persona system** - Switch between different AI personas and behaviors
- 🔌 **Connection modes** - Support for local and remote Ollama instances
- ⚙️ **Settings UI** - Graphical configuration without editing files
- 🌐 **Cross-platform** - Runs on Windows, macOS, and Linux

## Usage

### Interactive Mode

```bash
# Basic usage
prometheus-cli

# With custom backend or model (HTTPS required for remote)
prometheus-cli --url https://my-ollama-server.com:11434 --model codellama

# Local development (HTTP allowed for localhost)
prometheus-cli --url http://localhost:11434 --model codellama

# Available commands
> /help                    # Show all commands
> /start-local             # Start local Ollama and switch to it
> /switch local            # Same as /start-local
> /switch <url>            # Switch to a specific endpoint
> /switch <name>           # Switch to a saved endpoint
> /new                     # Start new conversation
> /models                  # List available models
> /clear                   # Clear screen
> /exit                    # Save and quit
```

#### Security Requirements

**HTTPS Enforcement:** Remote backend URLs must use HTTPS to ensure your prompts and responses are encrypted in transit.

**Valid URLs:**
- `https://my-ollama-server.com:11434` (remote HTTPS)
- `https://api.example.com:8080` (remote HTTPS with custom port)
- `http://localhost:11434` (localhost development)
- `http://127.0.0.1:11434` (localhost development)

**Invalid URLs:**
- `http://remote-server.com:11434` (remote HTTP not allowed)
- `http://192.168.1.100:11434` (remote HTTP not allowed)

### Non-Interactive Mode

Perfect for scripts, automation, and command-line workflows:

```bash
# Simple question
prometheus-cli "What is the capital of France?"

# Using pipes
echo "Explain this error" | prometheus-cli

# File analysis
prometheus-cli --file main.rs "Review this code for bugs"

# Multiple files
prometheus-cli --file src/main.rs --file src/lib.rs "Summarize these modules"

# Output control
prometheus-cli --quiet "What is 2+2?"                    # Response only
prometheus-cli --json "Generate a haiku"                 # JSON format
prometheus-cli --no-stream "Write a long essay"          # Wait for complete response

# Model parameters
prometheus-cli --temperature 0.1 "Write precise documentation"
prometheus-cli --max-tokens 100 "Brief explanation"
prometheus-cli --system "You are a Python expert" "How do I parse JSON?"
```

### Advanced Examples

```bash
# Code review workflow with secure remote server
prometheus-cli --url https://my-ollama-server.com:11434 \
  --file src/main.rs --system "You are a senior developer" \
  --temperature 0.3 "Review this code for bugs and improvements"

# Data processing pipeline
cat data.csv | prometheus-cli --quiet --max-tokens 500 \
  "Summarize the key trends in this data" > summary.txt

# Batch processing with local development server
for file in *.py; do
  prometheus-cli --url http://localhost:11434 --file "$file" --quiet \
    "Rate this code quality 1-10" >> ratings.txt
done

# Error analysis with context using secure connection
tail -100 /var/log/app.log | prometheus-cli --url https://api.example.com:8080 \
  --system "You are a DevOps expert" "What's causing these errors and how to fix them?"

# Command substitution
commit_msg=$(prometheus-cli --quiet "Generate a git commit message for bug fixes")
git commit -m "$commit_msg"

# Remote server with custom port (HTTPS required)
prometheus-cli --url https://ollama.company.com:8443 "Explain quantum computing"
```

## Command-Line Arguments

| Argument | Short | Description | Example |
|----------|-------|-------------|---------|
| `PROMPT` | | Prompt text (enables non-interactive mode) | `"What is Rust?"` |
| `--url` | `-u` | Ollama backend URL (HTTPS required for remote) | `-u https://my-server.com:11434` |
| `--model` | `-m` | Model name to use | `-m llama2` |
| `--config` | `-c` | Configuration file path | `-c /path/to/config.toml` |
| `--file` | | Include file contents (repeatable) | `--file main.rs` |
| `--system` | | System prompt for context | `--system "You are helpful"` |
| `--temperature` | | Generation temperature (0.0-2.0) | `--temperature 0.7` |
| `--max-tokens` | | Maximum response tokens | `--max-tokens 500` |
| `--quiet` | `-q` | Output only response | `--quiet` |
| `--json` | | Output in JSON format | `--json` |
| `--no-stream` | | Wait for complete response | `--no-stream` |
| `--verbose` | `-v` | Include debug information | `--verbose` |
| `--save-on-interrupt` | | Save partial responses when interrupted | `--save-on-interrupt` |

## Exit Codes (Non-Interactive Mode)

| Exit Code | Meaning | Example Cause |
|-----------|---------|---------------|
| 0 | Success | Request completed successfully |
| 1 | Invalid arguments | Empty prompt, invalid temperature |
| 2 | Backend unreachable | Ollama server not running |
| 3 | Authentication failed | Invalid API key |
| 4 | Model unavailable | Model not found on server |
| 5 | File error | File not found, permission denied |
| 130 | Interrupted (SIGINT) | User pressed Ctrl+C |
| 143 | Terminated (SIGTERM) | Process killed |

## Configuration

### Configuration File

Create or edit `config.toml` in the project root:

```toml
[app]
window_title = "Prometheus v0.2.0"
window_width = 900.0
window_height = 650.0

[backend]
# For remote servers, use HTTPS (required for security)
url = "https://my-ollama-server.com:11434"
ollama_url = "https://my-ollama-server.com:11434"

# For local development, HTTP is allowed
# url = "http://localhost:11434"
# ollama_url = "http://localhost:11434"

timeout_seconds = 30

# Example saved URLs (remote URLs must use HTTPS)
saved_urls = [
    "https://api.openai.com/v1",
    "https://api.anthropic.com/v1",
    "https://my-ollama-server.com:11434",
    "http://localhost:11434"  # localhost exception
]

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
```

### Configuration Precedence

1. **Default values** - Built-in defaults
2. **Config file** - Values from `config.toml`
3. **CLI arguments** - Command-line flags (highest priority)

## Desktop Application

### Building and Running

```bash
# Navigate to the Tauri directory
cd src-tauri

# Development mode
cargo tauri dev

# Build for production
cargo build --release
```

### Features

- **Conversation Management** - Browse and search chat history
- **Persona System** - Switch between different AI personas
- **Settings UI** - Configure without editing files
- **Modern Interface** - Rich web-based UI with themes

## Troubleshooting

### Quick Start with Local Ollama

**Easiest way to get started:**

Use the `/start-local` command in interactive mode to automatically set up your local Ollama instance:

```bash
prometheus-cli
> /start-local
```

This command will:
- Switch your backend to localhost if needed
- Check if Ollama is running
- Automatically start Ollama if it's not running
- Show you available models to select from

### Switching Between Endpoints

Use the `/switch` command to easily change between different Ollama instances:

```bash
# Switch to local Ollama (same as /start-local)
> /switch local

# Switch to a specific URL
> /switch https://my-server.ts.net:11434

# Switch to a saved endpoint by name (from config.toml)
> /switch my-remote-server
```

The `/switch` command will:
- Validate the URL
- Connect to the new endpoint
- Fetch available models
- Let you select a model to use

### Connection Issues

**Problem:** `Failed to connect to http://localhost:11434`

**Solutions:**
1. Use the `/start-local` command to automatically start Ollama
2. Manually check if Ollama is running: `curl http://localhost:11434/api/tags`
3. Manually start Ollama: `ollama serve`
4. Verify URL: `prometheus-cli --url http://localhost:11434`

**Problem:** `Invalid backend URL protocol` or HTTPS-related errors

**Solutions:**
1. Use HTTPS for remote servers: `prometheus-cli --url https://my-server.com:11434`
2. For localhost development, HTTP is allowed: `prometheus-cli --url http://localhost:11434`
3. Check that remote URLs use HTTPS protocol
4. Verify SSL certificates are valid for HTTPS connections

### Ollama Service Issues

**Problem:** Ollama won't start or `/start-local` fails

**Solutions:**
1. **Ollama not installed:**
   - Install Ollama from https://ollama.ai
   - Verify installation: `which ollama`
   - Check version: `ollama --version`

2. **Port already in use:**
   - Check what's using port 11434: `lsof -i :11434` (macOS/Linux)
   - Kill the process or use a different port
   - Restart Ollama: `ollama serve`

3. **Permission issues:**
   - Ensure you have permission to run `ollama serve`
   - Check file permissions in Ollama's data directory
   - Try running with appropriate permissions

4. **Service not responding:**
   - Wait 10-15 seconds for Ollama to fully start
   - Check Ollama logs for errors
   - Restart Ollama manually: `pkill ollama && ollama serve`

5. **Timeout during startup:**
   - Ollama may be slow to start on first run
   - Try starting manually first: `ollama serve`
   - Wait for "Ollama is running" message
   - Then use `/start-local` in Prometheus CLI

### Model Issues

**Problem:** `Model 'xyz' not found` or no models available

**Solutions:**
1. List available models: `prometheus-cli` then `/models`
2. Pull a model: `ollama pull llama2`
3. Pull a smaller model for testing: `ollama pull llama2:7b`
4. Verify model name spelling
5. Check available models: `ollama list`

**Problem:** `/start-local` shows no models installed

**Solutions:**
1. Pull your first model: `ollama pull llama2`
2. Or pull a smaller model: `ollama pull llama2:7b`
3. Verify installation: `ollama list`
4. Run `/start-local` again to see the new models

### Non-Interactive Mode Issues

**Problem:** Non-interactive mode not activating

**Solutions:**
1. Ensure you provide a prompt: `prometheus-cli "Hello"` (not just `prometheus-cli --quiet`)
2. Check stdin detection: `echo "test" | prometheus-cli`
3. Verify file exists: `ls -la myfile.txt` before using `--file myfile.txt`

### Performance Issues

**Solutions:**
1. Use smaller models: `prometheus-cli --model llama2:7b`
2. Increase timeout: Edit `config.toml` and set `timeout_seconds = 60`
3. Check system resources: `top`
4. Ensure Ollama has sufficient RAM (8GB+ recommended for larger models)

## Development

### Repository Structure

```
prometheus/
├── prometheus-cli/          # CLI application
├── src-tauri/              # Desktop application
├── ui/                     # Web UI for desktop app
├── archived-iced-gui/      # Legacy GUI (archived)
├── config.toml             # Shared configuration
└── conversations/          # Shared conversation storage
```

### Building Components

```bash
# Build CLI only
cargo build -p prometheus-cli

# Build desktop app
cd src-tauri && cargo build

# Build everything
cargo build --workspace
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

See LICENSE file for details.

## Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - Technical design and system overview
- **[Changelog](docs/CHANGELOG.md)** - Version history and release notes
- **Man Page:** `man prometheus-cli` - Comprehensive command reference
- **Shell Completions:** Tab completion for all commands and options

### Shell Completions

Prometheus CLI supports auto-completion for all commands and options:

```bash
# Generate completions for your shell
prometheus-cli --generate-completions zsh > ~/.zsh/completions/_prometheus-cli
prometheus-cli --generate-completions bash > ~/.bash_completions/prometheus-cli

# Add to your shell config (zsh example)
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# Reload shell
source ~/.zshrc
```

Now you can use tab completion:
```bash
prometheus-cli --<TAB>          # Shows all options
prometheus-cli --url <TAB>      # URL completion
prometheus-cli --model <TAB>    # Model completion
```

## Support

- **In-App Help:** `/help` command in CLI
- **Man Page:** `man prometheus-cli` (comprehensive reference)
- **Issues:** GitHub Issues
- **Troubleshooting:** See sections above

---

**Choose your interface:** Terminal for speed and automation, Desktop for rich interaction and conversation management.