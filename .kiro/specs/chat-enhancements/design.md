# Design Document

## Overview

This design covers three interconnected features that will significantly enhance the chat application's usability and functionality. The features are designed to work together cohesively while maintaining the existing hacker aesthetic and performance characteristics.

## Architecture

### High-Level Component Structure

```
ChatApp
├── ConversationManager (new)
│   ├── conversations/
│   │   ├── {uuid}.json
│   │   └── metadata.json
│   └── active_conversation_id
├── MessageRenderer (enhanced)
│   ├── MarkdownParser
│   ├── CodeBlockRenderer
│   └── SyntaxHighlighter
└── ContextMenu (new)
    ├── MessageContextMenu
    └── ConversationContextMenu
```

## Components and Interfaces

### 1. Message Context Menu

#### Data Structures

```rust
#[derive(Debug, Clone)]
pub enum ContextMenuAction {
    Copy,
    Delete,
    Edit,
}

pub struct ContextMenuState {
    visible: bool,
    message_index: Option<usize>,
    position: (f32, f32),
}
```

#### Message Enum Updates

```rust
pub enum Message {
    // ... existing messages
    ShowContextMenu(usize, (f32, f32)),  // message index, position
    HideContextMenu,
    ContextMenuAction(ContextMenuAction, usize),
    StartEditMessage(usize),
    UpdateEditMessage(String),
    ConfirmEditMessage,
    CancelEditMessage,
}
```

#### Implementation Details

- Use Iced's `mouse_area` widget to detect right-clicks on messages
- Context menu will be a floating overlay positioned at cursor location
- Edit mode will replace message bubble with a text input
- Editing a user message will clear all subsequent messages (AI responses become invalid)

### 2. Conversation Management

#### Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,  // UUID
    pub name: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: String,
    pub updated_at: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: String,
    pub name: String,
    pub preview: String,  // First 50 chars of first message
    pub updated_at: String,
    pub message_count: usize,
}

pub struct ConversationManager {
    conversations: Vec<ConversationMetadata>,
    active_conversation: Option<Conversation>,
    conversations_dir: PathBuf,
}
```

#### Message Enum Updates

```rust
pub enum Message {
    // ... existing messages
    NewConversation,
    LoadConversation(String),  // conversation ID
    ConversationsLoaded(Result<Vec<ConversationMetadata>, String>),
    SaveConversation,
    ConversationSaved(Result<(), String>),
    ShowConversationContextMenu(String, (f32, f32)),
    RenameConversation(String),  // conversation ID
    DeleteConversation(String),  // conversation ID
    UpdateConversationName(String),
    ConfirmRenameConversation,
}
```

#### File Structure

```
conversations/
├── metadata.json           # List of all conversations with metadata
├── {uuid-1}.json          # Full conversation data
├── {uuid-2}.json
└── ...
```

#### Implementation Details

- Each conversation stored as separate JSON file with UUID filename
- Metadata file for quick loading of conversation list
- Auto-save after each message exchange
- Sidebar shows scrollable list of conversations
- Active conversation highlighted in sidebar
- Lazy loading: only load full conversation when selected

### 3. Code Block Formatting

#### Data Structures

```rust
#[derive(Debug, Clone)]
pub enum MessageSegment {
    Text(String),
    CodeBlock { language: Option<String>, code: String },
    InlineCode(String),
    Bold(String),
    Italic(String),
    ListItem(String),
}

pub struct ParsedMessage {
    segments: Vec<MessageSegment>,
}
```

#### Dependencies

We'll use existing Rust crates:
- `syntect` for syntax highlighting (supports 100+ languages)
- Custom markdown parser (simple regex-based for our needs)

#### Message Enum Updates

```rust
pub enum Message {
    // ... existing messages
    CopyCodeBlock(String),  // code content
    CodeBlockCopied,
}
```

#### Implementation Details

**Markdown Parsing:**
- Parse message content on display, not on storage
- Detect code blocks: ` ```language\ncode\n``` `
- Detect inline code: `` `code` ``
- Detect bold: `**text**`
- Detect italic: `*text*`
- Detect lists: lines starting with `- ` or `* `

**Code Block Rendering:**
- Each code block rendered as separate container
- Dark background (#1a1a1a) with border
- Syntax highlighting using syntect with a dark theme
- Copy button positioned absolute top-right
- Button shows "Copy" normally, "Copied!" for 2 seconds after click

**Syntax Highlighting:**
- Use syntect's built-in themes (base16-ocean.dark or similar)
- Support common languages: rust, python, javascript, typescript, java, go, etc.
- Fallback to plain monospace if language not recognized

## Data Models

### Conversation File Format

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Rust async discussion",
  "messages": [
    {
      "role": "user",
      "content": "How do I use async/await in Rust?",
      "timestamp": "2025-11-17T14:30:00"
    },
    {
      "role": "assistant",
      "content": "Here's how to use async/await...",
      "timestamp": "2025-11-17T14:30:05"
    }
  ],
  "created_at": "2025-11-17T14:30:00",
  "updated_at": "2025-11-17T14:30:05",
  "model": "llama2"
}
```

### Metadata File Format

```json
{
  "conversations": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Rust async discussion",
      "preview": "How do I use async/await in Rust?",
      "updated_at": "2025-11-17T14:30:05",
      "message_count": 2
    }
  ]
}
```

## Error Handling

### Conversation Management Errors
- File I/O errors: Log and show user-friendly error message
- JSON parse errors: Attempt recovery, fallback to empty conversation
- Disk full: Warn user, prevent new conversations

### Context Menu Errors
- Clipboard access denied: Show error toast
- Edit conflicts: Prevent editing if streaming in progress

### Code Formatting Errors
- Invalid markdown: Render as plain text
- Syntax highlighting failure: Fallback to monospace
- Copy failure: Show error message

## Testing Strategy

### Unit Tests
- Markdown parser with various code block formats
- Conversation serialization/deserialization
- Context menu state management
- Message editing logic

### Integration Tests
- Create, save, load, delete conversations
- Edit message and verify subsequent messages cleared
- Copy code blocks and verify clipboard content
- Render various markdown formats

### Manual Testing
- Test with long conversations (100+ messages)
- Test with large code blocks (1000+ lines)
- Test rapid conversation switching
- Test context menu on different message types
- Test syntax highlighting for 10+ languages

## Performance Considerations

### Conversation Loading
- Lazy load: Only load metadata on startup
- Load full conversation only when selected
- Cache active conversation in memory
- Debounce auto-save (save 1 second after last message)

### Markdown Parsing
- Parse on render, not on storage
- Cache parsed segments per message
- Invalidate cache only when message changes

### Syntax Highlighting
- Use syntect's cached syntax sets
- Limit highlighting to visible code blocks
- Consider async highlighting for very large blocks

## UI/UX Design

### Context Menu
- Semi-transparent dark background (#1a1a1a with 95% opacity)
- Cyan border (#00ffaa)
- Hover state: lighter background
- Positioned at cursor, adjusted if near screen edge

### Sidebar Conversations List
- Each conversation: 2-line item
  - Line 1: Name (bold, truncated)
  - Line 2: Preview (muted, truncated)
- Active conversation: cyan left border
- Hover: subtle background highlight
- Right-click: show rename/delete menu

### Code Blocks
- Dark background (#1a1a1a)
- Cyan border (1px)
- Language label in top-left (muted)
- Copy button in top-right
- Padding: 12px
- Monospace font: "Fira Code" or "JetBrains Mono" if available

### Edit Mode
- Replace message bubble with text input
- Show "Save" and "Cancel" buttons below
- Warning text: "Editing will clear subsequent messages"
- Input has same styling as message bubble
