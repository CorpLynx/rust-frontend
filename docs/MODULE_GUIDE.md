# Module Guide

This guide explains each source file in the Prometheus CLI application, what it does, and how it fits into the overall system.

## Entry Point

### main.rs
**Purpose:** Application entry point and mode routing

**What it does:**
- Parses command-line arguments using clap
- Detects whether to run in interactive or non-interactive mode
- Validates URLs before creating backend connections
- Routes execution to the appropriate mode handler
- Handles top-level errors and exit codes

**Key functions:**
- `main()` - Entry point, parses args and routes to mode
- `run_interactive_mode()` - Launches the interactive REPL
- `run_non_interactive_mode()` - Processes single prompt and exits

**When you'd modify this:**
- Adding new top-level CLI flags
- Changing application initialization logic
- Adding new execution modes

---

## Core Application

### app.rs
**Purpose:** Interactive mode application logic

**What it does:**
- Manages the interactive REPL (Read-Eval-Print Loop)
- Handles user input in a loop
- Processes commands like /help, /exit, /models
- Sends prompts to the backend
- Displays streaming responses
- Manages conversation state
- Auto-saves conversations

**Key structures:**
- `CliApp` - Main application state
  - `config` - Application configuration
  - `backend` - Backend client for API calls
  - `conversation` - Current conversation
  - `conversation_manager` - Handles saving/loading

**Key methods:**
- `new()` - Creates new app instance
- `run()` - Main REPL loop
- `handle_command()` - Processes slash commands
- `send_prompt()` - Sends prompt to backend
- `display_response()` - Shows AI response

**When you'd modify this:**
- Adding new interactive commands
- Changing the REPL behavior
- Modifying how responses are displayed
- Adding new interactive features

---

## Backend Communication

### backend.rs
**Purpose:** HTTP communication with Ollama server

**What it does:**
- Creates HTTP client with proper timeouts
- Validates backend URLs for security
- Sends prompts to Ollama API
- Handles streaming responses
- Lists available models
- Manages connection errors

**Key structures:**
- `BackendClient` - HTTP client wrapper
  - `client` - reqwest HTTP client
  - `base_url` - Ollama server URL
  - `timeout` - Request timeout duration

**Key methods:**
- `new()` - Creates client with URL validation
- `send_prompt_streaming()` - Sends prompt, streams response
- `list_models()` - Gets available models from server
- `check_connection()` - Tests if server is reachable

**When you'd modify this:**
- Supporting new Ollama API endpoints
- Changing request/response format
- Adding authentication
- Implementing retry logic

---

## Configuration

### config.rs
**Purpose:** Configuration management

**What it does:**
- Loads configuration from config.toml
- Provides default values
- Validates configuration settings
- Manages saved URLs list
- Filters invalid URLs from config
- Handles configuration persistence

**Key structures:**
- `AppConfig` - Top-level config
  - `app` - App settings (window title, etc.)
  - `backend` - Backend settings (URL, timeout)
  - `ui` - UI settings (font size, theme)

**Key methods:**
- `load()` - Loads config from file
- `default()` - Provides default configuration
- `validate_urls()` - Filters invalid URLs
- `save()` - Persists config to disk

**When you'd modify this:**
- Adding new configuration options
- Changing default values
- Adding config validation rules
- Supporting new config file formats

---

## Input Processing

### input.rs
**Purpose:** Input processing and validation

**What it does:**
- Validates user prompts
- Reads file contents for --file flag
- Combines multiple input sources
- Validates parameters (temperature, max_tokens)
- Builds final prompt from components
- Checks for empty or invalid input

**Key functions:**
- `validate_prompt()` - Ensures prompt is valid
- `validate_parameters()` - Checks temperature, tokens
- `read_file_content()` - Reads file for inclusion
- `build_prompt()` - Combines prompt + files + system prompt

**When you'd modify this:**
- Adding new input sources
- Changing validation rules
- Supporting new file formats
- Adding input preprocessing

---

### mode.rs
**Purpose:** Execution mode detection

**What it does:**
- Detects interactive vs non-interactive mode
- Checks for prompt arguments
- Detects stdin input
- Determines output flags
- Creates mode configuration

**Key structures:**
- `ExecutionMode` - Enum of possible modes
  - `Interactive` - REPL mode
  - `NonInteractive` - Single-shot mode
- `NonInteractiveOptions` - Flags for non-interactive mode

**Key functions:**
- `detect_mode()` - Analyzes args and environment
- `is_stdin_available()` - Checks if stdin has data

**When you'd modify this:**
- Adding new execution modes
- Changing mode detection logic
- Adding new mode-specific options

---

### non_interactive.rs
**Purpose:** Non-interactive mode handler

**What it does:**
- Processes single prompt and exits
- Handles --quiet, --json, --no-stream flags
- Manages output formatting
- Handles interruption (Ctrl+C)
- Optionally saves to conversation history
- Returns appropriate exit codes

**Key structures:**
- `NonInteractiveHandler` - Manages non-interactive execution
  - `backend` - Backend client
  - `conversation_manager` - For saving history

**Key methods:**
- `new()` - Creates handler with config
- `process_prompt()` - Main processing function
- `handle_response()` - Formats and outputs response

**When you'd modify this:**
- Adding new output formats
- Changing non-interactive behavior
- Adding new flags
- Modifying exit code logic

---

## Output & Display

### output.rs
**Purpose:** Output formatting

**What it does:**
- Formats output based on flags
- Handles --quiet mode (response only)
- Handles --json mode (JSON output)
- Detects output redirection
- Adjusts formatting for pipes
- Manages verbose output

**Key structures:**
- `OutputFormatter` - Handles output formatting
- `JsonOutput` - Structure for JSON output

**Key methods:**
- `format_response()` - Formats based on options
- `format_json()` - Creates JSON output
- `adjust_for_redirection()` - Detects pipes/redirects

**When you'd modify this:**
- Adding new output formats
- Changing JSON structure
- Adding output filters
- Customizing formatting

---

### streaming.rs
**Purpose:** Streaming response handling

**What it does:**
- Processes streaming HTTP responses
- Accumulates chunks into complete response
- Handles partial JSON objects
- Manages streaming errors
- Provides callback interface for real-time display

**Key functions:**
- `process_stream()` - Main streaming handler
- `parse_chunk()` - Parses individual JSON chunks
- `accumulate_response()` - Builds complete response

**When you'd modify this:**
- Changing streaming protocol
- Adding streaming filters
- Implementing backpressure
- Adding streaming metrics

---

### markdown_renderer.rs
**Purpose:** Terminal markdown rendering

**What it does:**
- Renders markdown in the terminal
- Syntax highlights code blocks
- Formats headers, lists, links
- Handles terminal width
- Provides colored output

**Key functions:**
- `render()` - Renders markdown to terminal
- `render_code_block()` - Syntax highlights code
- `render_inline()` - Handles inline formatting

**When you'd modify this:**
- Adding new markdown features
- Changing syntax highlighting
- Customizing colors/themes
- Supporting new terminal types

---

### terminal.rs
**Purpose:** Terminal state management

**What it does:**
- Manages terminal raw mode
- Handles terminal size
- Clears screen
- Manages cursor position
- Restores terminal on exit

**Key structures:**
- `TerminalState` - Manages terminal state
  - Enables/disables raw mode
  - Tracks terminal dimensions

**Key methods:**
- `new()` - Initializes terminal
- `cleanup()` - Restores terminal state
- `clear()` - Clears screen
- `size()` - Gets terminal dimensions

**When you'd modify this:**
- Adding terminal features
- Supporting new terminal types
- Changing terminal behavior
- Adding terminal effects

---

## Commands

### commands.rs
**Purpose:** Interactive command parsing

**What it does:**
- Parses slash commands (/help, /exit, etc.)
- Defines available commands
- Provides command descriptions
- Handles command aliases
- Case-insensitive parsing

**Key structures:**
- `Command` - Enum of all commands
  - `Exit` - Exit application
  - `Help` - Show help
  - `Models` - List models
  - `New` - New conversation
  - `Clear` - Clear screen
  - `Update` - Self-update

**Key methods:**
- `parse()` - Parses command string
- `description()` - Gets command description
- `display_help()` - Shows all commands

**When you'd modify this:**
- Adding new commands
- Changing command syntax
- Adding command aliases
- Modifying help text

---

## Data Management

### conversation.rs
**Purpose:** Conversation persistence

**What it does:**
- Saves conversations to JSON files
- Loads conversation history
- Manages conversation metadata
- Creates unique conversation IDs
- Tracks message timestamps
- Maintains conversation list

**Key structures:**
- `Conversation` - A chat conversation
  - `id` - Unique identifier
  - `messages` - List of messages
  - `created_at` - Creation timestamp
  - `updated_at` - Last update timestamp
- `ChatMessage` - A single message
  - `role` - "user" or "assistant"
  - `content` - Message text
  - `timestamp` - When sent
- `ConversationManager` - Handles file operations

**Key methods:**
- `save_conversation()` - Saves to disk
- `load_conversation()` - Loads from disk
- `list_conversations()` - Gets all conversations
- `delete_conversation()` - Removes conversation

**When you'd modify this:**
- Changing storage format
- Adding conversation features
- Implementing search
- Adding export functionality

---

## Utilities

### url_validator.rs
**Purpose:** URL validation and security

**What it does:**
- Validates backend URLs
- Enforces HTTPS for remote servers
- Allows HTTP for localhost
- Detects localhost URLs
- Suggests HTTPS alternatives
- Provides helpful error messages

**Key functions:**
- `validate_backend_url()` - Main validation function
- `is_localhost_url()` - Checks if URL is localhost
- `suggest_https_url()` - Converts HTTP to HTTPS
- `filter_valid_urls()` - Filters URL list

**Security rules:**
- Remote URLs must use HTTPS
- Localhost URLs can use HTTP
- Invalid URLs are rejected with clear errors

**When you'd modify this:**
- Adding new validation rules
- Supporting new URL schemes
- Changing security policy
- Adding URL transformations

---

### exit_codes.rs
**Purpose:** Exit code definitions

**What it does:**
- Defines exit codes for different errors
- Categorizes errors by type
- Provides script-friendly exit codes
- Maps errors to codes
- Handles exit with proper code

**Exit codes:**
- `0` - Success
- `1` - Invalid arguments
- `2` - Backend unreachable
- `3` - Authentication failed
- `4` - Model unavailable
- `5` - File error
- `130` - Interrupted (Ctrl+C)
- `143` - Terminated (SIGTERM)

**Key functions:**
- `categorize_error()` - Maps error to exit code
- `exit_with_error()` - Exits with code and message

**When you'd modify this:**
- Adding new error types
- Changing exit code mappings
- Adding error categorization logic

---

### error.rs
**Purpose:** Error types and handling

**What it does:**
- Defines custom error types
- Provides error context
- Implements error display
- Handles error conversion
- Provides error utilities

**Key types:**
- `CliError` - Main error type
- `ValidationError` - Input validation errors
- `BackendError` - Backend communication errors

**When you'd modify this:**
- Adding new error types
- Changing error messages
- Adding error recovery logic
- Implementing error reporting

---

### update.rs
**Purpose:** Self-update functionality

**What it does:**
- Checks for new versions
- Downloads updates from GitHub
- Validates installation
- Performs git pull updates
- Rebuilds application
- Handles update errors

**Key functions:**
- `check_for_updates()` - Checks GitHub for new version
- `perform_update()` - Executes update process
- `validate_installation()` - Checks if update is possible

**When you'd modify this:**
- Changing update mechanism
- Adding update notifications
- Implementing auto-updates
- Adding rollback functionality

---

## Module Dependencies

```
main.rs
├── mode.rs (detect execution mode)
├── config.rs (load configuration)
├── url_validator.rs (validate URLs)
├── app.rs (interactive mode)
│   ├── backend.rs (API communication)
│   ├── commands.rs (command parsing)
│   ├── conversation.rs (persistence)
│   ├── terminal.rs (terminal control)
│   ├── markdown_renderer.rs (display)
│   └── streaming.rs (response handling)
└── non_interactive.rs (non-interactive mode)
    ├── backend.rs (API communication)
    ├── input.rs (input processing)
    ├── output.rs (output formatting)
    ├── streaming.rs (response handling)
    └── exit_codes.rs (exit handling)
```

## Learning Path

If you're new to the codebase, read modules in this order:

1. **main.rs** - Understand the entry point
2. **mode.rs** - See how modes are detected
3. **config.rs** - Learn about configuration
4. **backend.rs** - Understand API communication
5. **app.rs** - See interactive mode in action
6. **non_interactive.rs** - See non-interactive mode
7. **conversation.rs** - Understand data persistence
8. **commands.rs** - Learn available commands
9. **input.rs** - See input processing
10. **output.rs** - Understand output formatting

Then explore the utility modules as needed.
