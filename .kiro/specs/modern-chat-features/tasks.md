# Implementation Plan

## Phase 1: Core Features (MVP)

### 1. Search Functionality

- [x] 1. Set up search module structure
  - Create `src/search/mod.rs` with SearchEngine struct
  - Create `src/search/index.rs` for indexing logic
  - Create `src/search/query.rs` for query processing
  - _Requirements: 1.1_

- [x] 2. Implement search indexing
- [x] 2.1 Create SearchEngine data structures
  - Define SearchEngine, SearchResult, and SearchQuery structs
  - Implement serialization for persistence
  - _Requirements: 1.1_

- [x] 2.2 Implement index_conversation method
  - Parse conversation messages into searchable tokens
  - Store message positions and context
  - Handle special characters and punctuation
  - _Requirements: 1.1_

- [x] 2.3 Build search index on startup
  - Load all conversations and index them
  - Display progress indicator for large datasets
  - _Requirements: 1.1_

- [x] 3. Implement search query processing
- [x] 3.1 Create search method
  - Implement case-sensitive and case-insensitive search
  - Support whole-word matching
  - Return results with match positions
  - _Requirements: 1.2_

- [x] 3.2 Implement result highlighting
  - Create highlight_matches function
  - Generate TextSegments with highlighted regions
  - _Requirements: 1.3_

- [x] 4. Create search UI
- [x] 4.1 Add search bar to header
  - Create floating search input field
  - Add search icon button
  - Style with hacker theme
  - _Requirements: 1.1_

- [x] 4.2 Implement search results display
  - Show result count in search bar
  - Display matching messages with highlights
  - Add navigation arrows for results
  - _Requirements: 1.3, 1.5_

- [x] 4.3 Add search state management
  - Add search_active, search_query, search_results to ChatApp
  - Handle search input changes
  - Debounce search queries (300ms)
  - _Requirements: 1.2_

- [x] 4.4 Implement search clear functionality
  - Add clear button to search bar
  - Restore full conversation view
  - _Requirements: 1.4_

- [x] 5. Write search tests
  - Test indexing with various message types
  - Test case-sensitive and case-insensitive search
  - Test result highlighting
  - Test with large conversation sets
  - _Requirements: 1.1-1.5_

### 2. Export Functionality

- [ ] 6. Set up export module
  - Create `src/export/mod.rs` with ExportManager
  - Create `src/export/markdown.rs` for Markdown export
  - Create `src/export/json.rs` for JSON export
  - Create `src/export/plaintext.rs` for plain text export
  - _Requirements: 2.1_

- [ ] 7. Implement export data structures
  - Define ExportFormat enum
  - Define ExportOptions struct
  - Create ExportManager struct
  - _Requirements: 2.2_

- [ ] 8. Implement Markdown export
- [ ] 8.1 Create format_markdown function
  - Generate Markdown headers with conversation metadata
  - Format messages with role headers
  - Include timestamps if enabled
  - Preserve code blocks and formatting
  - _Requirements: 2.3, 2.4_

- [ ] 8.2 Handle special characters in Markdown
  - Escape Markdown special characters
  - Preserve existing code blocks
  - _Requirements: 2.3_

- [ ] 9. Implement JSON export
- [ ] 9.1 Create format_json function
  - Serialize conversation to JSON
  - Include all metadata fields
  - Format with pretty printing
  - _Requirements: 2.3, 2.4_

- [ ] 10. Implement plain text export
- [ ] 10.1 Create format_plaintext function
  - Format as simple text with role labels
  - Include timestamps if enabled
  - Remove formatting markup
  - _Requirements: 2.3, 2.4_

- [ ] 11. Implement file saving
- [ ] 11.1 Create save_to_file method
  - Use rfd crate for file dialog
  - Save content to selected path
  - Handle file write errors
  - _Requirements: 2.5_

- [ ] 11.2 Add export confirmation
  - Show success message after export
  - Display file path
  - _Requirements: 2.5_

- [ ] 12. Create export UI
- [ ] 12.1 Add export button to conversation menu
  - Add export option to context menu
  - Style with hacker theme
  - _Requirements: 2.1_

- [ ] 12.2 Create export dialog
  - Build modal with format selection
  - Add checkboxes for options (timestamps, metadata)
  - Add file name input with default
  - Add preview button
  - _Requirements: 2.2_

- [ ] 12.3 Implement export state management
  - Add export_dialog_open, export_format, export_options to ChatApp
  - Handle format selection
  - Handle option toggles
  - _Requirements: 2.2_

- [ ]* 13. Write export tests
  - Test Markdown generation
  - Test JSON serialization
  - Test plain text formatting
  - Test with various conversation sizes
  - _Requirements: 2.1-2.5_

### 3. Prompt Templates

- [ ] 14. Set up templates module
  - Create `src/templates/mod.rs` with TemplateManager
  - Create `src/templates/defaults.rs` for default templates
  - Create templates directory structure
  - _Requirements: 4.1_

- [ ] 15. Implement template data structures
  - Define Template struct with serialization
  - Define TemplateManager struct
  - Create default templates list
  - _Requirements: 4.2_

- [ ] 16. Implement template management
- [ ] 16.1 Create load_templates method
  - Load templates from JSON file
  - Include default templates
  - Handle missing file gracefully
  - _Requirements: 4.2_

- [ ] 16.2 Create save_templates method
  - Serialize templates to JSON
  - Save to templates file
  - Handle write errors
  - _Requirements: 4.5_

- [ ] 16.3 Implement add_template method
  - Validate template content
  - Add to templates list
  - Save to disk
  - _Requirements: 4.4_

- [ ] 16.4 Implement delete_template method
  - Remove from templates list
  - Save updated list
  - _Requirements: 4.4_

- [ ] 17. Create templates UI
- [ ] 17.1 Add templates button to input area
  - Create dropdown menu button
  - Style with hacker theme
  - Position near input field
  - _Requirements: 4.1_

- [ ] 17.2 Build templates dropdown menu
  - Display templates grouped by category
  - Show template names
  - Add search/filter input
  - _Requirements: 4.2, 4.3_

- [ ] 17.3 Implement template selection
  - Populate input field with template content
  - Allow editing before sending
  - _Requirements: 4.3_

- [ ] 17.4 Add custom template creation
  - Create "New Template" button
  - Show template editor dialog
  - Save custom templates
  - _Requirements: 4.4_

- [ ]* 18. Write template tests
  - Test template loading and saving
  - Test template validation
  - Test default templates
  - _Requirements: 4.1-4.5_

### 4. Keyboard Shortcuts

- [ ] 19. Set up keyboard module
  - Create `src/keyboard/mod.rs` with KeyboardManager
  - Define KeyboardShortcut and ShortcutAction enums
  - _Requirements: 8.1-8.5_

- [ ] 20. Implement keyboard shortcut handling
- [ ] 20.1 Create handle_key_press method
  - Detect key combinations
  - Match to actions
  - Return action to execute
  - _Requirements: 8.1-8.5_

- [ ] 20.2 Implement shortcut actions
  - New conversation (Cmd/Ctrl+N)
  - Search (Cmd/Ctrl+K)
  - Export (Cmd/Ctrl+E)
  - Save (Cmd/Ctrl+S)
  - Close modals (Escape)
  - _Requirements: 8.1-8.5_

- [ ] 21. Create keyboard shortcuts help panel
- [ ] 21.1 Build help dialog
  - Display all shortcuts in table format
  - Group by category
  - Style with hacker theme
  - _Requirements: 8.5_

- [ ] 21.2 Add help shortcut (Cmd/Ctrl+/)
  - Toggle help panel visibility
  - _Requirements: 8.5_

- [ ] 22. Integrate keyboard handling into app
  - Add keyboard event subscription
  - Handle shortcuts in update method
  - Prevent conflicts with text input
  - _Requirements: 8.1-8.5_

### 5. Conversation Pinning

- [ ] 23. Extend conversation data model
  - Add is_pinned field to Conversation struct
  - Update serialization
  - _Requirements: 10.1_

- [ ] 24. Implement pin functionality
- [ ] 24.1 Add pin/unpin methods
  - Toggle is_pinned flag
  - Save conversation
  - Update metadata
  - _Requirements: 10.2_

- [ ] 24.2 Sort conversations with pinned first
  - Modify conversation list sorting
  - Keep pinned at top
  - Sort unpinned by date
  - _Requirements: 10.2_

- [ ] 25. Create pin UI
- [ ] 25.1 Add pin option to context menu
  - Show "Pin" or "Unpin" based on state
  - _Requirements: 10.1_

- [ ] 25.2 Display pin indicator
  - Show pin icon on pinned conversations
  - Style with theme colors
  - _Requirements: 10.3_

- [ ] 26. Persist pin state
  - Save is_pinned with conversation
  - Load on startup
  - _Requirements: 10.4_

## Phase 2: Enhanced Features

### 6. File Attachments

- [ ] 27. Set up attachments module
  - Create `src/attachments/mod.rs` with AttachmentHandler
  - Create `src/attachments/file_reader.rs`
  - Create `src/attachments/image_handler.rs`
  - Create attachments directory for storage
  - _Requirements: 3.1_

- [ ] 28. Implement attachment data structures
  - Define Attachment struct with FileType enum
  - Define AttachmentHandler with validation
  - _Requirements: 3.4_

- [ ] 29. Implement file reading
- [ ] 29.1 Create read_file method
  - Read file from disk
  - Detect file type using mime_guess
  - Store file content
  - _Requirements: 3.2_

- [ ] 29.2 Implement file validation
  - Check file size (max 10MB)
  - Validate file type
  - Return errors for invalid files
  - _Requirements: 3.4_

- [ ] 29.3 Generate file previews
  - Create text preview for text files
  - Generate thumbnail for images
  - _Requirements: 3.6_

- [ ] 30. Create attachment UI
- [ ] 30.1 Add attachment button to input area
  - Create paperclip icon button
  - Open file dialog on click
  - _Requirements: 3.1_

- [ ] 30.2 Display attached files
  - Show file chips below input
  - Display file name and size
  - Add remove button for each file
  - _Requirements: 3.3_

- [ ] 30.3 Implement drag-and-drop
  - Detect file drops on input area
  - Add files to attachment list
  - Show drop zone indicator
  - _Requirements: 3.2_

- [ ] 31. Integrate attachments with messages
- [ ] 31.1 Extend ChatMessage model
  - Add attachments field
  - Update serialization
  - _Requirements: 3.5_

- [ ] 31.2 Include attachments in API requests
  - Encode file content (base64 for images)
  - Add to request payload
  - _Requirements: 3.5_

- [ ] 31.3 Display attachments in message history
  - Show file indicators in messages
  - Display image thumbnails
  - Add download/view buttons
  - _Requirements: 3.6_

- [ ]* 32. Write attachment tests
  - Test file reading and validation
  - Test file type detection
  - Test size limits
  - _Requirements: 3.1-3.6_

### 7. Token Usage Tracking

- [ ] 33. Extend message and conversation models
  - Add token_count to ChatMessage
  - Add total token_count to Conversation
  - Update serialization
  - _Requirements: 5.1, 5.2_

- [ ] 34. Implement token parsing
- [ ] 34.1 Parse token data from API responses
  - Extract input_tokens and output_tokens
  - Handle different response formats
  - _Requirements: 5.4_

- [ ] 34.2 Update token counts
  - Add tokens to message
  - Update conversation total
  - Save with conversation
  - _Requirements: 5.1, 5.2_

- [ ] 35. Create token display UI
- [ ] 35.1 Show token count per message
  - Display below message content
  - Use muted color
  - _Requirements: 5.1_

- [ ] 35.2 Show total tokens in header
  - Display conversation total
  - Update in real-time
  - _Requirements: 5.3_

- [ ] 35.3 Add token breakdown
  - Show input vs output tokens
  - Display on hover or in tooltip
  - _Requirements: 5.4_

- [ ] 36. Persist token data
  - Save token counts with messages
  - Save totals with conversation
  - _Requirements: 5.5_

### 8. Regenerate Response

- [ ] 37. Implement regenerate functionality
- [ ] 37.1 Add regenerate button to AI messages
  - Show button on hover or always visible
  - Style with hacker theme
  - _Requirements: 6.1_

- [ ] 37.2 Implement regenerate logic
  - Get previous user message
  - Resend to backend
  - Replace existing AI response
  - _Requirements: 6.2, 6.3_

- [ ] 37.3 Handle regeneration state
  - Show loading indicator during regeneration
  - Disable regenerate button while loading
  - _Requirements: 6.2_

- [ ] 38. Maintain conversation history
  - Keep messages before regenerated response
  - Update conversation file
  - _Requirements: 6.4, 6.5_

- [ ] 39. Track regeneration count
  - Add regeneration_count to ChatMessage
  - Increment on each regeneration
  - Display count if > 0
  - _Requirements: 6.5_

### 9. Folders and Tags

- [ ] 40. Set up folders module
  - Create `src/folders/mod.rs` with FolderManager
  - Create `src/folders/tags.rs` for tag management
  - Create folders directory structure
  - _Requirements: 9.1_

- [ ] 41. Implement folder data structures
  - Define Folder struct with parent_id for nesting
  - Define Tag struct with color
  - Create FolderManager
  - _Requirements: 9.1, 9.4_

- [ ] 42. Implement folder management
- [ ] 42.1 Create folder creation
  - Add create_folder method
  - Support nested folders
  - Save to folders file
  - _Requirements: 9.1_

- [ ] 42.2 Implement move to folder
  - Add move_to_folder method
  - Update conversation metadata
  - _Requirements: 9.2_

- [ ] 42.3 Add folder deletion
  - Remove folder and move conversations to parent
  - Update all affected conversations
  - _Requirements: 9.1_

- [ ] 43. Implement tag management
- [ ] 43.1 Create tag addition
  - Add add_tag method
  - Support multiple tags per conversation
  - Save with conversation
  - _Requirements: 9.4_

- [ ] 43.2 Implement tag removal
  - Remove tag from conversation
  - Update conversation file
  - _Requirements: 9.4_

- [ ] 44. Create folders UI
- [ ] 44.1 Build folder tree in sidebar
  - Display folders with expand/collapse
  - Show conversation count per folder
  - Support nested folders
  - _Requirements: 9.3_

- [ ] 44.2 Implement drag-and-drop for folders
  - Allow dragging conversations to folders
  - Show drop target indicator
  - _Requirements: 9.2_

- [ ] 44.3 Add folder context menu
  - Rename, delete, new subfolder options
  - _Requirements: 9.1_

- [ ] 45. Create tags UI
- [ ] 45.1 Display tags on conversations
  - Show tag chips with colors
  - Truncate if too many tags
  - _Requirements: 9.4_

- [ ] 45.2 Add tag editor
  - Create tag input in conversation settings
  - Support tag creation and removal
  - Color picker for tags
  - _Requirements: 9.4_

- [ ] 45.3 Implement tag filtering
  - Add tag filter dropdown
  - Show only conversations with selected tag
  - _Requirements: 9.5_

- [ ] 46. Extend conversation model
  - Add folder_id field
  - Add tags array
  - Update serialization
  - _Requirements: 9.2, 9.4_

- [ ] 47. Persist folder and tag data
  - Save folders to folders.json
  - Save tags with conversations
  - Load on startup
  - _Requirements: 9.6_

### 10. Draft Auto-save

- [ ] 48. Set up drafts module
  - Create `src/drafts/mod.rs` with DraftManager
  - Create drafts directory structure
  - _Requirements: 15.1_

- [ ] 49. Implement draft data structures
  - Define Draft struct with conversation_id and content
  - Create DraftManager with debouncing
  - _Requirements: 15.1_

- [ ] 50. Implement draft saving
- [ ] 50.1 Create save_draft method
  - Debounce saves (2 seconds)
  - Save to drafts file
  - Handle errors gracefully
  - _Requirements: 15.1_

- [ ] 50.2 Implement draft loading
  - Load draft for conversation
  - Restore to input field
  - _Requirements: 15.3_

- [ ] 50.3 Add draft clearing
  - Clear draft when message sent
  - Clear on explicit user action
  - _Requirements: 15.4_

- [ ] 51. Integrate drafts with app
- [ ] 51.1 Save draft on input change
  - Trigger save after 2 seconds of inactivity
  - _Requirements: 15.1_

- [ ] 51.2 Save draft on conversation switch
  - Save current draft before switching
  - Load draft for new conversation
  - _Requirements: 15.2_

- [ ] 51.3 Persist drafts between sessions
  - Save drafts to file on app close
  - Load drafts on app start
  - _Requirements: 15.5_

## Phase 3: Advanced Features

### 11. Message Branching

- [ ] 52. Extend conversation model for branching
  - Add parent_conversation_id field
  - Add branch_point field (message index)
  - Update serialization
  - _Requirements: 7.2_

- [ ] 53. Implement branch creation
- [ ] 53.1 Offer branch option on edit
  - Show dialog when editing message
  - Options: "Edit in place" or "Create branch"
  - _Requirements: 7.1_

- [ ] 53.2 Create branch logic
  - Copy conversation up to edit point
  - Create new conversation with edited message
  - Link to parent conversation
  - _Requirements: 7.2_

- [ ] 54. Create branch UI
- [ ] 54.1 Display branch indicators
  - Show branch icon on branched conversations
  - Display parent conversation link
  - _Requirements: 7.3_

- [ ] 54.2 Implement branch navigation
  - Add "View parent" button
  - Show branch tree visualization
  - _Requirements: 7.4_

- [ ] 55. Persist branch relationships
  - Save parent_id and branch_point
  - Load branch data on startup
  - _Requirements: 7.5_

### 12. Voice Input

- [ ] 56. Set up voice module
  - Create `src/voice/mod.rs` with VoiceInput
  - Create `src/voice/transcription.rs`
  - Add cpal and hound dependencies
  - _Requirements: 18.1_

- [ ] 57. Implement audio capture
- [ ] 57.1 Request microphone permissions
  - Check for microphone access
  - Request permissions if needed
  - Handle permission denial
  - _Requirements: 18.2_

- [ ] 57.2 Implement recording
  - Start audio capture using cpal
  - Store audio in buffer
  - Show recording indicator
  - _Requirements: 18.3_

- [ ] 57.3 Stop recording
  - End audio capture
  - Return audio buffer
  - _Requirements: 18.4_

- [ ] 58. Implement transcription
- [ ] 58.1 Add transcription service
  - Support local (whisper-rs) or remote (API)
  - Configure in settings
  - _Requirements: 18.4_

- [ ] 58.2 Transcribe audio
  - Send audio to transcription service
  - Parse transcription result
  - Handle errors
  - _Requirements: 18.4_

- [ ] 59. Create voice input UI
- [ ] 59.1 Add microphone button
  - Create mic icon button in input area
  - Toggle recording on click
  - _Requirements: 18.1_

- [ ] 59.2 Show recording indicator
  - Display pulsing animation
  - Show recording timer
  - Add waveform visualization
  - _Requirements: 18.3_

- [ ] 59.3 Insert transcribed text
  - Add transcription to input field
  - Allow editing before sending
  - _Requirements: 18.5_

- [ ]* 60. Write voice input tests
  - Test audio capture
  - Test transcription (mock)
  - Test error handling
  - _Requirements: 18.1-18.5_

### 13. Statistics Dashboard

- [ ] 61. Set up stats module
  - Create `src/stats/mod.rs` with StatsTracker
  - Create `src/stats/aggregator.rs`
  - _Requirements: 14.1_

- [ ] 62. Implement statistics calculation
- [ ] 62.1 Create calculate_stats method
  - Count total conversations and messages
  - Sum total tokens
  - Calculate averages
  - Group by model and date
  - _Requirements: 14.2, 14.3, 14.4_

- [ ] 62.2 Implement date range filtering
  - Filter conversations by date range
  - Recalculate stats for range
  - _Requirements: 14.5_

- [ ] 63. Create statistics UI
- [ ] 63.1 Build statistics panel
  - Create modal dialog with tabs
  - Overview, Models, Timeline tabs
  - _Requirements: 14.1_

- [ ] 63.2 Display overview statistics
  - Show total conversations, messages, tokens
  - Display averages
  - Use large, readable numbers
  - _Requirements: 14.2, 14.3_

- [ ] 63.3 Show model usage
  - Display model names and usage counts
  - Show as bar chart or list
  - _Requirements: 14.4_

- [ ] 63.4 Add timeline view
  - Show conversations over time
  - Display as simple chart
  - _Requirements: 14.4_

- [ ] 63.5 Add date range selector
  - Dropdown for common ranges (week, month, year, all)
  - Custom date picker
  - _Requirements: 14.5_

- [ ] 64. Persist statistics
  - Save stats to stats.json
  - Update incrementally
  - Load on startup
  - _Requirements: 14.1_

### 14. System Prompt Configuration

- [ ] 65. Extend conversation model
  - Add system_prompt field
  - Update serialization
  - _Requirements: 16.1_

- [ ] 66. Implement system prompt functionality
- [ ] 66.1 Add system prompt to API requests
  - Include in request payload
  - Handle different API formats
  - _Requirements: 16.2_

- [ ] 67. Create system prompt UI
- [ ] 67.1 Add system prompt editor
  - Create text area in conversation settings
  - Show character count
  - _Requirements: 16.1_

- [ ] 67.2 Display system prompt indicator
  - Show icon when custom prompt is active
  - Display prompt on hover
  - _Requirements: 16.3_

- [ ] 67.3 Add clear prompt button
  - Remove custom prompt
  - Revert to default behavior
  - _Requirements: 16.4_

- [ ] 68. Persist system prompts
  - Save with conversation data
  - Load on conversation load
  - _Requirements: 16.5_

### 15. Multi-line Input

- [ ] 69. Enhance input field
- [ ] 69.1 Support multi-line input
  - Expand input on Shift+Enter
  - Support up to 20 visible lines
  - Add scrollbar for overflow
  - _Requirements: 12.1, 12.2, 12.3_

- [ ] 69.2 Maintain keyboard shortcuts
  - Enter sends message
  - Shift+Enter adds new line
  - _Requirements: 12.4_

- [ ] 69.3 Preserve formatting
  - Keep line breaks in messages
  - Display properly in chat history
  - _Requirements: 12.5_

### 16. Response Comparison

- [ ] 70. Extend message model
  - Add alternatives array to ChatMessage
  - Store multiple responses for same prompt
  - _Requirements: 17.1_

- [ ] 71. Implement comparison functionality
- [ ] 71.1 Store alternative responses
  - Save each regenerated response
  - Keep up to 3 alternatives
  - _Requirements: 17.1_

- [ ] 71.2 Create comparison view
  - Display responses in columns
  - Highlight differences
  - _Requirements: 17.2, 17.3_

- [ ] 71.3 Implement response selection
  - Allow choosing which response to keep
  - Update conversation with selected response
  - _Requirements: 17.4_

- [ ] 72. Create comparison UI
- [ ] 72.1 Add comparison button
  - Show when alternatives exist
  - Open comparison view
  - _Requirements: 17.1_

- [ ] 72.2 Build comparison layout
  - Side-by-side columns
  - Synchronized scrolling
  - _Requirements: 17.2_

- [ ] 72.3 Add selection controls
  - Radio buttons or checkboxes
  - Confirm selection button
  - _Requirements: 17.4_

### 17. Streaming Controls

- [ ] 73. Implement streaming controls
- [ ] 73.1 Add pause/resume functionality
  - Pause token display while continuing to receive
  - Resume display from paused point
  - _Requirements: 11.2, 11.3_

- [ ] 73.2 Add speed control
  - Slider for display speed (0.5x to 2x)
  - Adjust token display rate
  - _Requirements: 11.4_

- [ ] 74. Create streaming controls UI
- [ ] 74.1 Add control buttons
  - Pause/Resume button during streaming
  - Stop button (already exists)
  - _Requirements: 11.1_

- [ ] 74.2 Add speed slider
  - Display current speed
  - Update in real-time
  - _Requirements: 11.4_

- [ ] 75. Persist streaming preferences
  - Save speed preference
  - Load on startup
  - _Requirements: 11.5_

### 18. Response Formatting Options

- [ ] 76. Implement formatting modes
- [ ] 76.1 Add compact mode
  - Reduce spacing and padding
  - Smaller font size
  - _Requirements: 13.2_

- [ ] 76.2 Add expanded mode
  - Increase spacing for readability
  - Larger font size
  - _Requirements: 13.3_

- [ ] 76.3 Add font toggle
  - Switch between monospace and proportional
  - Apply to all messages
  - _Requirements: 13.4_

- [ ] 77. Create formatting UI
- [ ] 77.1 Add formatting menu
  - Dropdown in settings or header
  - Options for compact/expanded/normal
  - Font toggle
  - _Requirements: 13.1_

- [ ] 78. Persist formatting preferences
  - Save per conversation
  - Load on conversation load
  - _Requirements: 13.5_

## Phase 4: Collaboration Features

### 19. Conversation Import

- [ ] 79. Implement import functionality
- [ ] 79.1 Create import parser
  - Support JSON format
  - Support Markdown format
  - Support ChatGPT export format
  - _Requirements: 19.2_

- [ ] 79.2 Validate imported data
  - Check required fields
  - Validate message structure
  - Handle errors gracefully
  - _Requirements: 19.3_

- [ ] 79.3 Create conversations from import
  - Generate new conversation IDs
  - Save to conversations directory
  - Update metadata
  - _Requirements: 19.4_

- [ ] 80. Create import UI
- [ ] 80.1 Add import option to menu
  - File menu or settings
  - _Requirements: 19.1_

- [ ] 80.2 Show import dialog
  - File picker for import file
  - Format selection
  - Progress indicator
  - _Requirements: 19.3_

- [ ] 80.3 Display import results
  - Show success count
  - List any errors
  - _Requirements: 19.5_

### 20. Conversation Sharing (Basic)

- [ ] 81. Implement basic sharing
- [ ] 81.1 Generate shareable export
  - Create JSON export with metadata
  - Include share permissions
  - _Requirements: 20.2_

- [ ] 81.2 Create share link/file
  - Generate unique identifier
  - Save to shared directory
  - _Requirements: 20.2_

- [ ] 82. Create sharing UI
- [ ] 82.1 Add share option to menu
  - Conversation context menu
  - _Requirements: 20.1_

- [ ] 82.2 Show share dialog
  - Permission options (view/edit)
  - Generate link or file
  - Copy to clipboard
  - _Requirements: 20.3_

- [ ] 83. Implement shared conversation viewing
- [ ] 83.1 Load shared conversations
  - Parse shared file
  - Display in read-only mode
  - _Requirements: 20.4_

- [ ] 83.2 Handle edit permissions
  - Allow editing if permitted
  - Sync changes (basic file-based)
  - _Requirements: 20.5_

## Integration & Polish

- [ ] 84. Update configuration system
  - Add new settings to config.toml
  - Add feature flags
  - Add attachment settings
  - Add voice settings
  - _Requirements: All_

- [ ] 85. Update error handling
  - Add new error types
  - Improve error messages
  - Add error recovery
  - _Requirements: All_

- [ ] 86. Performance optimization
  - Optimize search indexing
  - Lazy load conversations
  - Cache parsed markdown
  - Debounce expensive operations
  - _Requirements: All_

- [ ] 87. Update documentation
  - Update README with new features
  - Add feature documentation
  - Create user guide
  - Add keyboard shortcuts reference
  - _Nice to have_

- [ ]* 88. Comprehensive testing
  - Test all features together
  - Test with large datasets
  - Test edge cases
  - Test cross-platform
  - _Requirements: All_

- [ ] 89. UI polish
  - Ensure consistent styling
  - Add animations and transitions
  - Improve accessibility
  - Test responsive layout
  - _Nice to have_

- [ ] 90. Create migration guide
  - Document breaking changes
  - Provide upgrade path
  - Test migration from old version
  - _Nice to have_
