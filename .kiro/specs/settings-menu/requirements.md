# Requirements Document - Settings Menu Feature

## Introduction

This feature adds a settings menu to the Neural Interface Rust app, allowing users to configure the backend URL where requests are forwarded. The settings will be accessible via a gear icon button in the header and will open a modal/overlay with input fields for configuration.

## Requirements

### Requirement 1: Settings Button

**User Story:** As a user, I want a settings button in the header, so that I can access the configuration menu.

#### Acceptance Criteria

1. WHEN the app loads THEN a settings button (gear icon) SHALL be displayed in the header
2. WHEN the settings button is clicked THEN a settings modal SHALL open
3. WHEN the settings button is hovered THEN it SHALL show a visual hover effect
4. WHEN the settings modal is open THEN the button SHALL indicate active state

### Requirement 2: Settings Modal/Overlay

**User Story:** As a user, I want a settings modal to appear when I click the settings button, so that I can configure the app.

#### Acceptance Criteria

1. WHEN the settings button is clicked THEN a modal overlay SHALL appear over the main interface
2. WHEN the modal is displayed THEN it SHALL have a dark semi-transparent background
3. WHEN the modal is displayed THEN it SHALL show a settings panel in the center
4. WHEN clicking outside the settings panel THEN the modal SHALL close
5. WHEN pressing ESC key THEN the modal SHALL close

### Requirement 3: Backend URL Configuration

**User Story:** As a user, I want to input and save the backend URL, so that I can direct requests to my preferred server.

#### Acceptance Criteria

1. WHEN the settings modal is open THEN it SHALL display an input field for the backend URL
2. WHEN the input field is displayed THEN it SHALL show the current backend URL value
3. WHEN I type in the input field THEN the value SHALL update in real-time
4. WHEN I click "Save" THEN the new URL SHALL be saved to the config file
5. WHEN the URL is saved THEN subsequent requests SHALL use the new URL

### Requirement 4: Ollama URL Configuration

**User Story:** As a user, I want to input and save the Ollama API URL, so that I can fetch models from my Ollama server.

#### Acceptance Criteria

1. WHEN the settings modal is open THEN it SHALL display an input field for the Ollama URL
2. WHEN the input field is displayed THEN it SHALL show the current Ollama URL value
3. WHEN I type in the input field THEN the value SHALL update in real-time
4. WHEN I click "Save" THEN the new Ollama URL SHALL be saved to the config file
5. WHEN the Ollama URL is saved THEN the app SHALL refetch available models

### Requirement 5: Settings Persistence

**User Story:** As a user, I want my settings to be saved permanently, so that they persist across app restarts.

#### Acceptance Criteria

1. WHEN I save settings THEN they SHALL be written to the config.toml file
2. WHEN the app restarts THEN it SHALL load the saved settings
3. WHEN the config file is invalid THEN the app SHALL use default values
4. WHEN settings are saved THEN a success message SHALL be displayed

### Requirement 6: Visual Design Consistency

**User Story:** As a user, I want the settings menu to match the app's hacker aesthetic, so that the interface remains visually cohesive.

#### Acceptance Criteria

1. WHEN viewing the settings modal THEN it SHALL use the same color scheme (neon green, cyan)
2. WHEN the modal is displayed THEN it SHALL have rounded corners and matching styling
3. WHEN input fields are displayed THEN they SHALL use the monospace font
4. WHEN buttons are displayed THEN they SHALL match the existing button styles
