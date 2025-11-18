# Implementation Plan

## Phase 1: Foundation & Dependencies

- [x] 1. Add required dependencies to Cargo.toml
  - Add `syntect` for syntax highlighting
  - Add `uuid` for conversation IDs
  - Add `regex` for markdown parsing
  - _Requirements: All features_

- [x] 2. Create conversations directory structure
  - Create `conversations/` directory on startup
  - Create metadata.json if it doesn't exist
  - Add directory to .gitignore
  - _Requirements: 2.9_

## Phase 2: Conversation Management Core

- [x] 3. Implement Conversation data structures
  - [x] 3.1 Create Conversation struct with serialization
    - Define Conversation struct with id, name, messages, timestamps, model
    - Implement Serialize and Deserialize traits
    - Add helper methods for creating new conversations
    - _Requirements: 2.1, 2.7_
  
  - [x] 3.2 Create ConversationMetadata struct
    - Define metadata struct with preview generation
    - Implement preview truncation logic (50 chars)
    - Add sorting by updated_at
    - _Requirements: 2.10, 2.11, 2.12_
  
  - [x] 3.3 Implement ConversationManager
    - Create manager struct with load/save methods
    - Implement conversation file I/O
    - Implement metadata file management
    - Add error handling for file operations
    - _Requirements: 2.9_

- [x] 4. Add conversation management to ChatApp state
  - Add active_conversation_id field
  - Add conversations list field
  - Add conversation_context_menu state
  - Update ChatApp::create() to load conversations
  - _Requirements: 2.8_

- [x] 5. Implement conversation file operations
  - [x] 5.1 Implement save_conversation function
    - Save conversation to JSON file with UUID filename
    - Update metadata.json with conversation info
    - Handle file write errors gracefully
    - _Requirements: 2.7_
  
  - [x] 5.2 Implement load_conversation function
    - Load conversation from JSON file by ID
    - Parse and validate conversation data
    - Handle missing or corrupted files
    - _Requirements: 2.3, 2.8_
  
  - [x] 5.3 Implement delete_conversation function
    - Delete conversation JSON file
    - Update metadata.json to remove entry
    - Handle deletion errors
    - _Requirements: 2.6_

## Phase 3: Conversation UI

- [x] 6. Update sidebar to show conversations list
  - [x] 6.1 Create conversation list item widget
    - Display conversation name (bold)
    - Display preview text (muted)
    - Show timestamp
    - Highlight active conversation
    - _Requirements: 2.2, 2.10, 2.11, 2.12_
  
  - [x] 6.2 Make conversation items clickable
    - Add click handler to load conversation
    - Update active conversation state
    - Clear current chat and load selected conversation
    - _Requirements: 2.3_
  
  - [x] 6.3 Add scrollable container for conversations
    - Wrap conversation list in scrollable
    - Style scrollbar to match theme
    - _Requirements: 2.2_

- [x] 7. Implement new conversation functionality
  - Update new chat button to create new conversation
  - Generate UUID for new conversation
  - Set default name with timestamp
  - Clear current messages
  - _Requirements: 2.1_

- [x] 8. Implement conversation context menu
  - [x] 8.1 Add right-click detection on conversation items
    - Use mouse_area widget for right-click detection
    - Show context menu at cursor position
    - _Requirements: 2.4_
  
  - [x] 8.2 Create conversation context menu UI
    - Add Rename and Delete options
    - Style menu with hacker aesthetic
    - Position menu near cursor
    - _Requirements: 2.4_
  
  - [x] 8.3 Implement rename conversation
    - Show inline text input for renaming
    - Update conversation name on confirm
    - Save updated metadata
    - _Requirements: 2.5_
  
  - [x] 8.4 Implement delete conversation
    - Show confirmation (optional)
    - Delete conversation file
    - Update conversations list
    - Switch to another conversation if active was deleted
    - _Requirements: 2.6_

- [x] 9. Implement auto-save on message send
  - Save conversation after each message exchange
  - Update metadata with new timestamp
  - Debounce saves to avoid excessive I/O
  - _Requirements: 2.7_

## Phase 4: Message Context Menu

- [x] 10. Add context menu state to ChatApp
  - Add context_menu_state field
  - Add editing_message_index field
  - Add edit_message_content field
  - _Requirements: 1.1_

- [x] 11. Implement message right-click detection
  - Wrap message bubbles in mouse_area widget
  - Detect right-click events
  - Store message index and cursor position
  - _Requirements: 1.1_

- [ ] 12. Create message context menu UI
  - [ ] 12.1 Build context menu widget
    - Create floating menu with Copy, Delete, Edit options
    - Position at cursor location
    - Add semi-transparent dark background
    - Add cyan border
    - _Requirements: 1.1_
  
  - [ ] 12.2 Implement menu positioning logic
    - Adjust position if near screen edges
    - Ensure menu stays within window bounds
    - _Requirements: 1.6_
  
  - [ ] 12.3 Add click-outside-to-close behavior
    - Detect clicks outside menu
    - Close menu when clicking elsewhere
    - _Requirements: 1.6_

- [ ] 13. Implement Copy action
  - Extract message content
  - Copy to clipboard using arboard
  - Close context menu
  - _Requirements: 1.2_

- [ ] 14. Implement Delete action
  - Remove message from chat_history
  - Update conversation file
  - Close context menu
  - _Requirements: 1.3, 1.8_

- [ ] 15. Implement Edit action
  - [ ] 15.1 Create edit mode UI
    - Replace message bubble with text input
    - Pre-fill with current message content
    - Add Save and Cancel buttons
    - Show warning about clearing subsequent messages
    - _Requirements: 1.4_
  
  - [ ] 15.2 Implement edit confirmation
    - Update message content
    - Remove all messages after edited message
    - Save updated conversation
    - Exit edit mode
    - _Requirements: 1.5_
  
  - [ ] 15.3 Disable edit for AI messages
    - Hide or disable Edit option for assistant messages
    - Only allow editing user messages
    - _Requirements: 1.7_

## Phase 5: Code Block Formatting

- [x] 16. Implement markdown parser
  - [x] 16.1 Create MessageSegment enum
    - Define variants for Text, CodeBlock, InlineCode, Bold, Italic, ListItem
    - _Requirements: 3.1_
  
  - [x] 16.2 Implement code block detection
    - Use regex to find ```language...``` blocks
    - Extract language and code content
    - Handle blocks without language specification
    - _Requirements: 3.1, 3.8_
  
  - [x] 16.3 Implement inline code detection
    - Use regex to find `code` patterns
    - Handle escaped backticks
    - _Requirements: 3.9_
  
  - [x] 16.4 Implement bold/italic detection
    - Detect **bold** and *italic* patterns
    - Handle nested formatting
    - _Requirements: 3.10_
  
  - [x] 16.5 Implement list detection
    - Detect lines starting with - or *
    - Handle nested lists
    - _Requirements: 3.11_
  
  - [x] 16.6 Create parse_message function
    - Parse message into MessageSegment vector
    - Handle mixed content (text + code + formatting)
    - Cache parsed results per message
    - _Requirements: 3.1_

- [ ] 17. Implement syntax highlighting
  - [ ] 17.1 Initialize syntect
    - Load syntax definitions
    - Load dark theme (base16-ocean.dark)
    - Cache syntax set for performance
    - _Requirements: 3.2_
  
  - [ ] 17.2 Create highlight_code function
    - Take language and code as input
    - Return highlighted HTML or styled text
    - Handle unknown languages gracefully
    - _Requirements: 3.2, 3.8_

- [x] 18. Create code block renderer widget
  - [x] 18.1 Build code block container
    - Dark background (#1a1a1a)
    - Cyan border
    - Padding and spacing
    - _Requirements: 3.3_
  
  - [x] 18.2 Add language label
    - Show language in top-left corner
    - Use muted color
    - _Requirements: 3.2_
  
  - [x] 18.3 Add Copy button
    - Position in top-right corner
    - Show "Copy" text normally
    - Add hover state
    - _Requirements: 3.4, 3.6_
  
  - [x] 18.4 Implement copy functionality
    - Copy code content to clipboard
    - Show "Copied!" feedback for 2 seconds
    - Handle clipboard errors
    - _Requirements: 3.5, 3.7_

- [x] 19. Update message rendering to use parsed segments
  - [x] 19.1 Modify message bubble rendering
    - Iterate through MessageSegments
    - Render each segment with appropriate styling
    - _Requirements: 3.1_
  
  - [x] 19.2 Render inline code
    - Monospace font
    - Subtle background color
    - _Requirements: 3.9_
  
  - [x] 19.3 Render bold and italic
    - Apply appropriate text styles
    - _Requirements: 3.10_
  
  - [x] 19.4 Render lists
    - Add bullet points
    - Apply proper indentation
    - _Requirements: 3.11_

## Phase 6: Integration & Polish

- [ ] 20. Update message history persistence
  - Ensure edited messages are saved correctly
  - Ensure deleted messages are removed from file
  - Test conversation switching with unsaved changes
  - _Requirements: 1.8, 2.7_

- [ ] 21. Add keyboard shortcuts (optional enhancement)
  - Cmd/Ctrl+N for new conversation
  - Escape to close context menus
  - _Nice to have_

- [ ] 22. Test and fix edge cases
  - Test with empty conversations
  - Test with very long messages
  - Test with many conversations (50+)
  - Test with large code blocks (1000+ lines)
  - Test rapid conversation switching
  - Test editing while streaming
  - _Requirements: All_

- [ ] 23. Update documentation
  - Update README with new features
  - Document conversation file format
  - Add screenshots of new features
  - _Nice to have_
