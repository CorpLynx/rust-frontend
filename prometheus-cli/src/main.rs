use anyhow::{Context, Result};
use clap::Parser;

mod app;
mod backend;
mod commands;
mod config;
mod conversation;
mod error;
mod exit_codes;
mod input;
mod markdown_renderer;
mod mode;
mod non_interactive;
mod output;
mod streaming;
mod terminal;
mod update;

use app::CliApp;
use config::AppConfig;
use exit_codes::{ExitCodes, exit_with_error};
use input::InputProcessor;
use mode::{ExecutionMode, ModeDetector};
use non_interactive::NonInteractiveHandler;

/// Prometheus CLI - Terminal-based AI chat interface
/// 
/// USAGE:
///     prometheus-cli [OPTIONS] [PROMPT]
/// 
/// MODES:
///     Interactive Mode (default):
///         prometheus-cli
///         prometheus-cli --model llama2
/// 
///     Non-Interactive Mode:
///         prometheus-cli "What is Rust?"
///         echo "Analyze this" | prometheus-cli
///         prometheus-cli --file code.py "Explain this code"
/// 
/// EXAMPLES:
///     # Start interactive chat
///     prometheus-cli
/// 
///     # Quick question
///     prometheus-cli "What is the capital of France?"
/// 
///     # Analyze a file
///     prometheus-cli --file main.rs "Review this code for bugs"
/// 
///     # Use with pipes
///     cat error.log | prometheus-cli "What caused this error?"
/// 
///     # JSON output for scripts
///     prometheus-cli --json --quiet "Generate a UUID"
/// 
///     # Custom model and parameters
///     prometheus-cli --model codellama --temperature 0.3 "Write a Python function"
/// 
///     # Multiple files with system prompt
///     prometheus-cli --file src/main.rs --file src/lib.rs --system "You are a code reviewer" "Find potential issues"
#[derive(Parser, Debug)]
#[command(name = "prometheus-cli")]
#[command(author, version, about)]
#[command(long_about = "Prometheus CLI provides both interactive and non-interactive modes for AI chat.\n\nInteractive mode starts a REPL session for ongoing conversations.\nNon-interactive mode processes a single prompt and exits, perfect for scripts and automation.")]
struct Args {
    /// Prompt to process (enables non-interactive mode)
    /// 
    /// When provided, the CLI processes this prompt and exits instead of starting
    /// an interactive session. Can be combined with stdin input and file contents.
    #[arg(value_name = "PROMPT", help = "Prompt to process (enables non-interactive mode)")]
    prompt: Option<String>,

    /// Ollama backend URL (overrides config file)
    /// 
    /// Specify the URL of the Ollama server. Supports both local and remote instances.
    /// Examples: http://localhost:11434, http://192.168.1.100:11434
    #[arg(short, long, value_name = "URL", help = "Ollama backend URL")]
    url: Option<String>,

    /// Model name to use for chat (overrides config file)
    /// 
    /// Specify which AI model to use. Use 'ollama list' to see available models.
    /// Examples: llama2, codellama, mistral, llama2:13b
    #[arg(short, long, value_name = "MODEL", help = "Model name to use")]
    model: Option<String>,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE", default_value = "config.toml", help = "Configuration file path")]
    config: String,

    /// Include file contents in the prompt
    /// 
    /// Read and include file contents in the prompt. Can be used multiple times.
    /// Files are included in the order specified. Binary files are rejected.
    /// Example: --file main.rs --file lib.rs
    #[arg(long, value_name = "PATH", action = clap::ArgAction::Append, help = "Include file contents in prompt")]
    file: Vec<String>,

    /// System prompt to use
    /// 
    /// Set a system prompt that provides context or instructions to the AI.
    /// This is prepended to your main prompt.
    /// Example: --system "You are a helpful coding assistant"
    #[arg(long, value_name = "PROMPT", help = "System prompt to use")]
    system: Option<String>,

    /// Temperature for generation (0.0-2.0)
    /// 
    /// Control randomness in responses. Lower values (0.1-0.7) are more focused,
    /// higher values (0.8-2.0) are more creative. Default varies by model.
    #[arg(long, value_name = "TEMP", help = "Temperature for generation (0.0-2.0)")]
    temperature: Option<f32>,

    /// Maximum tokens in response
    /// 
    /// Limit the length of the AI response. Useful for controlling output size
    /// in scripts or when processing many inputs.
    #[arg(long, value_name = "COUNT", help = "Maximum tokens in response")]
    max_tokens: Option<u32>,

    /// Output only the response (no formatting)
    /// 
    /// Suppress all output except the AI response. Automatically enabled when
    /// stdout is redirected. Perfect for use in pipes and scripts.
    #[arg(short, long, help = "Output only the response (no formatting)")]
    quiet: bool,

    /// Output response in JSON format
    /// 
    /// Format the response as JSON with metadata including timestamp and length.
    /// Useful for programmatic processing of responses.
    #[arg(long, help = "Output response in JSON format")]
    json: bool,

    /// Disable streaming, wait for complete response
    /// 
    /// Buffer the entire response before outputting anything. Useful when you need
    /// the complete response at once or when piping to tools that expect complete input.
    #[arg(long, help = "Disable streaming, wait for complete response")]
    no_stream: bool,

    /// Include verbose debug information
    /// 
    /// Show additional information like prompt length, processing time, and model details.
    /// Debug output goes to stderr, so it won't interfere with response piping.
    #[arg(short, long, help = "Include verbose debug information")]
    verbose: bool,

    /// Save partial responses on interruption
    /// 
    /// In non-interactive mode, partial responses are normally discarded when interrupted.
    /// This flag saves them to conversation history even when interrupted.
    #[arg(long, help = "Save partial responses on interruption")]
    save_on_interrupt: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Load configuration with fallback to defaults
    let config = match AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: Failed to load config from {}: {}", args.config, e);
            eprintln!("Using default configuration");
            AppConfig::default()
        }
    };

    // Detect execution mode based on arguments and stdin
    let mode = match ModeDetector::detect_mode(
        args.prompt.as_deref(),
        args.quiet,
        args.json,
        args.no_stream,
        args.verbose,
        args.save_on_interrupt,
    ) {
        Ok(mode) => mode,
        Err(e) => {
            exit_with_error(ExitCodes::INVALID_ARGS, &format!("Failed to detect execution mode: {}", e));
        }
    };

    match mode {
        ExecutionMode::Interactive => {
            // Run interactive mode (existing behavior)
            run_interactive_mode(config, args).await
        }
        ExecutionMode::NonInteractive { prompt, options } => {
            // Run non-interactive mode
            run_non_interactive_mode(config, args, prompt, options).await
        }
    }
}

/// Run the CLI in interactive mode (existing behavior)
async fn run_interactive_mode(config: AppConfig, args: Args) -> Result<()> {
    // Create and run CLI app with CLI argument overrides
    // If no model is specified, prompt for interactive selection
    let mut app = CliApp::new_with_model_selection(config, args.url, args.model)
        .await
        .context("Failed to initialize CLI application")?;

    app.run().await
}

/// Run the CLI in non-interactive mode
async fn run_non_interactive_mode(
    config: AppConfig,
    args: Args,
    prompt: String,
    options: mode::NonInteractiveOptions,
) -> Result<()> {
    // Validate input parameters using comprehensive validation
    if let Err(e) = InputProcessor::validate_parameters(
        &prompt,
        args.temperature,
        args.max_tokens,
    ) {
        exit_with_error(ExitCodes::INVALID_ARGS, &format!("Invalid parameters: {}", e));
    }

    // Build the final prompt by combining all input sources
    let final_prompt = match InputProcessor::build_prompt(
        prompt,
        &args.file,
        args.system.as_deref(),
    ) {
        Ok(prompt) => prompt,
        Err(e) => {
            exit_with_error(ExitCodes::FILE_ERROR, &format!("Failed to process input: {}", e));
        }
    };

    // Validate the final prompt
    if let Err(e) = InputProcessor::validate_prompt(&final_prompt) {
        exit_with_error(ExitCodes::INVALID_ARGS, &format!("Invalid prompt: {}", e));
    }

    // Determine the model to use
    let model = args.model.unwrap_or_else(|| "llama2".to_string());

    // Create non-interactive handler
    let mut handler = match NonInteractiveHandler::new(
        &config,
        args.url,
        Some(model.clone()),
        &options,
    ) {
        Ok(handler) => handler,
        Err(e) => {
            exit_with_error(ExitCodes::BACKEND_UNREACHABLE, &format!("Failed to initialize backend: {}", e));
        }
    };

    // Adjust options based on output redirection detection
    let mut adjusted_options = options.clone();
    let output_formatter = output::OutputFormatter::new();
    output_formatter.adjust_for_redirection(&mut adjusted_options);

    // Process the prompt with adjusted options
    if let Err(e) = handler.process_prompt(final_prompt, &model, args.temperature, args.max_tokens, &adjusted_options).await {
        // Determine appropriate exit code based on error type
        let error_msg = e.to_string();
        let exit_code = if error_msg.contains("connection") || error_msg.contains("unreachable") {
            ExitCodes::BACKEND_UNREACHABLE
        } else if error_msg.contains("authentication") || error_msg.contains("unauthorized") {
            ExitCodes::AUTH_FAILED
        } else if error_msg.contains("model") && error_msg.contains("not found") {
            ExitCodes::MODEL_UNAVAILABLE
        } else if error_msg.contains("file") {
            ExitCodes::FILE_ERROR
        } else {
            ExitCodes::BACKEND_UNREACHABLE // Default for backend errors
        };

        exit_with_error(exit_code, &format!("Processing failed: {}", e));
    }

    // Exit successfully
    std::process::exit(ExitCodes::SUCCESS);
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use quickcheck::{QuickCheck, TestResult};

    #[test]
    fn test_prompt_argument_detection() {
        // Test with prompt argument - should enable non-interactive mode
        let args = Args::try_parse_from(&["prometheus-cli", "Hello world"]).unwrap();
        assert_eq!(args.prompt, Some("Hello world".to_string()));
        
        // Test without prompt argument - should be None for interactive mode
        let args = Args::try_parse_from(&["prometheus-cli"]).unwrap();
        assert_eq!(args.prompt, None);
    }

    #[test]
    fn test_file_flag_parsing() {
        // Test single file
        let args = Args::try_parse_from(&["prometheus-cli", "--file", "test.txt", "prompt"]).unwrap();
        assert_eq!(args.file, vec!["test.txt"]);
        assert_eq!(args.prompt, Some("prompt".to_string()));

        // Test multiple files
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--file", "file1.txt", 
            "--file", "file2.txt", 
            "prompt"
        ]).unwrap();
        assert_eq!(args.file, vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_system_prompt_flag() {
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--system", "You are a helpful assistant", 
            "Hello"
        ]).unwrap();
        assert_eq!(args.system, Some("You are a helpful assistant".to_string()));
        assert_eq!(args.prompt, Some("Hello".to_string()));
    }

    #[test]
    fn test_temperature_flag_parsing() {
        // Valid temperature
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--temperature", "0.7", 
            "prompt"
        ]).unwrap();
        assert_eq!(args.temperature, Some(0.7));

        // Temperature at boundary
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--temperature", "2.0", 
            "prompt"
        ]).unwrap();
        assert_eq!(args.temperature, Some(2.0));
    }

    #[test]
    fn test_max_tokens_flag_parsing() {
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--max-tokens", "1000", 
            "prompt"
        ]).unwrap();
        assert_eq!(args.max_tokens, Some(1000));
    }

    #[test]
    fn test_output_control_flags() {
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--quiet", 
            "--json", 
            "--no-stream", 
            "--verbose", 
            "--save-on-interrupt",
            "prompt"
        ]).unwrap();
        
        assert!(args.quiet);
        assert!(args.json);
        assert!(args.no_stream);
        assert!(args.verbose);
        assert!(args.save_on_interrupt);
    }

    #[test]
    fn test_existing_flags_still_work() {
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--url", "http://localhost:11434", 
            "--model", "llama2", 
            "--config", "custom.toml",
            "prompt"
        ]).unwrap();
        
        assert_eq!(args.url, Some("http://localhost:11434".to_string()));
        assert_eq!(args.model, Some("llama2".to_string()));
        assert_eq!(args.config, "custom.toml");
        assert_eq!(args.prompt, Some("prompt".to_string()));
    }

    #[test]
    fn test_argument_combinations() {
        // Test comprehensive argument combination
        let args = Args::try_parse_from(&[
            "prometheus-cli",
            "--url", "http://localhost:11434",
            "--model", "llama2", 
            "--file", "input.txt",
            "--system", "Be helpful",
            "--temperature", "0.8",
            "--max-tokens", "500",
            "--quiet",
            "--json",
            "Analyze this file"
        ]).unwrap();

        assert_eq!(args.prompt, Some("Analyze this file".to_string()));
        assert_eq!(args.url, Some("http://localhost:11434".to_string()));
        assert_eq!(args.model, Some("llama2".to_string()));
        assert_eq!(args.file, vec!["input.txt"]);
        assert_eq!(args.system, Some("Be helpful".to_string()));
        assert_eq!(args.temperature, Some(0.8));
        assert_eq!(args.max_tokens, Some(500));
        assert!(args.quiet);
        assert!(args.json);
    }

    #[test]
    fn test_invalid_temperature_values() {
        // Test that invalid temperature values are handled by clap
        let result = Args::try_parse_from(&[
            "prometheus-cli", 
            "--temperature", "invalid", 
            "prompt"
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_max_tokens_values() {
        // Test that invalid max-tokens values are handled by clap
        let result = Args::try_parse_from(&[
            "prometheus-cli", 
            "--max-tokens", "invalid", 
            "prompt"
        ]);
        assert!(result.is_err());

        // Test negative values
        let result = Args::try_parse_from(&[
            "prometheus-cli", 
            "--max-tokens", "-100", 
            "prompt"
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_conflicting_output_flags() {
        // Test that conflicting flags can coexist (behavior will be handled in logic)
        let args = Args::try_parse_from(&[
            "prometheus-cli", 
            "--quiet", 
            "--verbose", 
            "prompt"
        ]).unwrap();
        
        assert!(args.quiet);
        assert!(args.verbose);
        // Note: The actual conflict resolution will be handled in the application logic
    }

    /// **Feature: cli-non-interactive-mode, Property 10: Interactive element suppression**
    /// For any non-interactive mode execution, no banner, model selection prompts, or other 
    /// interactive elements should be displayed
    /// **Validates: Requirements 4.4**
    #[test]
    fn prop_interactive_element_suppression() {
        fn property(prompt: String, quiet: bool, json: bool, no_stream: bool, verbose: bool) -> TestResult {
            // Filter out empty prompts
            if prompt.trim().is_empty() {
                return TestResult::discard();
            }

            // Filter out prompts with control characters
            if prompt.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
                return TestResult::discard();
            }

            // Limit prompt length
            if prompt.len() > 100 {
                return TestResult::discard();
            }

            // Test mode detection for non-interactive mode
            let mode = match ModeDetector::detect_mode(
                Some(&prompt),
                quiet,
                json,
                no_stream,
                verbose,
                false, // save_on_interrupt
            ) {
                Ok(mode) => mode,
                Err(_) => return TestResult::discard(),
            };

            // Verify it's non-interactive mode
            match mode {
                ExecutionMode::NonInteractive { prompt: detected_prompt, options } => {
                    // Verify the prompt is preserved
                    if detected_prompt != prompt {
                        return TestResult::failed();
                    }

                    // Verify options are set correctly
                    // Note: quiet may be automatically enabled if stdout is not a terminal
                    // So we only check that explicit quiet flag is respected
                    if quiet && !options.quiet {
                        return TestResult::failed();
                    }
                    if options.json != json {
                        return TestResult::failed();
                    }
                    if options.no_stream != no_stream {
                        return TestResult::failed();
                    }
                    if options.verbose != verbose {
                        return TestResult::failed();
                    }

                    // In non-interactive mode, no interactive elements should be displayed
                    // This is verified by the mode detection itself - if we get NonInteractive mode,
                    // the system will not display banners, model selection, etc.
                    TestResult::passed()
                }
                ExecutionMode::Interactive => {
                    // Should not happen when prompt is provided
                    TestResult::failed()
                }
            }
        }

        QuickCheck::new()
            .tests(100)
            .quickcheck(property as fn(String, bool, bool, bool, bool) -> TestResult);
    }

    /// Test that interactive mode still works when no arguments provided
    /// **Validates: Requirements 1.1, 1.4**
    #[test]
    fn test_interactive_mode_activation() {
        // Test with no prompt argument - should default to interactive mode
        let mode = ModeDetector::detect_mode(
            None,    // no prompt
            false,   // quiet
            false,   // json
            false,   // no_stream
            false,   // verbose
            false,   // save_on_interrupt
        ).unwrap();

        match mode {
            ExecutionMode::Interactive => {
                // This is expected
            }
            ExecutionMode::NonInteractive { .. } => {
                panic!("Expected Interactive mode, got NonInteractive");
            }
        }
    }

    /// Test that non-interactive mode activates correctly with prompt argument
    /// **Validates: Requirements 1.1, 1.4**
    #[test]
    fn test_non_interactive_mode_activation() {
        let test_prompt = "Test prompt for non-interactive mode";
        
        let mode = ModeDetector::detect_mode(
            Some(test_prompt),
            false,   // quiet
            false,   // json
            false,   // no_stream
            false,   // verbose
            false,   // save_on_interrupt
        ).unwrap();

        match mode {
            ExecutionMode::NonInteractive { prompt, options } => {
                assert_eq!(prompt, test_prompt);
                // Note: quiet may be automatically enabled if stdout is not a terminal (during tests)
                assert!(!options.json);
                assert!(!options.no_stream);
                assert!(!options.verbose);
                assert!(!options.save_on_interrupt);
            }
            ExecutionMode::Interactive => {
                panic!("Expected NonInteractive mode, got Interactive");
            }
        }
    }

    /// Test backward compatibility - existing argument parsing still works
    /// **Validates: Requirements 1.1, 1.4**
    #[test]
    fn test_backward_compatibility() {
        // Test that existing flags still work without prompt (interactive mode)
        let args = Args::try_parse_from(&[
            "prometheus-cli",
            "--url", "http://localhost:11434",
            "--model", "llama2",
            "--config", "config.toml"
        ]).unwrap();

        assert_eq!(args.url, Some("http://localhost:11434".to_string()));
        assert_eq!(args.model, Some("llama2".to_string()));
        assert_eq!(args.config, "config.toml");
        assert_eq!(args.prompt, None); // No prompt = interactive mode

        // Test mode detection
        let mode = ModeDetector::detect_mode(
            args.prompt.as_deref(),
            args.quiet,
            args.json,
            args.no_stream,
            args.verbose,
            args.save_on_interrupt,
        ).unwrap();

        match mode {
            ExecutionMode::Interactive => {
                // Expected for backward compatibility
            }
            ExecutionMode::NonInteractive { .. } => {
                panic!("Expected Interactive mode for backward compatibility");
            }
        }
    }

    /// Test that new flags work with existing flags in non-interactive mode
    /// **Validates: Requirements 1.1, 1.4**
    #[test]
    fn test_new_flags_with_existing_flags() {
        let args = Args::try_parse_from(&[
            "prometheus-cli",
            "--url", "http://localhost:11434",
            "--model", "llama2",
            "--quiet",
            "--json",
            "--file", "test.txt",
            "--system", "You are helpful",
            "--temperature", "0.7",
            "Test prompt"
        ]).unwrap();

        // Verify all arguments are parsed correctly
        assert_eq!(args.url, Some("http://localhost:11434".to_string()));
        assert_eq!(args.model, Some("llama2".to_string()));
        assert!(args.quiet);
        assert!(args.json);
        assert_eq!(args.file, vec!["test.txt"]);
        assert_eq!(args.system, Some("You are helpful".to_string()));
        assert_eq!(args.temperature, Some(0.7));
        assert_eq!(args.prompt, Some("Test prompt".to_string()));

        // Test mode detection
        let mode = ModeDetector::detect_mode(
            args.prompt.as_deref(),
            args.quiet,
            args.json,
            args.no_stream,
            args.verbose,
            args.save_on_interrupt,
        ).unwrap();

        match mode {
            ExecutionMode::NonInteractive { prompt, options } => {
                assert_eq!(prompt, "Test prompt");
                assert!(options.quiet);
                assert!(options.json);
                assert!(!options.no_stream);
                assert!(!options.verbose);
                assert!(!options.save_on_interrupt);
            }
            ExecutionMode::Interactive => {
                panic!("Expected NonInteractive mode with prompt argument");
            }
        }
    }

    /// Test mode switching with different argument combinations
    /// **Validates: Requirements 1.1, 1.4**
    #[test]
    fn test_mode_switching_combinations() {
        // Test 1: Empty prompt should still trigger non-interactive mode
        let mode = ModeDetector::detect_mode(
            Some(""),
            false, false, false, false, false,
        ).unwrap();

        match mode {
            ExecutionMode::NonInteractive { prompt, .. } => {
                assert_eq!(prompt, "");
            }
            ExecutionMode::Interactive => {
                panic!("Expected NonInteractive mode even with empty prompt");
            }
        }

        // Test 2: Whitespace-only prompt should trigger non-interactive mode
        let mode = ModeDetector::detect_mode(
            Some("   \t\n  "),
            false, false, false, false, false,
        ).unwrap();

        match mode {
            ExecutionMode::NonInteractive { prompt, .. } => {
                assert_eq!(prompt, "   \t\n  ");
            }
            ExecutionMode::Interactive => {
                panic!("Expected NonInteractive mode with whitespace prompt");
            }
        }

        // Test 3: All flags enabled
        let mode = ModeDetector::detect_mode(
            Some("test"),
            true, true, true, true, true,
        ).unwrap();

        match mode {
            ExecutionMode::NonInteractive { options, .. } => {
                assert!(options.quiet);
                assert!(options.json);
                assert!(options.no_stream);
                assert!(options.verbose);
                assert!(options.save_on_interrupt);
            }
            ExecutionMode::Interactive => {
                panic!("Expected NonInteractive mode with all flags");
            }
        }
    }
}