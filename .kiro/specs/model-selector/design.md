# Design Document - Model Selector Feature

## Overview

This design outlines the implementation of a model selector dropdown for the Rust/Iced Neural Interface app. The feature allows users to select which AI model to use for their queries by fetching available models from the Ollama API and displaying them in a dropdown widget.

## Architecture

### Component Structure

```
ChatApp
├── State
│   ├── available_models: Vec<String>
│   ├── selected_model: Option<String>
│   └── model_dropdown_open: bool
├── Messages
│   ├── FetchModels
│   ├── ModelsReceived(Result<Vec<String>, String>)
│   └── ModelSelected(String)
└── View
    └── Input Row
        ├── Model Selector Button (circular)
        ├── Text Input
        └── Send Button
```

## Components and Interfaces

### 1. State Management

Add new fields to `ChatApp` struct:

```rust
pub struct ChatApp {
    // ... existing fields
    available_models: Vec<String>,
    selected_model: Option<String>,
    models_loading: bool,
}
```

### 2. Message Types

Add new message variants to handle model operations:

```rust
pub enum Message {
    // ... existing messages
    FetchModels,
    ModelsReceived(Result<Vec<String>, String>),
    ModelSelected(String),
}
```

### 3. Ollama API Integration

**Endpoint:** `GET http://localhost:11434/api/tags`

**Response Format:**
```json
{
  "models": [
    {
      "name": "llama2:latest",
      "modified_at": "2024-01-01T00:00:00Z",
      "size": 3826793677
    }
  ]
}
```

**Implementation:**
- Use `reqwest` to fetch models on app startup
- Parse JSON response to extract model names
- Store in `available_models` vector
- Handle errors gracefully with fallback

### 4. UI Components

#### Model Selector Button
- **Type:** Iced `pick_list` widget
- **Style:** Circular button appearance
- **Position:** Left of send button in input row
- **Width:** Fixed 150px
- **Styling:** Match send button (rounded, cyan background)

#### Dropdown Menu
- **Type:** Native Iced pick list dropdown
- **Items:** List of model names from `available_models`
- **Selected:** Display currently selected model
- **Placeholder:** "Select Model" when none selected
- **Styling:** Dark background, neon green text, cyan borders

## Data Models

### Model Information

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}
```

### Configuration

Update `config.toml` to include Ollama API endpoint:

```toml
[backend]
url = "http://localhost:8000/generate"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
```

## Error Handling

### Model Fetch Failures
- **Scenario:** Ollama server not running
- **Handling:** Log error, show default "No models available" option
- **User Feedback:** Display error in UI (optional)

### Empty Model List
- **Scenario:** No models installed
- **Handling:** Show message "No models installed"
- **User Action:** Direct user to install models via Ollama CLI

### Network Timeouts
- **Scenario:** API request times out
- **Handling:** Retry once, then fail gracefully
- **Fallback:** Use last known model list if available

## Testing Strategy

### Unit Tests
1. Test model list parsing from JSON
2. Test model selection state updates
3. Test error handling for API failures

### Integration Tests
1. Test fetching models from mock Ollama API
2. Test sending messages with selected model
3. Test UI updates when model is changed

### Manual Testing
1. Verify dropdown appears and functions correctly
2. Test with Ollama server running and stopped
3. Verify selected model persists across messages
4. Check visual styling matches hacker aesthetic

## Implementation Notes

### Iced Pick List Widget

```rust
use iced::widget::pick_list;

// In view() method
let model_selector = pick_list(
    &self.available_models[..],
    self.selected_model.as_ref(),
    Message::ModelSelected,
)
.placeholder("Select Model")
.width(Length::Fixed(150.0))
.style(/* custom style */);
```

### Fetching Models on Startup

```rust
fn new(_flags: ()) -> (Self, Command<Message>) {
    let config = AppConfig::load().unwrap_or_default();
    let app = Self::create(config);
    
    Command::batch(vec![
        Self::load_history(),
        Self::fetch_models(app.config.backend.ollama_url.clone()),
    ])
}
```

### Sending Model with Request

Update the request body to include model parameter:

```rust
let request_body = serde_json::json!({
    "prompt": prompt,
    "model": self.selected_model.as_ref().unwrap_or(&"default".to_string())
});
```

## Visual Design

### Color Scheme
- **Button Background:** `rgba(0, 204, 255, 0.12)` (cyan tint)
- **Button Border:** `rgba(0, 204, 255, 0.5)` (cyan)
- **Text Color:** `#00FF99` (neon green)
- **Hover:** `rgba(0, 255, 153, 0.2)` (green tint)
- **Dropdown Background:** `#141418` (dark)

### Layout
```
[Model Selector ▼] [Input Field........................] [↑]
     150px              flex-grow                      50px
```

### Styling Details
- Border radius: 8px (matching send button)
- Padding: 12px
- Font: Monospace, 14px
- Dropdown max height: 300px (scrollable)

## Performance Considerations

- Fetch models once on startup, cache in memory
- Debounce model selection to avoid rapid API calls
- Use async commands to avoid blocking UI
- Limit dropdown items to reasonable number (e.g., 20 models)

## Security Considerations

- Validate model names before sending to backend
- Sanitize model list from API response
- Use HTTPS for Ollama API if available
- Handle malformed JSON responses gracefully

## Future Enhancements

1. **Model Refresh Button:** Allow manual refresh of model list
2. **Model Details:** Show model size and last modified date
3. **Model Search:** Filter models in dropdown
4. **Model Favorites:** Pin frequently used models to top
5. **Model Icons:** Display icons for different model types
6. **Model Info Tooltip:** Show model details on hover
