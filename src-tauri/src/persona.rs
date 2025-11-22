use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// Represents a predefined AI persona with specific behavioral characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Persona {
    /// Validates that a persona has all required fields
    /// Requirements: 4.3
    pub fn validate(&self) -> bool {
        !self.id.is_empty()
            && !self.name.is_empty()
            && !self.icon.is_empty()
            && !self.system_prompt.is_empty()
    }

    /// Returns the built-in default persona
    /// Requirements: 4.1
    pub fn default_persona() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Assistant".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful AI assistant. Provide clear, accurate, and concise responses to user questions.".to_string(),
            description: Some("General purpose assistant".to_string()),
        }
    }
}

/// Configuration structure for loading personas from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfig {
    pub personas: Vec<Persona>,
}

/// Manages persona configuration and state
pub struct PersonaManager {
    personas: Vec<Persona>,
    active_persona: Mutex<Option<String>>,
}

impl PersonaManager {
    /// Creates a new PersonaManager with the default persona
    /// Requirements: 4.1
    pub fn new() -> Self {
        Self {
            personas: vec![Persona::default_persona()],
            active_persona: Mutex::new(None),
        }
    }

    /// Loads personas from a configuration file
    /// Requirements: 4.2, 4.4
    pub fn load_personas_from_file(config_path: PathBuf) -> Result<Vec<Persona>, String> {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<PersonaConfig>(&content) {
                    Ok(config) => {
                        let mut valid_personas = Vec::new();
                        let mut seen_ids = std::collections::HashSet::new();

                        for persona in config.personas {
                            // Validate persona
                            if !persona.validate() {
                                log::error!(
                                    "Invalid persona configuration: missing required fields for persona '{}'",
                                    persona.id
                                );
                                continue;
                            }

                            // Check for duplicate IDs
                            if seen_ids.contains(&persona.id) {
                                log::warn!(
                                    "Duplicate persona ID '{}' found, skipping duplicate",
                                    persona.id
                                );
                                continue;
                            }

                            seen_ids.insert(persona.id.clone());
                            valid_personas.push(persona);
                        }

                        Ok(valid_personas)
                    }
                    Err(e) => {
                        log::error!("Failed to parse persona configuration: {}", e);
                        Err(format!("Failed to parse persona configuration: {}", e))
                    }
                }
            }
            Err(e) => {
                log::warn!("Could not read persona config file: {}", e);
                Ok(Vec::new()) // Return empty vec if file doesn't exist
            }
        }
    }

    /// Loads additional personas from configuration and adds them to the manager
    /// Requirements: 4.2
    pub fn load_additional_personas(&mut self, config_path: PathBuf) {
        match Self::load_personas_from_file(config_path) {
            Ok(personas) => {
                for persona in personas {
                    // Don't add if ID already exists
                    if !self.personas.iter().any(|p| p.id == persona.id) {
                        self.personas.push(persona);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to load additional personas: {}", e);
            }
        }
    }

    /// Gets a persona by ID
    pub fn get_persona(&self, id: &str) -> Option<&Persona> {
        self.personas.iter().find(|p| p.id == id)
    }

    /// Gets all available personas
    pub fn get_all_personas(&self) -> &[Persona] {
        &self.personas
    }

    /// Sets the active persona by ID
    /// Requirements: 2.1, 3.5
    pub fn set_active_persona(&self, persona_id: Option<String>) -> Result<(), String> {
        if let Some(id) = &persona_id {
            // Verify the persona exists
            if self.get_persona(id).is_none() {
                log::warn!("Attempted to set non-existent persona: {}", id);
                return Err(format!("Persona '{}' not found", id));
            }
        }

        match self.active_persona.lock() {
            Ok(mut active) => {
                *active = persona_id;
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to lock active persona: {}", e);
                Err("Failed to set active persona".to_string())
            }
        }
    }

    /// Gets the currently active persona
    /// Requirements: 3.5
    pub fn get_active_persona(&self) -> Option<Persona> {
        match self.active_persona.lock() {
            Ok(active) => {
                if let Some(id) = active.as_ref() {
                    self.get_persona(id).cloned()
                } else {
                    None
                }
            }
            Err(e) => {
                log::error!("Failed to lock active persona: {}", e);
                None
            }
        }
    }
}

impl Default for PersonaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_default_persona_is_valid() {
        let persona = Persona::default_persona();
        assert!(persona.validate());
        assert_eq!(persona.id, "default");
        assert!(!persona.system_prompt.is_empty());
    }

    #[test]
    fn test_persona_validation_rejects_empty_id() {
        let persona = Persona {
            id: "".to_string(),
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test prompt".to_string(),
            description: None,
        };
        assert!(!persona.validate());
    }

    #[test]
    fn test_persona_validation_rejects_empty_name() {
        let persona = Persona {
            id: "test".to_string(),
            name: "".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test prompt".to_string(),
            description: None,
        };
        assert!(!persona.validate());
    }

    #[test]
    fn test_persona_validation_rejects_empty_icon() {
        let persona = Persona {
            id: "test".to_string(),
            name: "Test".to_string(),
            icon: "".to_string(),
            system_prompt: "Test prompt".to_string(),
            description: None,
        };
        assert!(!persona.validate());
    }

    #[test]
    fn test_persona_validation_rejects_empty_system_prompt() {
        let persona = Persona {
            id: "test".to_string(),
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "".to_string(),
            description: None,
        };
        assert!(!persona.validate());
    }

    #[test]
    fn test_persona_manager_starts_with_default() {
        let manager = PersonaManager::new();
        assert_eq!(manager.get_all_personas().len(), 1);
        assert_eq!(manager.get_all_personas()[0].id, "default");
    }

    #[test]
    fn test_set_and_get_active_persona() {
        let manager = PersonaManager::new();
        
        // Initially no active persona
        assert!(manager.get_active_persona().is_none());
        
        // Set active persona to default
        assert!(manager.set_active_persona(Some("default".to_string())).is_ok());
        
        // Should now return the default persona
        let active = manager.get_active_persona();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "default");
        
        // Clear active persona
        assert!(manager.set_active_persona(None).is_ok());
        assert!(manager.get_active_persona().is_none());
    }

    #[test]
    fn test_set_nonexistent_persona_fails() {
        let manager = PersonaManager::new();
        let result = manager.set_active_persona(Some("nonexistent".to_string()));
        assert!(result.is_err());
    }

    // **Feature: persona-switcher, Property 10: Persona validation**
    // **Validates: Requirements 4.3, 4.4**
    // 
    // For any persona definition, if it lacks a required field (id, name, icon, or system_prompt),
    // it should be rejected and excluded from available personas.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_persona_with_missing_field_is_invalid(
            id in prop::option::of(prop::string::string_regex("[a-z0-9-]{1,20}").unwrap()),
            name in prop::option::of(prop::string::string_regex("[A-Za-z ]{1,30}").unwrap()),
            icon in prop::option::of(prop::string::string_regex(".{1,5}").unwrap()),
            system_prompt in prop::option::of(prop::string::string_regex(".{1,100}").unwrap()),
        ) {
            let persona = Persona {
                id: id.unwrap_or_default(),
                name: name.unwrap_or_default(),
                icon: icon.unwrap_or_default(),
                system_prompt: system_prompt.unwrap_or_default(),
                description: None,
            };
            
            // If any required field is empty, validation should fail
            let has_empty_field = persona.id.is_empty() 
                || persona.name.is_empty() 
                || persona.icon.is_empty() 
                || persona.system_prompt.is_empty();
            
            if has_empty_field {
                prop_assert!(!persona.validate(), 
                    "Persona with empty field should be invalid: id='{}', name='{}', icon='{}', prompt='{}'",
                    persona.id, persona.name, persona.icon, persona.system_prompt);
            } else {
                prop_assert!(persona.validate(),
                    "Persona with all fields should be valid: id='{}', name='{}', icon='{}', prompt='{}'",
                    persona.id, persona.name, persona.icon, persona.system_prompt);
            }
        }
    }

    // Helper function to generate valid personas for testing
    fn valid_persona_strategy() -> impl Strategy<Value = Persona> {
        (
            prop::string::string_regex("[a-z0-9-]{1,20}").unwrap(),
            prop::string::string_regex("[A-Za-z ]{1,30}").unwrap(),
            prop::string::string_regex(".{1,5}").unwrap(),
            prop::string::string_regex(".{10,100}").unwrap(),
            prop::option::of(prop::string::string_regex(".{1,50}").unwrap()),
        )
            .prop_map(|(id, name, icon, system_prompt, description)| Persona {
                id,
                name,
                icon,
                system_prompt,
                description,
            })
    }

    // **Feature: persona-switcher, Property 9: Configuration loading completeness**
    // **Validates: Requirements 4.2**
    // 
    // For any valid persona configuration file, all valid personas in the file should be loaded
    // and available for selection.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_all_valid_personas_are_loaded(
            personas in prop::collection::vec(valid_persona_strategy(), 1..10)
        ) {
            // Create a temporary file with the persona configuration
            let temp_dir = std::env::temp_dir();
            let config_path = temp_dir.join(format!("test_personas_{}.json", std::process::id()));
            
            let config = PersonaConfig {
                personas: personas.clone(),
            };
            
            let json = serde_json::to_string_pretty(&config).unwrap();
            std::fs::write(&config_path, json).unwrap();
            
            // Load personas from the file
            let loaded_personas = PersonaManager::load_personas_from_file(config_path.clone());
            
            // Clean up
            let _ = std::fs::remove_file(&config_path);
            
            prop_assert!(loaded_personas.is_ok(), "Loading valid configuration should succeed");
            
            let loaded = loaded_personas.unwrap();
            
            // All personas in the input should be valid (by construction)
            let valid_input_personas: Vec<_> = personas.iter()
                .filter(|p| p.validate())
                .collect();
            
            // All valid personas should be loaded
            prop_assert_eq!(
                loaded.len(),
                valid_input_personas.len(),
                "All valid personas should be loaded. Expected {}, got {}",
                valid_input_personas.len(),
                loaded.len()
            );
            
            // Each valid persona should be present in the loaded set
            for input_persona in valid_input_personas {
                let found = loaded.iter().any(|p| {
                    p.id == input_persona.id
                        && p.name == input_persona.name
                        && p.icon == input_persona.icon
                        && p.system_prompt == input_persona.system_prompt
                });
                
                prop_assert!(
                    found,
                    "Persona '{}' should be loaded but was not found",
                    input_persona.id
                );
            }
        }
    }
}
