# How Prometheus CLI Works

This document explains the inner workings of Prometheus CLI in plain language, walking through what happens when you use the application.

## Starting the Application

### What happens when you run `prometheus-cli`?

1. **Parse Arguments** - The application reads your command-line arguments using the `clap` library
2. **Load Configuration** - Reads `config.toml` for settings like backend URL, timeout, etc.
3. **Detect Mode** - Determines if you want interactive or non-interactive mode
4. **Validate URL** - Checks that the backend URL is secure (HTTPS for remote, HTTP allowed for localhost)
5. **Route to Mode** - Launches either the interactive REPL or processes a single prompt

## Interactive Mode (REPL)

### The Main Loop

When you start interactive mode, the application enters a loop:

```
1. Display prompt: ">"
2. Wait for your input
3. When you press Enter:
   - Is it a command? (/help, /exit, etc.)
     → Execute the command
   - Is it a prompt?
     → Send to AI backend
4. Display response
5. Save to conversation history
6. Go back to step 1
```

### Sending a Prompt

Here's what happens when you type a message and press Enter:

1. **Validate Input** - Check that the prompt isn't empty
2. **Add to History** - Store your message in the conversation
3. **Create HTTP Request** - Build a JSON request for Ollama:
   ```json
   {
     "model": "llama2",
     "prompt": "Your message here",
     "stream": true
   }
   ```
4. **Send Request** - POST to `http://localhost:11434/api/generate`
5. **Stream Response** - As the AI generates text:
   - Receive chunks of JSON
   - Parse each chunk
   - Display text immediately (streaming)
   - Accumulate full response
6. **Render Markdown** - Format the response with syntax highlighting
7. **Save Response** - Add AI's response to conversation
8. **Write to Disk** - Save conversation to `conversations/{id}.json`

### Commands

Commands start with `/` and are handled differently:

- **/help** - Shows list of commands (no backend call)
- **/models** - Calls backend to list available models
- **/new** - Saves current conversation, starts fresh one
- **/clear** - Clears the terminal screen
- **/exit** - Saves conversation and quits
- **/update** - Checks for and installs updates

## Non-Interactive Mode

### Single-Shot Execution

When you provide a prompt as an argument, the app runs once and exits:

```bash
prometheus-cli "What is Rust?"
```

Here's the flow:

1. **Detect Mode** - Sees prompt argument, switches to non-interactive
2. **Build Prompt** - Combines:
   - Command-line prompt
   - Any `--file` contents
   - Any `--system` prompt
   - Any stdin input
3. **Validate** - Checks prompt, temperature, max_tokens
4. **Send Request** - Single HTTP request to backend
5. **Handle Response** - Based on flags:
   - Default: Show formatted response
   - `--quiet`: Show only response text
   - `--json`: Output as JSON
   - `--no-stream`: Wait for complete response
6. **Exit** - Return appropriate exit code (0 for success)

### Piping and Redirection

The app detects when output is redirected:

```bash
# Piping
echo "Explain this" | prometheus-cli

# Redirection
prometheus-cli "Generate text" > output.txt

# Both
cat file.txt | prometheus-cli "Summarize" > summary.txt
```

When output is redirected, the app automatically:
- Enables quiet mode (no extra formatting)
- Disables progress indicators
- Outputs only the response

## Backend Communication

### HTTP Requests

The app uses the `reqwest` library to make HTTP requests:

1. **Create Client** - Build HTTP client with timeout
2. **Build Request** - Create POST request with JSON body
3. **Send** - Make async HTTP call
4. **Handle Response** - Process streaming or complete response

### Streaming Protocol

Ollama returns responses as newline-delimited JSON:

```
{"response": "Hello", "done": false}
{"response": " world", "done": false}
{"response": "!", "done": true}
```

The app:
1. Reads each line
2. Parses JSON
3. Extracts "response" field
4. Displays immediately
5. Continues until "done": true

### Error Handling

If something goes wrong:

1. **Connection Error** - Can't reach backend
   - Check if Ollama is running
   - Verify URL is correct
   - Exit code: 2

2. **Model Not Found** - Model doesn't exist
   - List available models with `/models`
   - Pull model with `ollama pull`
   - Exit code: 4

3. **Timeout** - Request took too long
   - Increase timeout in config
   - Check backend performance
   - Exit code: 2

## Configuration System

### Loading Configuration

The app loads settings in this order (later overrides earlier):

1. **Built-in Defaults** - Hardcoded fallbacks
2. **config.toml** - Your configuration file
3. **CLI Arguments** - Command-line flags

Example:
```toml
[backend]
url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
theme = "Hacker Green"
```

### URL Validation

Before using any URL, the app validates it:

**Remote URLs (not localhost):**
- ✅ `https://my-server.com:11434` - HTTPS required
- ❌ `http://my-server.com:11434` - HTTP rejected

**Localhost URLs:**
- ✅ `http://localhost:11434` - HTTP allowed
- ✅ `http://127.0.0.1:11434` - HTTP allowed
- ✅ `https://localhost:11434` - HTTPS also works

This protects your prompts and responses from being intercepted.

## Conversation Storage

### File Structure

Conversations are stored in the `conversations/` directory:

```
conversations/
├── metadata.json              # List of all conversations
├── {uuid-1}.json             # Conversation 1
├── {uuid-2}.json             # Conversation 2
└── {uuid-3}.json             # Conversation 3
```

### Conversation Format

Each conversation file contains:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Chat 2024-01-15 10:30:00",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:35:00Z",
  "model": "llama2",
  "messages": [
    {
      "role": "user",
      "content": "What is Rust?",
      "timestamp": "2024-01-15T10:30:00Z"
    },
    {
      "role": "assistant",
      "content": "Rust is a systems programming language...",
      "timestamp": "2024-01-15T10:30:15Z"
    }
  ]
}
```

### Auto-Save

The app automatically saves:
- After each message exchange
- When starting a new conversation
- When exiting the application

## Terminal Rendering

### Markdown Support

The app renders markdown in the terminal:

- **Headers** - Colored and bold
- **Code blocks** - Syntax highlighted
- **Lists** - Properly indented
- **Links** - Underlined
- **Bold/Italic** - Styled text

### Syntax Highlighting

Code blocks are syntax highlighted based on language:

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

The app detects the language and applies appropriate colors.

### Terminal Control

The app uses `crossterm` to:
- Clear the screen
- Move the cursor
- Set colors
- Handle input
- Detect terminal size

## Async Operations

### Why Async?

The app uses async/await for:
- Non-blocking HTTP requests
- Concurrent file operations
- Responsive UI during long operations

### Tokio Runtime

The `#[tokio::main]` macro sets up an async runtime:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Async code here
}
```

This allows the app to:
- Make HTTP requests without blocking
- Handle multiple operations concurrently
- Remain responsive during AI generation

## Security Features

### HTTPS Enforcement

Remote backend URLs must use HTTPS:
- Encrypts prompts in transit
- Encrypts responses in transit
- Prevents man-in-the-middle attacks

### Input Validation

All inputs are validated:
- Prompts checked for emptiness
- Files checked for existence
- URLs checked for format
- Parameters checked for valid ranges

### Safe Defaults

The app uses secure defaults:
- HTTPS required for remote
- Reasonable timeouts
- No automatic command execution

## Update Mechanism

### Checking for Updates

The `/update --check` command:
1. Checks if app is in a git repository
2. Runs `git fetch` to get latest info
3. Compares local and remote versions
4. Reports if update is available

### Performing Updates

The `/update` command:
1. Validates installation (must be git repo)
2. Runs `git pull` to get latest code
3. Runs `cargo build --release` to rebuild
4. Reports success or failure

## Exit Codes

The app returns different exit codes for scripts:

- **0** - Success
- **1** - Invalid arguments (bad temperature, empty prompt, etc.)
- **2** - Backend unreachable (Ollama not running)
- **3** - Authentication failed
- **4** - Model not available
- **5** - File error (file not found, permission denied)
- **130** - Interrupted (Ctrl+C)
- **143** - Terminated (SIGTERM)

Scripts can check these codes:

```bash
prometheus-cli "test" || echo "Failed with code $?"
```

## Performance Optimizations

### Streaming Responses

Instead of waiting for the complete response:
- Display text as it arrives
- Lower perceived latency
- Better user experience

### Lazy Loading

Components are initialized only when needed:
- Backend client created on first use
- Conversations loaded on demand
- Models listed only when requested

### Efficient String Handling

The app minimizes string allocations:
- Uses string slices where possible
- Avoids unnecessary cloning
- Reuses buffers for streaming

## Common Workflows

### Basic Chat Session

```
1. Run: prometheus-cli
2. Type: "What is Rust?"
3. Read response
4. Type: "Give me an example"
5. Read response
6. Type: /exit
```

### Script Integration

```bash
# Generate commit message
msg=$(prometheus-cli --quiet "Generate a commit message for bug fixes")
git commit -m "$msg"

# Analyze log file
tail -100 app.log | prometheus-cli "What errors occurred?" > analysis.txt

# Batch processing
for file in *.py; do
  prometheus-cli --file "$file" --quiet "Rate code quality 1-10" >> ratings.txt
done
```

### Remote Server Usage

```bash
# Connect to secure remote server
prometheus-cli --url https://my-ollama-server.com:11434

# Use specific model
prometheus-cli --url https://my-server.com:11434 --model codellama
```

## Troubleshooting

### Connection Issues

**Problem:** "Failed to connect to http://localhost:11434"

**Solution:**
1. Check if Ollama is running: `curl http://localhost:11434/api/tags`
2. Start Ollama: `ollama serve`
3. Verify URL in config.toml

### Model Issues

**Problem:** "Model 'xyz' not found"

**Solution:**
1. List available models: `prometheus-cli` then `/models`
2. Pull the model: `ollama pull llama2`
3. Use correct model name

### HTTPS Issues

**Problem:** "Invalid backend URL protocol"

**Solution:**
1. Use HTTPS for remote servers: `--url https://server.com:11434`
2. For localhost, HTTP is allowed: `--url http://localhost:11434`
3. Check SSL certificates are valid

## Next Steps

Now that you understand how it works:

1. **Read the code** - Start with `main.rs` and follow the flow
2. **Experiment** - Try different commands and flags
3. **Modify** - Make small changes and see what happens
4. **Extend** - Add new features using the existing patterns

See `MODULE_GUIDE.md` for detailed information about each source file.
