# Implementation Plan - Model Selector Feature

- [x] 1. Update configuration to support Ollama API endpoint
  - Add `ollama_url` field to `BackendSettings` in `src/config.rs`
  - Update `config.toml` with default Ollama URL (`http://localhost:11434`)
  - _Requirements: 2.1_

- [x] 2. Create data models for Ollama API responses
  - [x] 2.1 Define `OllamaModel` struct with name, modified_at, and size fields
    - Add Serialize and Deserialize derives
    - _Requirements: 2.1, 2.2_
  
  - [x] 2.2 Define `OllamaModelsResponse` struct with models vector
    - Add Serialize and Deserialize derives
    - _Requirements: 2.1, 2.2_

- [x] 3. Add model selector state to ChatApp
  - Add `available_models: Vec<String>` field to store fetched models
  - Add `selected_model: Option<String>` field to track current selection
  - Add `models_loading: bool` field for loading state
  - Initialize fields in `create()` method
  - _Requirements: 1.1, 3.1, 4.2_

- [x] 4. Implement model fetching functionality
  - [x] 4.1 Add new message variants for model operations
    - Add `FetchModels` message variant
    - Add `ModelsReceived(Result<Vec<String>, String>)` message variant
    - Add `ModelSelected(String)` message variant
    - _Requirements: 2.1, 2.2, 3.4_
  
  - [x] 4.2 Create `fetch_models()` command function
    - Make async HTTP GET request to `{ollama_url}/api/tags`
    - Parse JSON response into `OllamaModelsResponse`
    - Extract model names into Vec<String>
    - Handle errors and return Result
    - _Requirements: 2.1, 2.2, 2.3_
  
  - [x] 4.3 Update `new()` to fetch models on startup
    - Call `fetch_models()` in Command::batch with load_history
    - _Requirements: 2.1_
  
  - [x] 4.4 Handle `ModelsReceived` message in `update()`
    - Store models in `available_models` on success
    - Log error and set empty list on failure
    - Set `models_loading` to false
    - _Requirements: 2.2, 2.3, 2.4_

- [x] 5. Implement model selection handling
  - Handle `ModelSelected` message in `update()`
  - Update `selected_model` field with chosen model
  - Log model selection for debugging
  - _Requirements: 3.4, 4.3_

- [x] 6. Update message sending to include selected model
  - Modify `send_request()` to accept model parameter
  - Add model field to request JSON body
  - Use selected_model or default if none selected
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 7. Create custom styling for model selector
  - [x] 7.1 Create `ModelSelectorStyle` struct implementing pick_list StyleSheet
    - Define active state with cyan background and border
    - Define hovered state with brighter colors
    - Set border radius to 8px for rounded appearance
    - Use neon green text color
    - _Requirements: 5.1, 5.2, 5.3, 5.4_
  
  - [x] 7.2 Create `ModelSelectorMenuStyle` for dropdown menu
    - Define dark background color
    - Define cyan border
    - Set text color to neon green
    - Add hover effects for menu items
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 8. Add model selector UI component to view
  - [x] 8.1 Create pick_list widget in input row
    - Set data source to `available_models`
    - Set selected value to `selected_model`
    - Set on_select callback to `ModelSelected` message
    - Set placeholder text to "Select Model"
    - Set width to Fixed(150.0)
    - Apply custom styling
    - _Requirements: 1.1, 1.2, 3.1, 3.2, 5.1, 5.2, 5.3_
  
  - [x] 8.2 Update input row layout
    - Add model selector before text input
    - Adjust spacing between elements
    - Ensure proper alignment with send button
    - _Requirements: 1.1, 5.3_

- [x] 9. Add error handling and user feedback
  - Display loading indicator while fetching models
  - Show error message if model fetch fails
  - Handle empty model list gracefully
  - Add logging for debugging
  - _Requirements: 2.3, 2.4_

- [x] 10. Test and verify functionality
  - Test with Ollama server running
  - Test with Ollama server stopped
  - Verify model selection persists across messages
  - Verify visual styling matches design
  - Test dropdown interaction (open/close/select)
  - _Requirements: All_
