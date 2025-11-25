# Flow Diagrams

Visual representations of how data and control flow through Prometheus CLI.

## Application Startup Flow

```
┌─────────────────────────────────────────────────────────────┐
│                         START                                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Parse CLI Arguments │
              │     (clap/main.rs)   │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Load Configuration  │
              │     (config.rs)      │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Validate Backend   │
              │        URL           │
              │  (url_validator.rs)  │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Detect Execution   │
              │        Mode          │
              │      (mode.rs)       │
              └──────────┬───────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│   Interactive    │        │ Non-Interactive  │
│      Mode        │        │      Mode        │
│    (app.rs)      │        │(non_interactive) │
└──────────────────┘        └──────────────────┘
```

## Interactive Mode (REPL) Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Interactive Mode                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Display Prompt ">"  │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Wait for Input     │
              │   (terminal.rs)      │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Parse Input        │
              │   (commands.rs)      │
              └──────────┬───────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│   Is Command?    │        │   Is Prompt?     │
│   (/help, etc)   │        │   (chat text)    │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
         ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│ Execute Command  │        │  Send to Backend │
│    (app.rs)      │        │   (backend.rs)   │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
         │                           ▼
         │                  ┌──────────────────┐
         │                  │ Stream Response  │
         │                  │  (streaming.rs)  │
         │                  └────────┬─────────┘
         │                           │
         │                           ▼
         │                  ┌──────────────────┐
         │                  │ Render Markdown  │
         │                  │(markdown_render) │
         │                  └────────┬─────────┘
         │                           │
         │                           ▼
         │                  ┌──────────────────┐
         │                  │ Save to History  │
         │                  │(conversation.rs) │
         │                  └────────┬─────────┘
         │                           │
         └───────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Loop Back to       │
              │   Display Prompt     │
              └──────────────────────┘
```

## Non-Interactive Mode Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  Non-Interactive Mode                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Collect All Inputs  │
              │   - CLI prompt       │
              │   - stdin            │
              │   - --file contents  │
              │   - --system prompt  │
              │    (input.rs)        │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Validate Parameters │
              │   - prompt not empty │
              │   - temperature valid│
              │   - max_tokens valid │
              │    (input.rs)        │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Send Single Request │
              │    (backend.rs)      │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Process Response    │
              │   (streaming.rs)     │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Format Output       │
              │   Based on Flags:    │
              │   --quiet            │
              │   --json             │
              │   --verbose          │
              │   (output.rs)        │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Exit with Code      │
              │   (exit_codes.rs)    │
              └──────────────────────┘
```

## Backend Communication Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Send Prompt to Backend                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Create HTTP Client  │
              │   with timeout       │
              │   (backend.rs)       │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Build JSON Request  │
              │  {                   │
              │    model: "llama2",  │
              │    prompt: "...",    │
              │    stream: true      │
              │  }                   │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  POST to Ollama      │
              │  /api/generate       │
              └──────────┬───────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│   Success?       │        │    Error?        │
│   Status 200     │        │   Status 4xx/5xx │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
         ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│ Stream Response  │        │  Parse Error     │
│ Line by Line     │        │  Return Error    │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
         ▼                           │
┌──────────────────┐                 │
│ Parse Each Line  │                 │
│ as JSON:         │                 │
│ {"response":"x", │                 │
│  "done": false}  │                 │
└────────┬─────────┘                 │
         │                           │
         ▼                           │
┌──────────────────┐                 │
│ Accumulate Text  │                 │
│ Call Callback    │                 │
└────────┬─────────┘                 │
         │                           │
         ▼                           │
┌──────────────────┐                 │
│ Until "done":true│                 │
└────────┬─────────┘                 │
         │                           │
         ▼                           │
┌──────────────────┐                 │
│ Return Complete  │                 │
│    Response      │                 │
└────────┬─────────┘                 │
         │                           │
         └───────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Return to Caller   │
              └──────────────────────┘
```

## Configuration Loading Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Load Configuration                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Try Load config.toml│
              │    (config.rs)       │
              └──────────┬───────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│   File Exists?   │        │  File Missing?   │
│   Parse TOML     │        │  Use Defaults    │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
         ▼                           │
┌──────────────────┐                 │
│ Validate URLs    │                 │
│ Filter invalid   │                 │
│(url_validator.rs)│                 │
└────────┬─────────┘                 │
         │                           │
         └───────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Merge with CLI Args │
              │  (CLI args override) │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Return Final Config │
              └──────────────────────┘
```

## Conversation Persistence Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Save Conversation                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Update Timestamp    │
              │  conversation.       │
              │  updated_at = now()  │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Serialize to JSON   │
              │  (serde_json)        │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Write to File       │
              │  conversations/      │
              │  {uuid}.json         │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Update Metadata     │
              │  - Load metadata.json│
              │  - Add/update entry  │
              │  - Sort by date      │
              │  - Save metadata.json│
              └──────────────────────┘
```

## Error Handling Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Error Occurs                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Categorize Error    │
              │  (exit_codes.rs)     │
              └──────────┬───────────┘
                         │
           ┌─────────────┼─────────────┐
           │             │             │
           ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ Backend  │  │   File   │  │Validation│
    │  Error   │  │  Error   │  │  Error   │
    └────┬─────┘  └────┬─────┘  └────┬─────┘
         │             │             │
         ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │Exit: 2-4 │  │ Exit: 5  │  │ Exit: 1  │
    └────┬─────┘  └────┬─────┘  └────┬─────┘
         │             │             │
         └─────────────┼─────────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Format Error Message│
            │  (error.rs)          │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Log to stderr       │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Exit with Code      │
            └──────────────────────┘
```

## URL Validation Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Validate URL                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │   Parse URL          │
              │   (url crate)        │
              └──────────┬───────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐
│  Valid Format?   │        │ Invalid Format?  │
│  http(s)://...   │        │  Return Error    │
└────────┬─────────┘        └──────────────────┘
         │
         ▼
┌──────────────────┐
│  Is Localhost?   │
│  - localhost     │
│  - 127.0.0.1     │
│  - ::1           │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌────────┐
│  Yes   │ │   No   │
│ (Local)│ │(Remote)│
└───┬────┘ └───┬────┘
    │          │
    ▼          ▼
┌────────┐ ┌────────┐
│HTTP or │ │ HTTPS  │
│HTTPS OK│ │ Only!  │
└───┬────┘ └───┬────┘
    │          │
    │     ┌────┴────┐
    │     │         │
    │     ▼         ▼
    │  ┌────────┐ ┌────────┐
    │  │ HTTPS? │ │ HTTP?  │
    │  │  OK!   │ │ Error! │
    │  └───┬────┘ └────────┘
    │      │
    └──────┴──────────┐
                      │
                      ▼
           ┌──────────────────────┐
           │   Return Valid URL   │
           └──────────────────────┘
```

## Command Processing Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Process Command                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Parse Command       │
              │  (commands.rs)       │
              └──────────┬───────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
    ┌────────┐      ┌────────┐     ┌────────┐
    │ /help  │      │ /exit  │     │/models │
    └───┬────┘      └───┬────┘     └───┬────┘
        │               │               │
        ▼               ▼               ▼
    ┌────────┐      ┌────────┐     ┌────────┐
    │ Show   │      │ Save & │     │ Call   │
    │Commands│      │ Quit   │     │Backend │
    └────────┘      └────────┘     └────────┘
         │               │               │
         │               │               │
         ▼               ▼               ▼
    ┌────────┐      ┌────────┐     ┌────────┐
    │ /new   │      │ /clear │     │/update │
    └───┬────┘      └───┬────┘     └───┬────┘
        │               │               │
        ▼               ▼               ▼
    ┌────────┐      ┌────────┐     ┌────────┐
    │ Start  │      │ Clear  │     │  Git   │
    │  New   │      │ Screen │     │ Pull & │
    │ Chat   │      │        │     │ Build  │
    └────────┘      └────────┘     └────────┘
```

## Module Dependency Graph

```
                    main.rs
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
    mode.rs      config.rs    url_validator.rs
        │              │              │
        │              └──────┬───────┘
        │                     │
        ▼                     ▼
    ┌────────────────────────────────┐
    │                                │
    ▼                                ▼
app.rs                    non_interactive.rs
    │                                │
    ├──────────┬──────────┐          │
    │          │          │          │
    ▼          ▼          ▼          ▼
backend.rs  commands.rs  conversation.rs
    │          │          │          │
    ▼          │          │          ▼
streaming.rs  │          │      input.rs
    │          │          │          │
    ▼          │          │          ▼
markdown_    │          │      output.rs
renderer.rs   │          │          │
    │          │          │          │
    ▼          ▼          ▼          ▼
terminal.rs ←─────────────────────────
                       │
                       ▼
                 exit_codes.rs
                       │
                       ▼
                   error.rs
```

## Data Structure Relationships

```
AppConfig
    ├── AppSettings
    │   ├── window_title: String
    │   ├── window_width: f32
    │   └── window_height: f32
    ├── BackendSettings
    │   ├── url: String
    │   ├── timeout_seconds: u64
    │   └── saved_urls: Vec<String>
    └── UISettings
        ├── font_size: u32
        ├── max_chat_history: usize
        └── theme: String

Conversation
    ├── id: String (UUID)
    ├── name: String
    ├── created_at: String (ISO 8601)
    ├── updated_at: String (ISO 8601)
    ├── model: Option<String>
    └── messages: Vec<ChatMessage>
                      │
                      └── ChatMessage
                          ├── role: String ("user"/"assistant")
                          ├── content: String
                          └── timestamp: String (ISO 8601)

BackendClient
    ├── client: reqwest::Client
    ├── base_url: String
    └── timeout: Duration

CliApp
    ├── config: AppConfig
    ├── backend: BackendClient
    ├── conversation: Conversation
    └── conversation_manager: ConversationManager
```

These diagrams should help you visualize how the application works!
