# Design Document

## Overview

This design document outlines the architecture and implementation approach for adding modern AI chat features to Prometheus. The design prioritizes user experience, performance, and maintainability while building on the existing Iced-based architecture.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Chat Application                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Search     │  │   Export     │  │  Templates   │     │
│  │   Engine     │  │   Manager    │  │   Manager    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Attachment   │  │   Stats      │  │   Voice      │     │
│  │   Handler    │  │   Tracker    │  │   Input      │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Folder     │  │   Draft      │  │   Import/    │     │
│  │   Manager    │  │   Manager    │  │   Export     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Persistence Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │Conversations │  │   Templates  │  │    Drafts    │     │
│  │     JSON     │  │     JSON     │  │     JSON     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Folders    │  │  Statistics  │  │ Attachments  │     │
│  │     JSON     │  │     JSON     │  │    Files     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Module Structure

```
src/
├── app.rs                    # Main application (existing)
├── config.rs                 # Configuration (existing)
├── conversation.rs           # Conversation management (existing)
├── markdown.rs              # Markdown parsing (existing)
├── search/
│   ├── mod.rs               # Search engine module
│   ├── index.rs             # Search indexing
│   └── query.rs             # Query processing
├── export/
│   ├── mod.rs               # Export manager
│   ├── markdown.rs          # Markdown export
│   ├── json.rs              # JSON export
│   └── plaintext.rs         # Plain text export
├── templates/
│   ├── mod.rs               # Template manager
│   └── defaults.rs          # Default templates
├── attachments/
│   ├── mod.rs               # Attachment handler
│   ├── file_reader.rs       # File reading
│   └── image_handler.rs     # Image processing
├── stats/
│   ├── mod.rs               # Statistics tracker
│   └── aggregator.rs        # Data aggregation
├── voice/
│   ├── mod.rs               # Voice input handler
│   └── transcription.rs     # Audio transcription
├── folders/
│   ├── mod.rs               # Folder manager
│   └── tags.rs              # Tag management
├── drafts/
│   └── mod.rs               # Draft manager
└── keyboard/
    └── mod.rs               # Keyboard shortcuts
```

## Components and Interfaces

### 1. Search Engine

**Purpose:** Enable fast full-text search across all conversations

**Data Structures:**
```rust
pub struct SearchEngine {
    index: HashMap<String, Vec<SearchResult>>,
    conversations: Vec<String>, // conversation IDs
}

pub struct SearchResult {
    conversation_id: String,
    message_index: usize,
    match_positions: Vec<(usize, usize)>, // start, end
    context: String, // surrounding text
}

pub struct SearchQuery {
    text: String,
    case_sensitive: bool,
    whole_word: bool,
}
```

**Key Methods:**
- `index_conversation(conversation: &Conversation)` - Add conversation to search index
- `search(query: &SearchQuery) -> Vec<SearchResult>` - Execute search
- `highlight_matches(text: &str, positions: &[(usize, usize)]) -> Vec<TextSegment>` - Highlight search results
- `rebuild_index()` - Rebuild entire search index

**Implementation Notes:**
- Use simple string matching for MVP (can upgrade to fuzzy search later)
- Index conversations on load and after modifications
- Store index in memory for fast access
- Debounce search queries (300ms) to avoid excessive processing

### 2. Export Manager

**Purpose:** Export conversations to various formats

**Data Structures:**
```rust
pub enum ExportFormat {
    Markdown,
    PlainText,
    Json,
}

pub struct ExportOptions {
    format: ExportFormat,
    include_timestamps: bool,
    include_metadata: bool,
    include_system_prompts: bool,
}

pub struct ExportManager {
    config: ExportOptions,
}
```

**Key Methods:**
- `export_conversation(conversation: &Conversation, format: ExportFormat) -> Result<String>` - Generate export content
- `save_to_file(content: &str, path: &Path) -> Result<()>` - Save to disk
- `format_markdown(conversation: &Conversation) -> String` - Format as Markdown
- `format_json(conversation: &Conversation) -> String` - Format as JSON
- `format_plaintext(conversation: &Conversation) -> String` - Format as plain text

**Export Formats:**

*Markdown:*
```markdown
# Conversation: [Name]
Date: [Created At]
Model: [Model Name]

## User
[Timestamp]
[Message content]

## Assistant
[Timestamp]
[Response content]
```

*JSON:*
```json
{
  "id": "uuid",
  "name": "Conversation Name",
  "created_at": "ISO8601",
  "model": "model-name",
  "messages": [
    {
      "role": "user",
      "content": "...",
      "timestamp": "..."
    }
  ]
}
```

### 3. Template Manager

**Purpose:** Manage prompt templates for common tasks

**Data Structures:**
```rust
pub struct Template {
    id: String,
    name: String,
    content: String,
    category: String,
    is_custom: bool,
    created_at: String,
}

pub struct TemplateManager {
    templates: Vec<Template>,
    templates_file: PathBuf,
}
```

**Default Templates:**
- Code Review: "Please review the following code and provide feedback on..."
- Explain: "Please explain the following concept in simple terms..."
- Debug: "I'm encountering an error. Here's the code and error message..."
- Summarize: "Please provide a concise summary of..."
- Translate: "Please translate the following text to [language]..."
- Refactor: "Please suggest improvements for this code..."

**Key Methods:**
- `load_templates() -> Result<Vec<Template>>` - Load from disk
- `save_templates() -> Result<()>` - Save to disk
- `add_template(template: Template) -> Result<()>` - Add custom template
- `delete_template(id: &str) -> Result<()>` - Remove template
- `get_by_category(category: &str) -> Vec<&Template>` - Filter by category

### 4. Attachment Handler

**Purpose:** Handle file attachments in messages

**Data Structures:**
```rust
pub struct Attachment {
    id: String,
    filename: String,
    file_type: FileType,
    size: u64,
    content: Vec<u8>,
    preview: Option<String>, // For text files
}

pub enum FileType {
    Text,
    Image,
    Code,
    Unknown,
}

pub struct AttachmentHandler {
    max_size: u64, // 10MB default
    supported_types: Vec<String>,
}
```

**Key Methods:**
- `read_file(path: &Path) -> Result<Attachment>` - Read file from disk
- `validate_file(attachment: &Attachment) -> Result<()>` - Check size and type
- `generate_preview(attachment: &Attachment) -> Option<String>` - Create text preview
- `encode_for_api(attachment: &Attachment) -> String` - Encode for backend

**Supported File Types:**
- Text: .txt, .md, .log
- Code: .rs, .py, .js, .ts, .java, .cpp, .c, .go
- Images: .png, .jpg, .jpeg, .gif, .webp
- Max size: 10MB per file

### 5. Statistics Tracker

**Purpose:** Track and display usage statistics

**Data Structures:**
```rust
pub struct ConversationStats {
    total_conversations: usize,
    total_messages: usize,
    total_tokens: u64,
    average_messages_per_conversation: f64,
    model_usage: HashMap<String, usize>,
    conversations_by_date: HashMap<String, usize>,
}

pub struct StatsTracker {
    stats: ConversationStats,
    stats_file: PathBuf,
}
```

**Key Methods:**
- `calculate_stats(conversations: &[Conversation]) -> ConversationStats` - Compute statistics
- `update_on_message(tokens: u64)` - Update after each message
- `get_date_range_stats(start: &str, end: &str) -> ConversationStats` - Filter by date
- `save_stats() -> Result<()>` - Persist to disk

### 6. Folder Manager

**Purpose:** Organize conversations into folders and tags

**Data Structures:**
```rust
pub struct Folder {
    id: String,
    name: String,
    parent_id: Option<String>, // For nested folders
    conversation_ids: Vec<String>,
    created_at: String,
}

pub struct Tag {
    name: String,
    color: String, // Hex color
}

pub struct FolderManager {
    folders: Vec<Folder>,
    tags: HashMap<String, Vec<Tag>>, // conversation_id -> tags
    folders_file: PathBuf,
}
```

**Key Methods:**
- `create_folder(name: String, parent_id: Option<String>) -> Result<Folder>` - Create new folder
- `move_to_folder(conversation_id: &str, folder_id: &str) -> Result<()>` - Move conversation
- `add_tag(conversation_id: &str, tag: Tag) -> Result<()>` - Add tag
- `get_by_folder(folder_id: &str) -> Vec<String>` - Get conversations in folder
- `get_by_tag(tag: &str) -> Vec<String>` - Get conversations with tag

### 7. Draft Manager

**Purpose:** Auto-save and restore draft messages

**Data Structures:**
```rust
pub struct Draft {
    conversation_id: String,
    content: String,
    saved_at: String,
}

pub struct DraftManager {
    drafts: HashMap<String, Draft>, // conversation_id -> draft
    drafts_file: PathBuf,
    save_delay: Duration, // 2 seconds
}
```

**Key Methods:**
- `save_draft(conversation_id: &str, content: &str)` - Save draft (debounced)
- `load_draft(conversation_id: &str) -> Option<String>` - Load draft
- `clear_draft(conversation_id: &str)` - Remove draft
- `persist_drafts() -> Result<()>` - Save to disk

### 8. Voice Input Handler

**Purpose:** Enable voice-to-text input

**Data Structures:**
```rust
pub struct VoiceInput {
    is_recording: bool,
    audio_buffer: Vec<u8>,
    transcription_service: TranscriptionService,
}

pub enum TranscriptionService {
    Local,  // Using local model
    Remote, // Using API
}
```

**Key Methods:**
- `start_recording() -> Result<()>` - Begin audio capture
- `stop_recording() -> Result<Vec<u8>>` - End capture and return audio
- `transcribe(audio: &[u8]) -> Result<String>` - Convert audio to text
- `request_permissions() -> Result<()>` - Request microphone access

**Implementation Notes:**
- Use `cpal` crate for audio capture
- Use `whisper-rs` for local transcription (optional)
- Support external API for transcription (OpenAI Whisper API)
- Display waveform visualization during recording

### 9. Keyboard Shortcuts Manager

**Purpose:** Handle keyboard shortcuts for common actions

**Data Structures:**
```rust
pub struct KeyboardShortcut {
    key: Key,
    modifiers: Modifiers,
    action: ShortcutAction,
}

pub enum ShortcutAction {
    NewConversation,
    Search,
    Export,
    Save,
    CloseModal,
    ShowHelp,
}

pub struct KeyboardManager {
    shortcuts: Vec<KeyboardShortcut>,
    enabled: bool,
}
```

**Default Shortcuts:**
- Cmd/Ctrl+N: New conversation
- Cmd/Ctrl+K: Search
- Cmd/Ctrl+E: Export
- Cmd/Ctrl+S: Save
- Cmd/Ctrl+/: Show shortcuts help
- Escape: Close modals

**Key Methods:**
- `handle_key_press(key: Key, modifiers: Modifiers) -> Option<ShortcutAction>` - Process key event
- `is_shortcut_enabled(action: ShortcutAction) -> bool` - Check if enabled
- `get_shortcut_display() -> Vec<(String, String)>` - Get list for help panel

## Data Models

### Extended Conversation Model

```rust
pub struct Conversation {
    // Existing fields
    pub id: String,
    pub name: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: String,
    pub updated_at: String,
    pub model: Option<String>,
    
    // New fields
    pub folder_id: Option<String>,
    pub tags: Vec<Tag>,
    pub is_pinned: bool,
    pub system_prompt: Option<String>,
    pub token_count: u64,
    pub parent_conversation_id: Option<String>, // For branches
    pub branch_point: Option<usize>, // Message index where branch occurred
}
```

### Extended Message Model

```rust
pub struct ChatMessage {
    // Existing fields
    pub role: String,
    pub content: String,
    pub timestamp: String,
    
    // New fields
    pub attachments: Vec<Attachment>,
    pub token_count: Option<u64>,
    pub regeneration_count: u32,
    pub alternatives: Vec<String>, // Alternative responses for comparison
}
```

### Configuration Extensions

```rust
pub struct UISettings {
    // Existing fields
    pub font_size: u16,
    pub max_chat_history: usize,
    pub theme: String,
    
    // New fields
    pub streaming_speed: f32, // 0.5 to 2.0
    pub auto_save_drafts: bool,
    pub show_token_counts: bool,
    pub compact_mode: bool,
    pub enable_voice_input: bool,
    pub search_case_sensitive: bool,
}
```

## Error Handling

### Error Types

```rust
pub enum AppError {
    // File operations
    FileNotFound(String),
    FileReadError(String),
    FileWriteError(String),
    
    // Search
    SearchIndexError(String),
    InvalidSearchQuery(String),
    
    // Export
    ExportFormatError(String),
    ExportSaveError(String),
    
    // Attachments
    FileTooLarge(u64),
    UnsupportedFileType(String),
    AttachmentReadError(String),
    
    // Voice
    MicrophonePermissionDenied,
    AudioCaptureError(String),
    TranscriptionError(String),
    
    // Templates
    TemplateNotFound(String),
    InvalidTemplate(String),
}
```

### Error Handling Strategy

1. **User-Facing Errors:** Display clear, actionable error messages in the UI
2. **Logging:** Log all errors to `logs/error.log` with context
3. **Recovery:** Attempt graceful degradation (e.g., disable voice if mic unavailable)
4. **Validation:** Validate user input before processing
5. **Fallbacks:** Provide fallback behavior when features fail

## Testing Strategy

### Unit Tests

- Test each module independently
- Mock file I/O operations
- Test error conditions
- Validate data transformations

**Priority Areas:**
- Search query parsing and matching
- Export format generation
- Template validation
- Attachment file type detection
- Statistics calculations

### Integration Tests

- Test feature interactions
- Test persistence layer
- Test UI state management
- Test keyboard shortcuts

**Test Scenarios:**
- Create conversation → Add tags → Search → Export
- Attach file → Send message → Save conversation
- Create template → Use template → Send message
- Record voice → Transcribe → Send message

### Manual Testing

- Test UI responsiveness
- Test keyboard navigation
- Test with large datasets (100+ conversations)
- Test with large files (near 10MB limit)
- Test cross-platform compatibility

## Performance Considerations

### Optimization Strategies

1. **Lazy Loading:** Load conversations on-demand, not all at startup
2. **Indexing:** Build search index incrementally
3. **Caching:** Cache parsed markdown and search results
4. **Debouncing:** Debounce search queries and draft saves
5. **Async Operations:** Use async for file I/O and API calls
6. **Memory Management:** Limit in-memory conversation count

### Performance Targets

- Search response: < 100ms for 1000 conversations
- Export generation: < 500ms for 100-message conversation
- Draft auto-save: < 50ms
- Voice transcription: < 2s for 30s audio
- UI responsiveness: 60fps during streaming

## Security Considerations

### Data Protection

1. **Local Storage:** All data stored locally, no cloud sync by default
2. **File Permissions:** Restrict file access to user's directory
3. **Input Validation:** Sanitize all user input
4. **Attachment Scanning:** Validate file types and sizes
5. **API Keys:** Store API keys securely (system keychain)

### Privacy

1. **No Telemetry:** No usage data sent to external servers
2. **Optional Features:** Voice input requires explicit permission
3. **Export Control:** User controls what data is exported
4. **Conversation Sharing:** Explicit user action required

## UI/UX Design

### Search Interface

- Floating search bar at top of chat area
- Real-time results as user types
- Highlight matches in yellow
- Show result count and navigation arrows
- Keyboard navigation (Up/Down arrows)

### Export Dialog

- Modal dialog with format selection
- Checkboxes for options (timestamps, metadata)
- File name input with default suggestion
- Preview button to see export before saving
- Progress indicator for large exports

### Template Picker

- Dropdown menu from input area
- Categories: Code, Writing, Analysis, Custom
- Search/filter templates
- Preview template content on hover
- Quick edit button for custom templates

### Attachment UI

- Paperclip icon in input area
- Drag-and-drop support
- File preview chips below input
- Remove button (X) on each attachment
- File size and type indicators

### Voice Input

- Microphone icon in input area
- Pulsing animation during recording
- Waveform visualization
- Timer showing recording duration
- Cancel and Done buttons

### Folder Sidebar

- Collapsible folder tree
- Drag-and-drop to move conversations
- Right-click context menu for folders
- Tag chips on conversation items
- Color-coded tags

### Statistics Panel

- Modal dialog with tabs: Overview, Models, Timeline
- Charts using simple ASCII art or basic shapes
- Date range selector
- Export statistics button

## Migration Strategy

### Phase 1: Core Features (MVP)
- Search functionality
- Export (Markdown, JSON)
- Prompt templates
- Keyboard shortcuts
- Conversation pinning

### Phase 2: Enhanced Features
- File attachments
- Token tracking
- Regenerate responses
- Folders and tags
- Draft auto-save

### Phase 3: Advanced Features
- Message branching
- Voice input
- Response comparison
- Statistics dashboard
- System prompts

### Phase 4: Collaboration (Future)
- Conversation import
- Sharing functionality
- Multi-user support

## Dependencies

### New Crates Required

```toml
[dependencies]
# Search
regex = "1.10"

# Export
csv = "1.3"  # For potential CSV export

# Attachments
mime_guess = "2.0"
base64 = "0.21"
image = "0.24"  # For image processing

# Voice
cpal = "0.15"  # Audio capture
hound = "3.5"  # WAV file handling

# Statistics
chrono = "0.4"  # Already included

# UI
rfd = "0.12"  # File dialogs
```

### Optional Dependencies

```toml
[dependencies]
# Local transcription (optional)
whisper-rs = { version = "0.10", optional = true }

# Advanced search (optional)
tantivy = { version = "0.21", optional = true }
```

## Configuration

### Extended config.toml

```toml
[app]
window_title = "Prometheus"
window_width = 800
window_height = 600

[backend]
url = "http://localhost:8000/generate"
ollama_url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
streaming_speed = 1.0
auto_save_drafts = true
show_token_counts = true
compact_mode = false
enable_voice_input = false
search_case_sensitive = false

[features]
enable_search = true
enable_export = true
enable_templates = true
enable_attachments = true
enable_voice = false
enable_statistics = true
enable_folders = true

[attachments]
max_file_size_mb = 10
supported_types = ["txt", "md", "rs", "py", "js", "ts", "png", "jpg"]

[voice]
transcription_service = "local"  # or "openai"
max_recording_seconds = 60

[export]
default_format = "markdown"
include_timestamps = true
include_metadata = true
```

## Future Enhancements

### Potential Additions

1. **Plugin System:** Allow third-party extensions
2. **Custom Themes:** User-created color schemes
3. **Conversation Analytics:** Advanced insights and trends
4. **Multi-Model Comparison:** Send same prompt to multiple models
5. **Conversation Merge:** Combine multiple conversations
6. **Advanced Formatting:** Tables, diagrams, LaTeX support
7. **Offline Mode:** Queue messages when backend unavailable
8. **Conversation Templates:** Pre-structured conversation flows
9. **Smart Suggestions:** AI-powered prompt suggestions
10. **Integration APIs:** Connect with external tools

---

**Design Version:** 1.0  
**Last Updated:** 2025-11-17  
**Status:** Ready for Implementation
