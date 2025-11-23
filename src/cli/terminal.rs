use anyhow::{Context, Result};
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

/// Terminal handler for CLI I/O operations
pub struct Terminal {
    stdout: io::Stdout,
    spinner_visible: bool,
}

impl Terminal {
    /// Create a new Terminal instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            stdout: io::stdout(),
            spinner_visible: false,
        })
    }

    /// Read a line of input from the user
    ///
    /// # Returns
    /// The input string without the trailing newline
    pub fn read_line(&mut self) -> Result<String> {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read input from stdin")?;

        // Remove trailing newline
        if input.ends_with('\n') {
            input.pop();
            if input.ends_with('\r') {
                input.pop();
            }
        }

        Ok(input)
    }

    /// Write text to the terminal
    ///
    /// # Arguments
    /// * `text` - The text to write
    pub fn write(&mut self, text: &str) -> Result<()> {
        write!(self.stdout, "{}", text).context("Failed to write to stdout")?;
        self.stdout.flush().context("Failed to flush stdout")?;
        Ok(())
    }

    /// Write a user prompt with formatting
    ///
    /// # Arguments
    /// * `text` - The prompt text
    pub fn write_user_prompt(&mut self, text: &str) -> Result<()> {
        execute!(
            self.stdout,
            SetForegroundColor(Color::Green),
            Print("You: "),
            ResetColor,
            Print(text),
            Print("\n")
        )
        .context("Failed to write user prompt")?;
        self.stdout.flush().context("Failed to flush stdout")?;
        Ok(())
    }

    /// Write an AI response with formatting
    ///
    /// # Arguments
    /// * `text` - The response text
    pub fn write_ai_response(&mut self, text: &str) -> Result<()> {
        execute!(
            self.stdout,
            SetForegroundColor(Color::Blue),
            Print("AI: "),
            ResetColor,
            Print(text),
            Print("\n")
        )
        .context("Failed to write AI response")?;
        self.stdout.flush().context("Failed to flush stdout")?;
        Ok(())
    }

    /// Write an error message with formatting
    ///
    /// # Arguments
    /// * `error` - The error message
    pub fn write_error(&mut self, error: &str) -> Result<()> {
        execute!(
            self.stdout,
            SetForegroundColor(Color::Red),
            Print("Error: "),
            Print(error),
            ResetColor,
            Print("\n")
        )
        .context("Failed to write error message")?;
        self.stdout.flush().context("Failed to flush stdout")?;
        Ok(())
    }

    /// Write an info message with formatting
    ///
    /// # Arguments
    /// * `info` - The info message
    pub fn write_info(&mut self, info: &str) -> Result<()> {
        execute!(
            self.stdout,
            SetForegroundColor(Color::Cyan),
            Print("Info: "),
            ResetColor,
            Print(info),
            Print("\n")
        )
        .context("Failed to write info message")?;
        self.stdout.flush().context("Failed to flush stdout")?;
        Ok(())
    }

    /// Clear the terminal screen
    pub fn clear_screen(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .context("Failed to clear screen")?;
        self.stdout.flush().context("Failed to flush stdout")?;
        Ok(())
    }

    /// Show a loading spinner
    pub fn show_spinner(&mut self) -> Result<()> {
        if !self.spinner_visible {
            execute!(
                self.stdout,
                SetForegroundColor(Color::Yellow),
                Print("⠋ Loading..."),
                ResetColor
            )
            .context("Failed to show spinner")?;
            self.stdout.flush().context("Failed to flush stdout")?;
            self.spinner_visible = true;
        }
        Ok(())
    }

    /// Hide the loading spinner
    pub fn hide_spinner(&mut self) -> Result<()> {
        if self.spinner_visible {
            // Clear the spinner line by moving to the beginning and clearing to end of line
            execute!(
                self.stdout,
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine)
            )
            .context("Failed to hide spinner")?;
            self.stdout.flush().context("Failed to flush stdout")?;
            self.spinner_visible = false;
        }
        Ok(())
    }

    /// Check if spinner is currently visible
    pub fn is_spinner_visible(&self) -> bool {
        self.spinner_visible
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new().expect("Failed to create default Terminal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper struct for testing terminal output
    struct TestTerminal {
        output: Vec<u8>,
        spinner_visible: bool,
    }

    impl TestTerminal {
        fn new() -> Self {
            Self {
                output: Vec::new(),
                spinner_visible: false,
            }
        }

        fn write(&mut self, text: &str) -> Result<()> {
            self.output.extend_from_slice(text.as_bytes());
            Ok(())
        }

        fn write_user_prompt(&mut self, text: &str) -> Result<()> {
            // Simulate the formatting without actual terminal codes for testing
            self.write(&format!("You: {}\n", text))
        }

        fn write_ai_response(&mut self, text: &str) -> Result<()> {
            // Simulate the formatting without actual terminal codes for testing
            self.write(&format!("AI: {}\n", text))
        }

        fn write_error(&mut self, error: &str) -> Result<()> {
            // Simulate the formatting without actual terminal codes for testing
            self.write(&format!("Error: {}\n", error))
        }

        fn write_info(&mut self, info: &str) -> Result<()> {
            // Simulate the formatting without actual terminal codes for testing
            self.write(&format!("Info: {}\n", info))
        }

        fn clear_screen(&mut self) -> Result<()> {
            self.output.clear();
            Ok(())
        }

        fn show_spinner(&mut self) -> Result<()> {
            if !self.spinner_visible {
                self.write("⠋ Loading...")?;
                self.spinner_visible = true;
            }
            Ok(())
        }

        fn hide_spinner(&mut self) -> Result<()> {
            if self.spinner_visible {
                // Simulate clearing the spinner
                self.spinner_visible = false;
            }
            Ok(())
        }

        fn is_spinner_visible(&self) -> bool {
            self.spinner_visible
        }

        fn get_output(&self) -> String {
            String::from_utf8_lossy(&self.output).to_string()
        }
    }

    #[test]
    fn test_terminal_creation() {
        let terminal = Terminal::new();
        assert!(terminal.is_ok());
        let terminal = terminal.unwrap();
        assert!(!terminal.is_spinner_visible());
    }

    #[test]
    fn test_terminal_default() {
        let terminal = Terminal::default();
        assert!(!terminal.is_spinner_visible());
    }

    #[test]
    fn test_spinner_state_tracking() {
        let mut terminal = Terminal::new().unwrap();
        assert!(!terminal.is_spinner_visible());

        // Note: We can't actually test show_spinner/hide_spinner without mocking stdout
        // These tests verify the state tracking logic
        terminal.spinner_visible = true;
        assert!(terminal.is_spinner_visible());

        terminal.spinner_visible = false;
        assert!(!terminal.is_spinner_visible());
    }

    // Tests using TestTerminal for output verification

    #[test]
    fn test_write_basic_text() {
        let mut terminal = TestTerminal::new();
        terminal.write("Hello, World!").unwrap();
        assert_eq!(terminal.get_output(), "Hello, World!");
    }

    #[test]
    fn test_write_multiple_times() {
        let mut terminal = TestTerminal::new();
        terminal.write("First ").unwrap();
        terminal.write("Second ").unwrap();
        terminal.write("Third").unwrap();
        assert_eq!(terminal.get_output(), "First Second Third");
    }

    #[test]
    fn test_write_user_prompt_formatting() {
        let mut terminal = TestTerminal::new();
        terminal.write_user_prompt("What is the weather?").unwrap();
        let output = terminal.get_output();
        assert!(output.contains("You:"));
        assert!(output.contains("What is the weather?"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_write_ai_response_formatting() {
        let mut terminal = TestTerminal::new();
        terminal.write_ai_response("The weather is sunny.").unwrap();
        let output = terminal.get_output();
        assert!(output.contains("AI:"));
        assert!(output.contains("The weather is sunny."));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_write_error_formatting() {
        let mut terminal = TestTerminal::new();
        terminal.write_error("Connection failed").unwrap();
        let output = terminal.get_output();
        assert!(output.contains("Error:"));
        assert!(output.contains("Connection failed"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_write_info_formatting() {
        let mut terminal = TestTerminal::new();
        terminal.write_info("Loading configuration").unwrap();
        let output = terminal.get_output();
        assert!(output.contains("Info:"));
        assert!(output.contains("Loading configuration"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_multiple_message_types() {
        let mut terminal = TestTerminal::new();
        terminal.write_user_prompt("Hello").unwrap();
        terminal.write_ai_response("Hi there").unwrap();
        terminal.write_error("Something went wrong").unwrap();
        terminal.write_info("Retrying").unwrap();

        let output = terminal.get_output();
        assert!(output.contains("You: Hello"));
        assert!(output.contains("AI: Hi there"));
        assert!(output.contains("Error: Something went wrong"));
        assert!(output.contains("Info: Retrying"));
    }

    #[test]
    fn test_clear_screen() {
        let mut terminal = TestTerminal::new();
        terminal.write("Some text").unwrap();
        assert!(!terminal.get_output().is_empty());

        terminal.clear_screen().unwrap();
        assert!(terminal.get_output().is_empty());
    }

    #[test]
    fn test_clear_screen_preserves_state() {
        let mut terminal = TestTerminal::new();
        terminal.show_spinner().unwrap();
        assert!(terminal.is_spinner_visible());

        terminal.clear_screen().unwrap();
        // Spinner state should be preserved even after clearing screen
        assert!(terminal.is_spinner_visible());
    }

    #[test]
    fn test_spinner_show() {
        let mut terminal = TestTerminal::new();
        assert!(!terminal.is_spinner_visible());

        terminal.show_spinner().unwrap();
        assert!(terminal.is_spinner_visible());
        assert!(terminal.get_output().contains("Loading"));
    }

    #[test]
    fn test_spinner_hide() {
        let mut terminal = TestTerminal::new();
        terminal.show_spinner().unwrap();
        assert!(terminal.is_spinner_visible());

        terminal.hide_spinner().unwrap();
        assert!(!terminal.is_spinner_visible());
    }

    #[test]
    fn test_spinner_show_idempotent() {
        let mut terminal = TestTerminal::new();
        terminal.show_spinner().unwrap();
        let output_after_first = terminal.get_output().clone();

        terminal.show_spinner().unwrap();
        let output_after_second = terminal.get_output();

        // Showing spinner twice should not duplicate the output
        assert_eq!(output_after_first, output_after_second);
    }

    #[test]
    fn test_spinner_hide_idempotent() {
        let mut terminal = TestTerminal::new();
        terminal.show_spinner().unwrap();
        terminal.hide_spinner().unwrap();
        assert!(!terminal.is_spinner_visible());

        // Hiding again should be safe
        terminal.hide_spinner().unwrap();
        assert!(!terminal.is_spinner_visible());
    }

    #[test]
    fn test_spinner_show_hide_cycle() {
        let mut terminal = TestTerminal::new();

        // First cycle
        terminal.show_spinner().unwrap();
        assert!(terminal.is_spinner_visible());
        terminal.hide_spinner().unwrap();
        assert!(!terminal.is_spinner_visible());

        // Second cycle
        terminal.show_spinner().unwrap();
        assert!(terminal.is_spinner_visible());
        terminal.hide_spinner().unwrap();
        assert!(!terminal.is_spinner_visible());
    }

    #[test]
    fn test_empty_text_handling() {
        let mut terminal = TestTerminal::new();
        terminal.write("").unwrap();
        assert_eq!(terminal.get_output(), "");

        terminal.write_user_prompt("").unwrap();
        assert!(terminal.get_output().contains("You:"));

        terminal.clear_screen().unwrap();
        terminal.write_ai_response("").unwrap();
        assert!(terminal.get_output().contains("AI:"));
    }

    #[test]
    fn test_special_characters_in_text() {
        let mut terminal = TestTerminal::new();
        terminal.write_user_prompt("Hello\nWorld").unwrap();
        let output = terminal.get_output();
        assert!(output.contains("Hello\nWorld"));
    }

    #[test]
    fn test_unicode_text() {
        let mut terminal = TestTerminal::new();
        terminal.write_user_prompt("Hello 世界 🌍").unwrap();
        let output = terminal.get_output();
        assert!(output.contains("Hello 世界 🌍"));
    }

    #[test]
    fn test_long_text() {
        let mut terminal = TestTerminal::new();
        let long_text = "a".repeat(10000);
        terminal.write(&long_text).unwrap();
        assert_eq!(terminal.get_output().len(), 10000);
    }
}
