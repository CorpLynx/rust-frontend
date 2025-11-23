/// Commands that can be executed in the CLI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Exit the application
    Exit,
    /// Quit the application (alias for Exit)
    Quit,
    /// Clear the terminal screen
    Clear,
    /// Start a new conversation
    New,
    /// Display help information
    Help,
    /// List available models
    Models,
    /// Unknown command
    Unknown(String),
}

impl Command {
    /// Parse a command string into a Command enum
    ///
    /// Commands are case-insensitive and must start with '/'
    ///
    /// # Arguments
    /// * `input` - The input string to parse (should start with '/')
    ///
    /// # Returns
    /// A Command enum variant
    ///
    /// # Examples
    /// ```
    /// use prometheus::cli::commands::Command;
    ///
    /// assert_eq!(Command::parse("/exit"), Command::Exit);
    /// assert_eq!(Command::parse("/QUIT"), Command::Quit);
    /// assert_eq!(Command::parse("/help"), Command::Help);
    /// assert_eq!(Command::parse("/unknown"), Command::Unknown("unknown".to_string()));
    /// ```
    pub fn parse(input: &str) -> Self {
        let input = input.trim();
        
        // Remove leading '/' if present
        let command = if input.starts_with('/') {
            &input[1..]
        } else {
            input
        };

        // Convert to lowercase for case-insensitive matching
        match command.to_lowercase().as_str() {
            "exit" => Command::Exit,
            "quit" => Command::Quit,
            "clear" => Command::Clear,
            "new" => Command::New,
            "help" => Command::Help,
            "models" => Command::Models,
            _ => Command::Unknown(command.to_string()),
        }
    }

    /// Get a description of the command
    pub fn description(&self) -> &str {
        match self {
            Command::Exit => "Exit the application",
            Command::Quit => "Quit the application (alias for /exit)",
            Command::Clear => "Clear the terminal screen",
            Command::New => "Start a new conversation",
            Command::Help => "Display this help message",
            Command::Models => "List available models from the backend",
            Command::Unknown(_) => "Unknown command",
        }
    }

    /// Get the command name as a string
    pub fn name(&self) -> String {
        match self {
            Command::Exit => "exit".to_string(),
            Command::Quit => "quit".to_string(),
            Command::Clear => "clear".to_string(),
            Command::New => "new".to_string(),
            Command::Help => "help".to_string(),
            Command::Models => "models".to_string(),
            Command::Unknown(cmd) => cmd.clone(),
        }
    }
}

/// Display help information for all available commands
pub fn display_help() -> String {
    let mut help = String::from("Available commands:\n\n");
    
    let commands = vec![
        Command::Exit,
        Command::Quit,
        Command::Clear,
        Command::New,
        Command::Help,
        Command::Models,
    ];

    for cmd in commands {
        help.push_str(&format!("  /{:<10} - {}\n", cmd.name(), cmd.description()));
    }

    help
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exit_command() {
        assert_eq!(Command::parse("/exit"), Command::Exit);
        assert_eq!(Command::parse("exit"), Command::Exit);
    }

    #[test]
    fn test_parse_quit_command() {
        assert_eq!(Command::parse("/quit"), Command::Quit);
        assert_eq!(Command::parse("quit"), Command::Quit);
    }

    #[test]
    fn test_parse_clear_command() {
        assert_eq!(Command::parse("/clear"), Command::Clear);
        assert_eq!(Command::parse("clear"), Command::Clear);
    }

    #[test]
    fn test_parse_new_command() {
        assert_eq!(Command::parse("/new"), Command::New);
        assert_eq!(Command::parse("new"), Command::New);
    }

    #[test]
    fn test_parse_help_command() {
        assert_eq!(Command::parse("/help"), Command::Help);
        assert_eq!(Command::parse("help"), Command::Help);
    }

    #[test]
    fn test_parse_models_command() {
        assert_eq!(Command::parse("/models"), Command::Models);
        assert_eq!(Command::parse("models"), Command::Models);
    }

    #[test]
    fn test_parse_unknown_command() {
        assert_eq!(
            Command::parse("/unknown"),
            Command::Unknown("unknown".to_string())
        );
        assert_eq!(
            Command::parse("/foo"),
            Command::Unknown("foo".to_string())
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(Command::parse("/EXIT"), Command::Exit);
        assert_eq!(Command::parse("/Exit"), Command::Exit);
        assert_eq!(Command::parse("/QUIT"), Command::Quit);
        assert_eq!(Command::parse("/Quit"), Command::Quit);
        assert_eq!(Command::parse("/CLEAR"), Command::Clear);
        assert_eq!(Command::parse("/Clear"), Command::Clear);
        assert_eq!(Command::parse("/NEW"), Command::New);
        assert_eq!(Command::parse("/New"), Command::New);
        assert_eq!(Command::parse("/HELP"), Command::Help);
        assert_eq!(Command::parse("/Help"), Command::Help);
        assert_eq!(Command::parse("/MODELS"), Command::Models);
        assert_eq!(Command::parse("/Models"), Command::Models);
    }

    #[test]
    fn test_parse_with_whitespace() {
        assert_eq!(Command::parse("  /exit  "), Command::Exit);
        assert_eq!(Command::parse("  /quit  "), Command::Quit);
        assert_eq!(Command::parse("  /help  "), Command::Help);
    }

    #[test]
    fn test_command_description() {
        assert_eq!(Command::Exit.description(), "Exit the application");
        assert_eq!(Command::Quit.description(), "Quit the application (alias for /exit)");
        assert_eq!(Command::Clear.description(), "Clear the terminal screen");
        assert_eq!(Command::New.description(), "Start a new conversation");
        assert_eq!(Command::Help.description(), "Display this help message");
        assert_eq!(Command::Models.description(), "List available models from the backend");
        assert_eq!(Command::Unknown("test".to_string()).description(), "Unknown command");
    }

    #[test]
    fn test_command_name() {
        assert_eq!(Command::Exit.name(), "exit");
        assert_eq!(Command::Quit.name(), "quit");
        assert_eq!(Command::Clear.name(), "clear");
        assert_eq!(Command::New.name(), "new");
        assert_eq!(Command::Help.name(), "help");
        assert_eq!(Command::Models.name(), "models");
        assert_eq!(Command::Unknown("test".to_string()).name(), "test");
    }

    #[test]
    fn test_display_help() {
        let help = display_help();
        assert!(help.contains("Available commands:"));
        assert!(help.contains("/exit"));
        assert!(help.contains("/quit"));
        assert!(help.contains("/clear"));
        assert!(help.contains("/new"));
        assert!(help.contains("/help"));
        assert!(help.contains("/models"));
    }

    #[test]
    fn test_parse_empty_string() {
        assert_eq!(Command::parse(""), Command::Unknown("".to_string()));
        assert_eq!(Command::parse("/"), Command::Unknown("".to_string()));
    }

    #[test]
    fn test_parse_slash_only() {
        assert_eq!(Command::parse("/"), Command::Unknown("".to_string()));
    }

    #[test]
    fn test_unknown_command_preserves_name() {
        let cmd = Command::parse("/foobar");
        match cmd {
            Command::Unknown(name) => assert_eq!(name, "foobar"),
            _ => panic!("Expected Unknown command"),
        }
    }

    #[test]
    fn test_command_equality() {
        assert_eq!(Command::Exit, Command::Exit);
        assert_eq!(Command::Quit, Command::Quit);
        assert_ne!(Command::Exit, Command::Quit);
        assert_eq!(
            Command::Unknown("test".to_string()),
            Command::Unknown("test".to_string())
        );
        assert_ne!(
            Command::Unknown("test1".to_string()),
            Command::Unknown("test2".to_string())
        );
    }
}
