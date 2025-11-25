# Prometheus CLI Architecture

## Overview

Prometheus CLI is a terminal-based AI chat application built in Rust. It provides both interactive and non-interactive modes for communicating with Ollama AI models.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Prometheus CLI                           │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │ Interactive  │         │    Non-      │                 │
│  │    Mode      │         │ Interactive  │                 │
│  │   (REPL)     │         │    Mode      │                 │
│  └──────┬───────┘         └──────┬───────┘                 │
│         │                        │                          │
│         └────────┬───────────────┘                          │
│                  │                                          │
│         ┌────────▼────────┐                                │
│         │  Backend Client │                                │
│         └────────┬────────┘                                │
└──────────────────┼─────────────────────────────────────────┘
                   │
                   │ HTTPS (remote) / HTTP (localhost)
                   ▼
         ┌─────────────────┐
         │  Ollama Server  │
         │  (AI Backend)   │
         └─────────────────┘
```

## Module Organization

The application is organized into focused modules, each handling a specific concern:

### Core Modules

1. **main.rs** - Entry point and mode routing
2. **app.rs** - Interactive mode application logic
3. **backend.rs** - HTTP communication with Ollama
4. **config.rs** - Configuration management

### Input/Output Modules

5. **input.rs** - Input processing and validation
6. **output.rs** - Output formatting
7. **streaming.rs** - Streaming response handling
8. **markdown_renderer.rs** - Terminal markdown rendering

### Mode & Command Modules

9. **mode.rs** - Execution mode detection
10. **non_interactive.rs** - Non-interactive mode handler
11. **commands.rs** - Interactive command parsing

### Data & State Modules

12. **conversation.rs** - Conversation persistence
13. **terminal.rs** - Terminal state management

### Utility Modules

14. **url_validator.rs** - URL validation and security
15. **exit_codes.rs** - Exit code definitions
16. **error.rs** - Error types and handling
17. **update.rs** - Self-update functionality

## Data Flow

### Interactive Mode Flow

```
User Input
    │
    ▼
┌─────────────────┐
│  Terminal       │ Read line from stdin
│  (crossterm)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Command Parser │ Check if it's a command (/help, /exit, etc.)
└────────┬────────┘
         │
         ├─▶ Command? ──▶ Execute command
         │
         └─▶ Prompt? ──▶ Send to backend
                            │
                            ▼
                    ┌──────────────┐
                    │ Backend      │ HTTP POST to Ollama
                    │ Client       │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ Streaming    │ Process chunks as they arrive
                    │ Handler      │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ Markdown     │ Render formatted output
                    │ Renderer     │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ Conversation │ Save to disk
                    │ Manager      │
                    └──────────────┘
```

### Non-Interactive Mode Flow

```
CLI Arguments + stdin
    │
    ▼
┌─────────────────┐
│  Mode Detector  │ Detect non-interactive mode
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Input          │ Combine prompt + files + stdin
│  Processor      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Validation     │ Validate parameters
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Backend        │ Send single request
│  Client         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Output         │ Format based on flags (--quiet, --json)
│  Formatter      │
└────────┬────────┘
         │
         ▼
    Exit with code
```

## Key Design Patterns

### 1. Mode Detection Pattern

The application automatically detects whether to run in interactive or non-interactive mode based on:
- Presence of prompt argument
- stdin availability
- Output flags (--quiet, --json, etc.)

This allows seamless use in both human interaction and scripting contexts.

### 2. Streaming Response Pattern

Responses from the AI are streamed chunk-by-chunk rather than waiting for completion:
- Lower perceived latency
- Better user experience
- Ability to interrupt long responses

### 3. Configuration Precedence Pattern

Settings are resolved in order of precedence:
1. CLI arguments (highest priority)
2. Config file values
3. Built-in defaults (lowest priority)

This provides flexibility while maintaining sensible defaults.

### 4. Security-First URL Validation

All backend URLs are validated before use:
- Remote URLs must use HTTPS
- Localhost URLs can use HTTP (development)
- Invalid URLs are rejected with helpful error messages

### 5. Conversation Persistence Pattern

Conversations are automatically saved:
- Each conversation has a unique ID
- Messages are appended incrementally
- Metadata is maintained separately for fast listing

## Error Handling Strategy

The application uses a layered error handling approach:

```
Application Layer
    │
    ├─▶ User-friendly messages (stderr)
    │
    ▼
Error Categorization Layer
    │
    ├─▶ Exit codes (for scripts)
    │
    ▼
Error Source Layer
    │
    ├─▶ Backend errors
    ├─▶ File system errors
    ├─▶ Validation errors
    └─▶ Network errors
```

Each error type maps to a specific exit code, making the CLI script-friendly.

## Performance Considerations

### Async I/O
- All network operations use Tokio async runtime
- Non-blocking HTTP requests
- Concurrent file operations where possible

### Memory Management
- Streaming responses avoid buffering entire responses
- Conversation history has configurable limits
- Efficient string handling with zero-copy where possible

### Startup Time
- Lazy initialization of components
- Config loading is fast (TOML parsing)
- No unnecessary network calls at startup

## Security Features

### HTTPS Enforcement
- Remote backend URLs must use HTTPS
- Prevents man-in-the-middle attacks
- Protects sensitive prompts and responses

### Input Validation
- All user inputs are validated
- File paths are checked before reading
- URL formats are strictly validated

### Safe Defaults
- Secure defaults in configuration
- No automatic execution of commands
- Clear error messages for security issues

## Extension Points

The architecture is designed to be extensible:

1. **New Commands** - Add to `commands.rs`
2. **New Output Formats** - Extend `output.rs`
3. **New Backends** - Implement in `backend.rs`
4. **New Validators** - Add to `url_validator.rs`

## Dependencies

### Core Dependencies
- **tokio** - Async runtime
- **reqwest** - HTTP client
- **serde/serde_json** - Serialization
- **anyhow** - Error handling

### CLI Dependencies
- **clap** - Argument parsing
- **crossterm** - Terminal control
- **termimad** - Markdown rendering

### Utility Dependencies
- **chrono** - Timestamps
- **uuid** - Unique IDs
- **config** - Config file parsing
- **url** - URL parsing

## Testing Strategy

The codebase includes multiple testing approaches:

1. **Unit Tests** - Test individual functions
2. **Property Tests** - Test invariants with QuickCheck
3. **Integration Tests** - Test end-to-end flows
4. **Mock Tests** - Test with mock HTTP servers

See individual module documentation for specific test coverage.
