use std::process;

/// Standard exit codes for different error types in non-interactive mode
/// 
/// These exit codes follow Unix conventions and provide clear categorization
/// for different types of failures that can occur during CLI execution.
pub struct ExitCodes;

impl ExitCodes {
    /// Success - operation completed successfully
    pub const SUCCESS: i32 = 0;
    
    /// Invalid arguments - user provided invalid command-line arguments
    /// Used for: empty prompts, invalid parameter values, conflicting flags
    pub const INVALID_ARGS: i32 = 1;
    
    /// Backend unreachable - cannot connect to the AI backend service
    /// Used for: network connectivity issues, invalid URLs, connection timeouts
    pub const BACKEND_UNREACHABLE: i32 = 2;
    
    /// Authentication failed - backend rejected authentication
    /// Used for: invalid API keys, permission denied by backend
    pub const AUTH_FAILED: i32 = 3;
    
    /// Model unavailable - specified model not found or cannot be loaded
    /// Used for: model not found, model loading failures
    pub const MODEL_UNAVAILABLE: i32 = 4;
    
    /// File error - file system operation failed
    /// Used for: file not found, permission denied, binary files, file too large
    pub const FILE_ERROR: i32 = 5;
    
    /// SIGINT received - user interrupted with Ctrl+C
    /// Standard Unix exit code for SIGINT (128 + 2)
    pub const SIGINT: i32 = 130;
    
    /// SIGTERM received - process terminated by system
    /// Standard Unix exit code for SIGTERM (128 + 15)
    pub const SIGTERM: i32 = 143;
}

/// Exit the program with an error code and message
/// 
/// This function provides consistent error handling by:
/// 1. Writing the error message to stderr
/// 2. Exiting with the appropriate status code
/// 
/// # Arguments
/// * `code` - The exit code to use (should be one of the ExitCodes constants)
/// * `message` - The error message to display to the user
/// 
/// # Requirements
/// * 1.3: Exit with non-zero status code on errors
/// * 7.1: Display clear error messages for backend connectivity issues
/// * 7.2: Display authentication error messages
/// * 7.3: Display model availability error messages
/// * 7.4: Display file operation error messages
/// * 7.5: Display proper exit codes for all error types
pub fn exit_with_error(code: i32, message: &str) -> ! {
    eprintln!("Error: {}", message);
    process::exit(code);
}

/// Exit the program with a success code
/// 
/// This function is used when operations complete successfully in non-interactive mode.
/// 
/// # Requirements
/// * 1.2: Exit with status code 0 on successful completion
pub fn exit_success() -> ! {
    process::exit(ExitCodes::SUCCESS);
}

/// Categorize an error and return the appropriate exit code
/// 
/// This function analyzes error messages and returns the most appropriate
/// exit code based on the error type. This helps ensure consistent exit
/// codes across the application.
/// 
/// # Arguments
/// * `error` - The error to categorize
/// 
/// # Returns
/// The appropriate exit code for the error type
/// 
/// # Requirements
/// * 7.1: Backend unreachable errors return code 2
/// * 7.2: Authentication errors return code 3
/// * 7.3: Model errors return code 4
/// * 7.4: File errors return code 5
/// * 7.5: Invalid arguments return code 1
pub fn categorize_error(error: &anyhow::Error) -> i32 {
    let error_str = error.to_string().to_lowercase();
    
    // Check for connection/network errors
    if error_str.contains("failed to connect") 
        || error_str.contains("connection refused")
        || error_str.contains("no route to host")
        || error_str.contains("network is unreachable")
        || error_str.contains("timed out")
        || error_str.contains("timeout") {
        return ExitCodes::BACKEND_UNREACHABLE;
    }
    
    // Check for file errors first (more specific than auth errors)
    if error_str.contains("file not found")
        || error_str.contains("no such file")
        || error_str.contains("cannot read file")
        || error_str.contains("cannot access file")
        || error_str.contains("binary file")
        || error_str.contains("appears to be binary")
        || error_str.contains("file too large")
        || error_str.contains("file") && error_str.contains("too large")
        || (error_str.contains("permission denied") && 
            (error_str.contains("cannot read file") || error_str.contains("cannot access file")) &&
            !error_str.contains("authentication")) {
        return ExitCodes::FILE_ERROR;
    }
    
    // Check for authentication errors
    if error_str.contains("unauthorized")
        || error_str.contains("authentication failed")
        || error_str.contains("invalid api key")
        || error_str.contains("permission denied")
        || error_str.contains("access denied") {
        return ExitCodes::AUTH_FAILED;
    }
    
    // Check for model errors
    if error_str.contains("model not found")
        || error_str.contains("model unavailable")
        || error_str.contains("model loading failed")
        || error_str.contains("unknown model") {
        return ExitCodes::MODEL_UNAVAILABLE;
    }
    
    // Check for argument validation errors
    if error_str.contains("prompt cannot be empty")
        || error_str.contains("invalid temperature")
        || error_str.contains("invalid max-tokens")
        || error_str.contains("invalid argument")
        || error_str.contains("missing required") {
        return ExitCodes::INVALID_ARGS;
    }
    
    // Default to invalid arguments for unknown errors
    ExitCodes::INVALID_ARGS
}

/// Exit with an error after categorizing it
/// 
/// This is a convenience function that combines error categorization
/// with the exit_with_error function.
/// 
/// # Arguments
/// * `error` - The error to categorize and exit with
/// 
/// # Requirements
/// * 1.3: Exit with appropriate non-zero status code
/// * 7.1, 7.2, 7.3, 7.4, 7.5: Use correct exit codes for error types
pub fn exit_with_categorized_error(error: &anyhow::Error) -> ! {
    let code = categorize_error(error);
    exit_with_error(code, &error.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{QuickCheck, TestResult};

        /// **Feature: cli-non-interactive-mode, Property 3: Error exit codes**
        /// 
        /// For any failed non-interactive operation, the system should exit with a 
        /// non-zero status code appropriate to the error type.
        /// 
        /// **Validates: Requirements 1.3, 7.1, 7.2, 7.3, 7.4, 7.5**
        fn prop_error_exit_codes(error_message: String) -> TestResult {
            // Filter out empty error messages
            if error_message.trim().is_empty() {
                return TestResult::discard();
            }

            // Filter out control characters (except newlines and tabs)
            if error_message.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
                return TestResult::discard();
            }

            // Limit error message length to reasonable size
            if error_message.len() > 1000 {
                return TestResult::discard();
            }

            // Create an error from the message
            let error = anyhow::anyhow!("{}", error_message);
            
            // Categorize the error
            let exit_code = categorize_error(&error);
            
            // Verify that the exit code is non-zero (requirement 1.3)
            if exit_code == 0 {
                return TestResult::failed();
            }
            
            // Verify that the exit code is one of our defined codes
            let valid_codes = vec![
                ExitCodes::INVALID_ARGS,
                ExitCodes::BACKEND_UNREACHABLE,
                ExitCodes::AUTH_FAILED,
                ExitCodes::MODEL_UNAVAILABLE,
                ExitCodes::FILE_ERROR,
            ];
            
            if !valid_codes.contains(&exit_code) {
                return TestResult::failed();
            }
            
            // Verify that specific error types get the correct exit codes
            let error_lower = error_message.to_lowercase();
            
            // Backend errors should get code 2 (requirement 7.1)
            if error_lower.contains("failed to connect") 
                || error_lower.contains("connection refused")
                || error_lower.contains("timed out") {
                if exit_code != ExitCodes::BACKEND_UNREACHABLE {
                    return TestResult::failed();
                }
            }
            
            // Auth errors should get code 3 (requirement 7.2)
            if error_lower.contains("unauthorized") 
                || error_lower.contains("authentication failed") {
                if exit_code != ExitCodes::AUTH_FAILED {
                    return TestResult::failed();
                }
            }
            
            // Model errors should get code 4 (requirement 7.3)
            if error_lower.contains("model not found") 
                || error_lower.contains("model unavailable") {
                if exit_code != ExitCodes::MODEL_UNAVAILABLE {
                    return TestResult::failed();
                }
            }
            
            // File errors should get code 5 (requirement 7.4)
            if error_lower.contains("file not found") 
                || error_lower.contains("cannot read file") {
                if exit_code != ExitCodes::FILE_ERROR {
                    return TestResult::failed();
                }
            }
            
            // Argument errors should get code 1 (requirement 7.5)
            if error_lower.contains("prompt cannot be empty") 
                || error_lower.contains("invalid temperature") {
                if exit_code != ExitCodes::INVALID_ARGS {
                    return TestResult::failed();
                }
            }
            
            TestResult::passed()
        }

        #[test]
        fn test_prop_error_exit_codes() {
            QuickCheck::new()
                .tests(100)
                .quickcheck(prop_error_exit_codes as fn(String) -> TestResult);
        }
    }

    #[test]
    fn test_exit_code_constants() {
        assert_eq!(ExitCodes::SUCCESS, 0);
        assert_eq!(ExitCodes::INVALID_ARGS, 1);
        assert_eq!(ExitCodes::BACKEND_UNREACHABLE, 2);
        assert_eq!(ExitCodes::AUTH_FAILED, 3);
        assert_eq!(ExitCodes::MODEL_UNAVAILABLE, 4);
        assert_eq!(ExitCodes::FILE_ERROR, 5);
        assert_eq!(ExitCodes::SIGINT, 130);
        assert_eq!(ExitCodes::SIGTERM, 143);
    }

    #[test]
    fn test_categorize_backend_errors() {
        // Connection errors
        let error = anyhow!("Failed to connect to http://localhost:11434");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        let error = anyhow!("Connection refused");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        let error = anyhow!("No route to host");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        let error = anyhow!("Network is unreachable");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        // Timeout errors
        let error = anyhow!("Request timed out");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        let error = anyhow!("Connection timeout");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
    }

    #[test]
    fn test_categorize_auth_errors() {
        let error = anyhow!("Unauthorized");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
        
        let error = anyhow!("Authentication failed");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
        
        let error = anyhow!("Invalid API key");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
        
        let error = anyhow!("Permission denied");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
        
        let error = anyhow!("Access denied");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
    }

    #[test]
    fn test_categorize_model_errors() {
        let error = anyhow!("Model not found");
        assert_eq!(categorize_error(&error), ExitCodes::MODEL_UNAVAILABLE);
        
        let error = anyhow!("Model unavailable");
        assert_eq!(categorize_error(&error), ExitCodes::MODEL_UNAVAILABLE);
        
        let error = anyhow!("Model loading failed");
        assert_eq!(categorize_error(&error), ExitCodes::MODEL_UNAVAILABLE);
        
        let error = anyhow!("Unknown model: llama3");
        assert_eq!(categorize_error(&error), ExitCodes::MODEL_UNAVAILABLE);
    }

    #[test]
    fn test_categorize_file_errors() {
        let error = anyhow!("File not found: input.txt");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
        
        let error = anyhow!("No such file or directory");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
        
        let error = anyhow!("Cannot read file: permission denied");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
        
        let error = anyhow!("Cannot access file: input.txt");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
        
        let error = anyhow!("File appears to be binary: image.png");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
        
        let error = anyhow!("File too large: 5MB exceeds limit");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
    }

    #[test]
    fn test_categorize_argument_errors() {
        let error = anyhow!("Prompt cannot be empty");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
        
        let error = anyhow!("Invalid temperature: must be between 0.0 and 2.0");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
        
        let error = anyhow!("Invalid max-tokens: must be positive integer");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
        
        let error = anyhow!("Invalid argument: --unknown-flag");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
        
        let error = anyhow!("Missing required argument");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
    }

    #[test]
    fn test_categorize_unknown_errors() {
        // Unknown errors should default to INVALID_ARGS
        let error = anyhow!("Some unknown error occurred");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
        
        let error = anyhow!("Unexpected internal error");
        assert_eq!(categorize_error(&error), ExitCodes::INVALID_ARGS);
    }

    #[test]
    fn test_case_insensitive_categorization() {
        // Test that error categorization is case-insensitive
        let error = anyhow!("CONNECTION REFUSED");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        let error = anyhow!("UNAUTHORIZED");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
        
        let error = anyhow!("MODEL NOT FOUND");
        assert_eq!(categorize_error(&error), ExitCodes::MODEL_UNAVAILABLE);
        
        let error = anyhow!("FILE NOT FOUND");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
    }

    #[test]
    fn test_error_message_substrings() {
        // Test that partial matches work correctly
        let error = anyhow!("Backend connection failed to connect to server");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        let error = anyhow!("User authentication failed due to invalid credentials");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
        
        let error = anyhow!("The specified model not found in registry");
        assert_eq!(categorize_error(&error), ExitCodes::MODEL_UNAVAILABLE);
        
        let error = anyhow!("Input file not found in current directory");
        assert_eq!(categorize_error(&error), ExitCodes::FILE_ERROR);
    }

    #[test]
    fn test_multiple_error_indicators() {
        // Test errors that could match multiple categories
        // Should match the first category found in the order checked
        
        // This should match backend error (checked first) even though it mentions "permission"
        let error = anyhow!("Failed to connect: permission denied by firewall");
        assert_eq!(categorize_error(&error), ExitCodes::BACKEND_UNREACHABLE);
        
        // This should match auth error even though it mentions "file"
        let error = anyhow!("Permission denied when accessing file due to authentication failure");
        assert_eq!(categorize_error(&error), ExitCodes::AUTH_FAILED);
    }

    #[test]
    fn test_signal_exit_codes() {
        // Test that signal exit codes follow Unix conventions
        assert_eq!(ExitCodes::SIGINT, 128 + 2);  // SIGINT is signal 2
        assert_eq!(ExitCodes::SIGTERM, 128 + 15); // SIGTERM is signal 15
    }

    #[test]
    fn test_exit_code_ranges() {
        // Test that exit codes are in expected ranges
        assert!(ExitCodes::SUCCESS == 0);
        assert!(ExitCodes::INVALID_ARGS > 0 && ExitCodes::INVALID_ARGS < 128);
        assert!(ExitCodes::BACKEND_UNREACHABLE > 0 && ExitCodes::BACKEND_UNREACHABLE < 128);
        assert!(ExitCodes::AUTH_FAILED > 0 && ExitCodes::AUTH_FAILED < 128);
        assert!(ExitCodes::MODEL_UNAVAILABLE > 0 && ExitCodes::MODEL_UNAVAILABLE < 128);
        assert!(ExitCodes::FILE_ERROR > 0 && ExitCodes::FILE_ERROR < 128);
        assert!(ExitCodes::SIGINT >= 128);
        assert!(ExitCodes::SIGTERM >= 128);
    }

    #[test]
    fn test_all_exit_codes_unique() {
        // Test that all exit codes are unique
        let codes = vec![
            ExitCodes::SUCCESS,
            ExitCodes::INVALID_ARGS,
            ExitCodes::BACKEND_UNREACHABLE,
            ExitCodes::AUTH_FAILED,
            ExitCodes::MODEL_UNAVAILABLE,
            ExitCodes::FILE_ERROR,
            ExitCodes::SIGINT,
            ExitCodes::SIGTERM,
        ];
        
        for (i, &code1) in codes.iter().enumerate() {
            for (j, &code2) in codes.iter().enumerate() {
                if i != j {
                    assert_ne!(code1, code2, "Exit codes {} and {} are not unique", code1, code2);
                }
            }
        }
    }
}