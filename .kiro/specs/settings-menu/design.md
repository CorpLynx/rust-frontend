# Design Document - Settings Menu Feature

## Overview

This design outlines the implementation of a settings menu for the Rust/Iced Neural Interface app. The feature allows users to configure backend and Ollama URLs through a modal interface that opens from a gear icon in the header.

## Architecture

### Component Structure

```
ChatApp
├── State
│   ├── settings_open: bool
│   ├── temp_backend_url: String
│   └── temp_ollama_url: String
├── Messages
│   ├── ToggleSettings
│   ├── BackendUrlChanged(String)
│   ├── OllamaUrlChanged(String)
│   └── SaveSettings
└── View
    ├── Settings Button (gear icon)
    └── Settings Modal (when open)
        ├── Backend URL Input
        ├── Ollama URL Input
        └── Save/Cancel Buttons
```

## Components and Interfaces

### 1. State Management

Add new fields to `ChatApp` struct:

```rust
pub struct ChatApp {
    // ... existing fields
    settings_open: bool,
    temp_backend_url: String,
    temp_ollama_url: String,
}
```

### 2. Message Types

Add new message variants:

```rust
pub enum Message {
    // ... existing messages
    ToggleSettings,
    BackendUrlChanged(String),
    OllamaUrlChanged(String),
    SaveSettings,
}
```

### 3. Settings Modal UI

**Layout:**
```
┌─────────────────────────────────────────┐
│  Semi-transparent dark overlay          │
│                                         │
│    ┌─────────────────────────────┐     │
│    │  ⚙ Settings                 │     │
│    ├─────────────────────────────┤     │
│    │                             │     │
│    │  Backend URL:               │     │
│    │  [input field............]  │     │
│    │                             │     │
│    │  Ollama URL:                │     │
│    │  [input field............]  │     │
│    │                             │     │
│    │  [Cancel]  [Save]           │     │
│    └─────────────────────────────┘     │
└─────────────────────────────────────────┘
```

### 4. Configuration Persistence

Update config.toml when settings are saved:

```rust
impl AppConfig {
    pub fn save(&self) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write("config.toml", toml_string)?;
        Ok(())
    }
}
```

## Data Models

No new data models needed - using existing `AppConfig` structure.

## UI Components

### Settings Button
- **Type:** Iced `button` widget
- **Icon:** ⚙ (gear emoji)
- **Position:** Header, right side
- **Styling:** Text button style, cyan color

### Settings Modal
- **Type:** Iced `Modal` or layered containers
- **Background:** Semi-transparent dark overlay
- **Panel:** Centered container with rounded corners
- **Width:** Fixed 500px
- **Styling:** Dark background, cyan borders

### Input Fields
- **Type:** Iced `text_input` widgets
- **Labels:** Text widgets above inputs
- **Styling:** Match existing input styling (HackerInputStyle)
- **Validation:** None initially (accept any string)

### Buttons
- **Save Button:** Primary style (cyan background)
- **Cancel Button:** Secondary style (text only)
- **Layout:** Right-aligned in modal footer

## Error Handling

### Config Save Failures
- **Scenario:** Cannot write to config.toml
- **Handling:** Show error message in modal
- **User Feedback:** Display error text in red

### Invalid URLs
- **Scenario:** User enters malformed URL
- **Handling:** Accept any string (validation happens at request time)
- **Future:** Add URL validation before saving

### Config Load Failures
- **Scenario:** Config file corrupted after save
- **Handling:** Fall back to defaults on next startup
- **User Feedback:** Log error

## Testing Strategy

### Manual Testing
1. Open settings modal
2. Edit URLs
3. Save settings
4. Verify config.toml updated
5. Restart app and verify settings loaded
6. Test with invalid URLs
7. Test cancel button (no changes saved)

## Implementation Notes

### Modal Implementation

Since Iced doesn't have a built-in modal, we'll use layered rendering:

```rust
// In view() method
if self.settings_open {
    // Render settings modal on top
    let modal = container(settings_panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(/* semi-transparent background */);
    
    // Layer modal over main content
    stack![main_content, modal]
} else {
    main_content
}
```

### Saving Configuration

```rust
Message::SaveSettings => {
    // Update config
    self.config.backend.url = self.temp_backend_url.clone();
    self.config.backend.ollama_url = self.temp_ollama_url.clone();
    
    // Save to file
    match self.config.save() {
        Ok(_) => {
            info!("Settings saved successfully");
            self.settings_open = false;
            // Refetch models with new Ollama URL
            return Self::fetch_models(self.config.backend.ollama_url.clone());
        }
        Err(e) => {
            error!("Failed to save settings: {}", e);
            self.error_message = Some(format!("Failed to save settings: {}", e));
        }
    }
    Command::none()
}
```

### Opening Settings

```rust
Message::ToggleSettings => {
    self.settings_open = !self.settings_open;
    if self.settings_open {
        // Load current values into temp fields
        self.temp_backend_url = self.config.backend.url.clone();
        self.temp_ollama_url = self.config.backend.ollama_url.clone();
    }
    Command::none()
}
```

## Visual Design

### Color Scheme
- **Modal Background:** `rgba(5, 5, 8, 0.9)` (dark with transparency)
- **Panel Background:** `#0D0D14` (solid dark)
- **Panel Border:** `rgba(0, 204, 255, 0.5)` (cyan)
- **Text Color:** `#00FF99` (neon green)
- **Input Background:** `#141418` (dark)
- **Button Colors:** Match existing buttons

### Layout Dimensions
- **Modal:** Full screen overlay
- **Panel Width:** 500px
- **Panel Padding:** 30px
- **Input Width:** Fill panel width
- **Button Width:** Auto (shrink to content)
- **Spacing:** 20px between elements

### Typography
- **Title:** 18px, bold
- **Labels:** 14px
- **Inputs:** 16px, monospace
- **Buttons:** 14px

## Performance Considerations

- Modal rendering is lightweight (just additional containers)
- Config save is synchronous but fast (small file)
- No performance impact when modal is closed

## Security Considerations

- URLs are not validated before saving
- Config file is plain text (no sensitive data encryption)
- User is responsible for entering correct URLs

## Future Enhancements

1. **URL Validation:** Validate URLs before saving
2. **Test Connection:** Button to test backend/Ollama connectivity
3. **More Settings:** Timeout, font size, theme options
4. **Import/Export:** Export/import config files
5. **Reset to Defaults:** Button to restore default settings
6. **Settings Categories:** Tabs for different setting groups
