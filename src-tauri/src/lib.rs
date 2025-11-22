mod commands;
mod persona;
pub mod config;

use commands::{
    get_models, send_message_stream, get_chat_history, new_conversation,
    get_personas, set_active_persona, get_active_persona
};
use persona::PersonaManager;
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
      
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
