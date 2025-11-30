# Quick Reference

Fast lookup for common tasks and module responsibilities.

## Module Responsibilities (One-Liner)

| Module | What It Does |
|--------|--------------|
| **main.rs** | Entry point, parses args, routes to interactive/non-interactive mode |
| **app.rs** | Interactive REPL loop, handles commands and prompts |
| **backend.rs** | HTTP client for Ollama API, sends prompts, streams responses |
| **config.rs** | Loads/saves config.toml, provides defaults, validates settings |
| **conversation.rs** | Saves/loads conversations to JSON files, manages history |
| **commands.rs** | Parses slash commands (/help, /exit, /models, etc.) |
| **input.rs** | Validates prompts, reads files, combines input sources |
| **output.rs** | Formats output (quiet, JSON, verbose modes) |
| **mode.rs** | Detects interactive vs non-interactive execution mode |
| **non_interactive.rs** | Handles single-shot prompt execution and exit |
| **streaming.rs** | Processes streaming HTTP responses chunk-by-chunk |
| **markdown_renderer.rs** | Renders markdown with syntax highlighting in terminal |
| **terminal.rs** | Manages terminal state, raw mode, screen clearing |
| **url_validator.rs** | Validates URLs, enforces HTTPS for remote servers |
| **exit_codes.rs** | Defines exit codes, categorizes errors |
| **error.rs** | Custom error types and error handling |
| **update.rs** | Self-update via git pull and cargo rebuild |

## Common Tasks

### Quick Start with Local Ollama

Use the `/start-local` command for the easiest setup:

```bash
prometheus-cli
> /start-local
```

This command will:
1. Switch to localhost endpoint if needed
2. Check if Ollama is running
3. Start Ollama automatically if not running
4. Show available models
5. Let you select a model

### Switching Between Endpoints

Use `/switch` to change between different Ollama instances:

```bash
# Switch to local
> /switch local

# Switch to remote via Tailscale
> /switch https://comp-b.ts.net:11434

# Switch to saved endpoint
> /switch my-server
```

### Adding a New Command

1. Add variant to `Command` enum in `commands.rs`
2. Add parsing logic in `Command::parse()`
3. Add description in `Command::description()`
4. Handle command in `app.rs` `handle_command()` method
5. Update help text in `display_help()`

### Adding a New CLI Flag

1. Add field to `Args` struct in `main.rs`
2. Add clap attribute with help text
3. Pass value through to relevant handler
4. Update documentation in README.md

### Adding a New Output Format

1. Add format option to `NonInteractiveOptions` in `mode.rs`
2. Implement formatting in `output.rs`
3. Add CLI flag in `main.rs`
4. Update help text and documentation

### Adding a New Validation Rule

1. Add validation function to `url_validator.rs` or `input.rs`
2. Call validation in appropriate place (main.rs, config.rs, etc.)
3. Define error message
4. Add tests for validation logic

### Changing Backend API

1. Update request format in `backend.rs` `send_prompt_streaming()`
2. Update response parsing in `streaming.rs`
3. Update error handling
4. Test with new API

## File Locations

| What | Where |
|------|-------|
| Configuration | `config.toml` (root directory) |
| Conversations | `conversations/*.json` |
| Conversation metadata | `conversations/metadata.json` |
| Logs | `logs/` directory |
| Man page | `docs/prometheus-cli.1` |
| Source code | `prometheus-cli/src/` |
| Tests | `prometheus-cli/tests/` |
| Integration tests | `prometheus-cli/tests/*.rs` |

## Key Data Structures

### Conversation
```rust
{
  id: String,              // UUID
  name: String,            // Display name
  messages: Vec<Message>,  // Chat history
  created_at: String,      // ISO 8601 timestamp
  updated_at: String,      // ISO 8601 timestamp
  model: Option<String>,   // Model name
}
```

### ChatMessage
```rust
{
  role: String,      // "user" or "assistant"
  content: String,   // Message text
  timestamp: String, // ISO 8601 timestamp
}
```

### AppConfig
```rust
{
  app: {
    window_title: String,
    window_width: f32,
    window_height: f32,
  },
  backend: {
    url: String,           // Backend URL
    timeout_seconds: u64,  // Request timeout
    saved_urls: Vec<String>,
  },
  ui: {
    font_size: u32,
    max_chat_history: usize,
    theme: String,
  }
}
```

## Exit Codes

| Code | Meaning | Example Cause |
|------|---------|---------------|
| 0 | Success | Request completed |
| 1 | Invalid arguments | Empty prompt, bad temperature |
| 2 | Backend unreachable | Ollama not running |
| 3 | Authentication failed | Invalid API key |
| 4 | Model unavailable | Model not found |
| 5 | File error | File not found |
| 130 | Interrupted | Ctrl+C pressed |
| 143 | Terminated | SIGTERM received |

## Interactive Commands

| Command | Description |
|---------|-------------|
| `/help` | Show all commands |
| `/start-local` | Start local Ollama and switch to it |
| `/switch local` | Same as /start-local |
| `/switch <url>` | Switch to a specific endpoint URL |
| `/switch <name>` | Switch to a saved endpoint from config |
| `/exit` | Save and quit |
| `/quit` | Alias for /exit |
| `/new` | Start new conversation |
| `/models` | List available models |
| `/clear` | Clear terminal screen |
| `/update` | Update to latest version |
| `/update --check` | Check for updates |

## CLI Flags

### Connection
- `--url <URL>` - Backend URL (HTTPS for remote)
- `--model <NAME>` - Model to use
- `--config <FILE>` - Config file path

### Input
- `PROMPT` - Prompt text (enables non-interactive)
- `--file <PATH>` - Include file (repeatable)
- `--system <TEXT>` - System prompt

### Parameters
- `--temperature <FLOAT>` - Generation temperature (0.0-2.0)
- `--max-tokens <INT>` - Max response tokens

### Output
- `--quiet` - Response only
- `--json` - JSON output
- `--no-stream` - Wait for complete response
- `--verbose` - Debug information

### Behavior
- `--save-on-interrupt` - Save partial responses
- `--generate-completions <SHELL>` - Generate shell completions

## Configuration Precedence

1. **CLI arguments** (highest priority)
2. **config.toml** values
3. **Built-in defaults** (lowest priority)

Example: If you have `url = "http://localhost:11434"` in config.toml but run with `--url https://remote.com:11434`, the CLI argument wins.

## URL Validation Rules

### Remote URLs (not localhost)
- ✅ Must use HTTPS
- ❌ HTTP is rejected

### Localhost URLs
- ✅ HTTP allowed
- ✅ HTTPS allowed
- Localhost includes: `localhost`, `127.0.0.1`, `::1`

## Streaming Response Format

Ollama returns newline-delimited JSON:
```json
{"response": "Hello", "done": false}
{"response": " world", "done": false}
{"response": "!", "done": true}
```

The app accumulates "response" fields until "done": true.

## Error Handling Flow

```
Error occurs
  ↓
Categorize error type
  ↓
Map to exit code
  ↓
Format user message
  ↓
Log to stderr
  ↓
Exit with code
```

## Testing Commands

```bash
# Run all tests
cargo test -p prometheus-cli

# Run specific test
cargo test -p prometheus-cli test_name

# Run with output
cargo test -p prometheus-cli -- --nocapture

# Run integration tests
cargo test -p prometheus-cli --test integration_test_name
```

## Build Commands

```bash
# Debug build
cargo build -p prometheus-cli

# Release build
cargo build -p prometheus-cli --release

# Install locally
cargo install --path prometheus-cli

# Check without building
cargo check -p prometheus-cli
```

## Common Patterns

### Adding Async Function
```rust
async fn my_function() -> Result<String> {
    // Use .await for async operations
    let response = client.get(url).send().await?;
    Ok(response.text().await?)
}
```

### Error Handling
```rust
use anyhow::{Context, Result};

fn my_function() -> Result<()> {
    something()
        .context("Failed to do something")?;
    Ok(())
}
```

### Configuration Loading
```rust
let config = AppConfig::load()
    .unwrap_or_else(|_| AppConfig::default());
```

### Streaming Response
```rust
backend.send_prompt_streaming(prompt, model, |chunk| {
    print!("{}", chunk);
    Ok(())
}).await?
```

## Debugging Tips

### Enable Logging
```bash
RUST_LOG=debug prometheus-cli
```

### Check Backend Connection
```bash
curl http://localhost:11434/api/tags
```

### Validate Config
```bash
cat config.toml
```

### Check Conversation Files
```bash
ls -la conversations/
cat conversations/metadata.json
```

### Test Non-Interactive Mode
```bash
prometheus-cli "test" --verbose
```

## Performance Tips

1. **Use streaming** - Don't use `--no-stream` unless necessary
2. **Limit history** - Set `max_chat_history` in config
3. **Increase timeout** - For slow backends, increase `timeout_seconds`
4. **Use local backend** - Localhost is faster than remote

## Security Checklist

- [ ] Remote URLs use HTTPS
- [ ] Sensitive prompts not logged
- [ ] Config file has proper permissions
- [ ] Backend server is trusted
- [ ] SSL certificates are valid

## Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| Can't connect | Use `/start-local` command or check Ollama is running: `ollama serve` |
| Model not found | Pull model: `ollama pull llama2` |
| HTTPS error | Use HTTPS for remote, HTTP for localhost |
| Slow responses | Increase timeout in config.toml |
| Empty response | Check model is working: `ollama run llama2` |
| Ollama won't start | Check port 11434 is free: `lsof -i :11434` |
| No models available | Install a model: `ollama pull llama2` |

## Useful Links

- **Ollama API Docs:** https://github.com/ollama/ollama/blob/main/docs/api.md
- **Clap Docs:** https://docs.rs/clap/
- **Tokio Docs:** https://docs.rs/tokio/
- **Reqwest Docs:** https://docs.rs/reqwest/
