# Implementation Plan - Settings Menu Feature

- [x] 1. Add settings state to ChatApp
  - Add `settings_open: bool` field to track modal state
  - Add `temp_backend_url: String` field for editing
  - Add `temp_ollama_url: String` field for editing
  - Initialize fields in `create()` method
  - _Requirements: 2.1, 3.2, 4.2_

- [x] 2. Add settings message variants
  - Add `ToggleSettings` message variant
  - Add `BackendUrlChanged(String)` message variant
  - Add `OllamaUrlChanged(String)` message variant
  - Add `SaveSettings` message variant
  - _Requirements: 1.2, 3.3, 4.3, 3.4_

- [x] 3. Implement config save functionality
  - [x] 3.1 Add `save()` method to `AppConfig` in config.rs
    - Serialize config to TOML string
    - Write to config.toml file
    - Handle errors and return Result
    - _Requirements: 5.1, 5.3_
  
  - [x] 3.2 Add toml dependency to Cargo.toml
    - Add `toml = "0.8"` to dependencies
    - _Requirements: 5.1_

- [x] 4. Handle settings messages in update()
  - [x] 4.1 Handle `ToggleSettings` message
    - Toggle `settings_open` boolean
    - Load current URLs into temp fields when opening
    - _Requirements: 1.2, 2.1_
  
  - [x] 4.2 Handle `BackendUrlChanged` message
    - Update `temp_backend_url` with new value
    - _Requirements: 3.3_
  
  - [x] 4.3 Handle `OllamaUrlChanged` message
    - Update `temp_ollama_url` with new value
    - _Requirements: 4.3_
  
  - [x] 4.4 Handle `SaveSettings` message
    - Update config with temp values
    - Call config.save() method
    - Close settings modal on success
    - Show error message on failure
    - Refetch models if Ollama URL changed
    - _Requirements: 3.4, 4.4, 5.1, 5.4_

- [x] 5. Create settings modal styles
  - [x] 5.1 Create `SettingsModalStyle` for overlay background
    - Define semi-transparent dark background
    - _Requirements: 2.2, 6.1_
  
  - [x] 5.2 Create `SettingsPanelStyle` for settings panel
    - Define solid dark background
    - Define cyan border
    - Set border radius to 16px
    - _Requirements: 2.3, 6.1, 6.2_

- [x] 6. Add settings button to header
  - Create button with gear icon (⚙)
  - Set on_press to `ToggleSettings` message
  - Apply text button styling
  - Position in header (right side)
  - _Requirements: 1.1, 1.3_

- [x] 7. Create settings modal UI
  - [x] 7.1 Create settings panel content
    - Add title text "⚙ Settings"
    - Add label and input for Backend URL
    - Add label and input for Ollama URL
    - Add Save and Cancel buttons
    - Apply proper spacing and padding
    - _Requirements: 3.1, 4.1, 6.3, 6.4_
  
  - [x] 7.2 Create modal overlay
    - Create full-screen container with semi-transparent background
    - Center the settings panel
    - Make clickable to close (optional)
    - _Requirements: 2.1, 2.2, 2.3_
  
  - [x] 7.3 Conditionally render modal in view()
    - Check if `settings_open` is true
    - Layer modal over main content when open
    - Use stack or container layering
    - _Requirements: 2.1, 2.4_

- [x] 8. Add keyboard support
  - Handle ESC key to close modal
  - Handle Enter key in inputs (optional)
  - _Requirements: 2.5_

- [x] 9. Test and verify functionality
  - Test opening/closing settings modal
  - Test editing URLs
  - Test saving settings
  - Verify config.toml is updated
  - Test app restart with saved settings
  - Test cancel button (no changes)
  - Verify models refetch after Ollama URL change
  - _Requirements: All_
