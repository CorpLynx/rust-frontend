mod commands;
mod persona;
pub mod config;
pub mod network;

use commands::{
    get_models, send_message_stream, get_chat_history, new_conversation,
    get_personas, set_active_persona, get_active_persona,
    add_remote_endpoint, remove_remote_endpoint, update_remote_endpoint,
    list_remote_endpoints, test_remote_endpoint,
    set_connection_mode, get_connection_mode, set_active_remote_endpoint,
    get_active_endpoint
};
use persona::PersonaManager;
use config::AppConfig;
use network::{ConnectionManager, OllamaClient};
use std::sync::{Arc, RwLock};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
      get_models,
      send_message_stream,
      get_chat_history,
      new_conversation,
      get_personas,
      set_active_persona,
      get_active_persona,
      add_remote_endpoint,
      remove_remote_endpoint,
      update_remote_endpoint,
      list_remote_endpoints,
      test_remote_endpoint,
      set_connection_mode,
      get_connection_mode,
      set_active_remote_endpoint,
      get_active_endpoint,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      // Initialize PersonaManager with default persona
      let mut persona_manager = PersonaManager::new();
      
      // Try to load additional personas from config file
      // Look for personas.json in the current directory for now
      let persona_config_path = std::path::PathBuf::from("personas.json");
      persona_manager.load_additional_personas(persona_config_path);
      
      // Store the PersonaManager in app state
      app.manage(persona_manager);
      
      // Load or create application configuration
      // Requirements: 1.5, 6.4
      let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Failed to load config, using defaults: {}", e);
        AppConfig::default()
      });
      
      // Perform migration if needed
      let mut config = config;
      config.backend.migrate_ollama_url();
      
      // Save the migrated config
      if let Err(e) = config.save() {
        eprintln!("Failed to save migrated config: {}", e);
      }
      
      // Wrap config in Arc<RwLock<>> for shared access
      let config = Arc::new(RwLock::new(config));
      
      // Initialize OllamaClient
      let client = Arc::new(OllamaClient::new());
      
      // Initialize ConnectionManager
      // Requirements: 6.4
      let connection_manager = Arc::new(ConnectionManager::new(
        Arc::clone(&config),
        Arc::clone(&client),
      ));
      
      // Store in app state
      app.manage(config);
      app.manage(connection_manager);
      
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
