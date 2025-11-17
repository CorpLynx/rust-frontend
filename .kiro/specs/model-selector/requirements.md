# Requirements Document - Model Selector Feature

## Introduction

This feature adds a model selection dropdown to the Neural Interface Rust app (using Iced GUI framework), allowing users to choose which AI model to send their queries to. The dropdown will fetch available models from the local Ollama server and display them in a circular button next to the send button, similar to Ollama's interface.

## Requirements

### Requirement 1: Model Selector Button

**User Story:** As a user, I want a circular button next to the send button, so that I can access the model selection dropdown.

#### Acceptance Criteria

1. WHEN the app loads THEN a circular button SHALL be displayed to the left of the send button
2. WHEN the button is clicked THEN a dropdown menu SHALL appear showing available models
3. WHEN the button is hovered THEN it SHALL show a visual hover effect
4. WHEN a model is selected THEN the button SHALL display the selected model name or icon

### Requirement 2: Fetch Available Models

**User Story:** As a user, I want the app to automatically retrieve available models from my local Ollama server, so that I can see which models I have installed.

#### Acceptance Criteria

1. WHEN the app starts THEN it SHALL fetch the list of available models from the Ollama API
2. WHEN the models are fetched successfully THEN they SHALL be stored in the app state
3. WHEN the model fetch fails THEN the app SHALL show a default model option and log the error
4. WHEN the model list is empty THEN the app SHALL display a message indicating no models are available

### Requirement 3: Model Selection Dropdown

**User Story:** As a user, I want to see a dropdown list of available models when I click the model selector button, so that I can choose which model to use.

#### Acceptance Criteria

1. WHEN the model selector button is clicked THEN a dropdown menu SHALL appear below the button
2. WHEN the dropdown is open THEN it SHALL display all available models as selectable options
3. WHEN a model option is hovered THEN it SHALL show a visual hover effect
4. WHEN a model is selected THEN the dropdown SHALL close and the selected model SHALL be set as active
5. WHEN clicking outside the dropdown THEN the dropdown SHALL close without changing the selection

### Requirement 4: Send Messages to Selected Model

**User Story:** As a user, I want my messages to be sent to the currently selected model, so that I can interact with different AI models.

#### Acceptance Criteria

1. WHEN a message is sent THEN it SHALL be directed to the currently selected model
2. WHEN no model is explicitly selected THEN the app SHALL use a default model
3. WHEN the selected model is changed THEN subsequent messages SHALL use the new model
4. WHEN the API request includes the model parameter THEN it SHALL match the selected model name

### Requirement 5: Visual Design Consistency

**User Story:** As a user, I want the model selector to match the app's hacker aesthetic, so that the interface remains visually cohesive.

#### Acceptance Criteria

1. WHEN viewing the model selector button THEN it SHALL use the same color scheme (neon green, cyan)
2. WHEN the dropdown is displayed THEN it SHALL have rounded corners and matching styling
3. WHEN the button is circular THEN it SHALL have the same size and styling as the send button
4. WHEN text is displayed THEN it SHALL use the monospace font consistent with the app
