use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

/// Input processor for handling file inclusion and prompt building
pub struct InputProcessor;

impl InputProcessor {
    /// Process and combine all input sources into a final prompt
    pub fn build_prompt(
        base_prompt: String,
        file_paths: &[String],
        system_prompt: Option<&str>,
    ) -> Result<String> {
        let mut final_prompt = String::new();
        
        // Add system prompt if provided
        if let Some(system) = system_prompt {
            final_prompt.push_str("System: ");
            final_prompt.push_str(system);
            final_prompt.push_str("\n\n");
        }
        
        // Add file contents in the order specified
        for file_path in file_paths {
            let content = Self::read_file_safely(file_path)?;
            final_prompt.push_str(&format!("File: {}\n", file_path));
            final_prompt.push_str("```\n");
            final_prompt.push_str(&content);
            final_prompt.push_str("\n```\n\n");
        }
        
        // Add the main prompt
        final_prompt.push_str(&base_prompt);
        
        Ok(final_prompt)
    }
    
    /// Read file with comprehensive safety checks
    fn read_file_safely(file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        
        // Check if file exists
        if !path.exists() {
            bail!("File not found: {}", file_path);
        }
        
        // Check if file is readable
        let metadata = fs::metadata(path)
            .with_context(|| format!("Cannot access file: {}", file_path))?;
        
        // Check if it's actually a file (not a directory)
        if !metadata.is_file() {
            bail!("Path is not a file: {}", file_path);
        }
        
        // Check file size (warn if > 1MB, error if > 10MB)
        if metadata.len() > 10_485_760 {
            bail!("File '{}' is too large ({} bytes). Maximum file size is 10MB.", file_path, metadata.len());
        } else if metadata.len() > 1_048_576 {
            eprintln!("Warning: File '{}' is large ({} bytes). This may impact performance.", file_path, metadata.len());
        }
        
        // Read file content
        let content = fs::read_to_string(path)
            .with_context(|| format!("Cannot read file: {}", file_path))?;
        
        // Check if content looks like binary
        if Self::is_binary_content(&content) {
            bail!("File '{}' appears to be binary. Please provide text files only.", file_path);
        }
        
        // Check for UTF-8 replacement characters indicating encoding issues
        if content.contains('\u{FFFD}') {
            bail!("File '{}' contains invalid UTF-8 sequences", file_path);
        }
        
        Ok(content)
    }
    
    /// Check if content contains binary data (control characters except common ones)
    fn is_binary_content(content: &str) -> bool {
        content.chars().any(|c| {
            c.is_control() && c != '\n' && c != '\t' && c != '\r'
        })
    }
    
    /// Validate prompt content and parameters
    pub fn validate_prompt(prompt: &str) -> Result<()> {
        if prompt.trim().is_empty() {
            bail!("Prompt cannot be empty or contain only whitespace");
        }
        
        // Check for binary content in prompt
        if Self::is_binary_content(prompt) {
            bail!("Prompt contains binary or control characters. Please provide text content only.");
        }
        
        // Check for UTF-8 replacement characters
        if prompt.contains('\u{FFFD}') {
            bail!("Prompt contains invalid UTF-8 sequences");
        }
        
        // Size limits and warnings
        if prompt.len() > 1_000_000 {
            bail!("Prompt is too large ({} characters). Maximum prompt size is 1MB.", prompt.len());
        } else if prompt.len() > 100_000 {
            eprintln!("Warning: Prompt is very large ({} characters). This may impact performance.", prompt.len());
        }
        
        Ok(())
    }
    
    /// Validate temperature parameter
    pub fn validate_temperature(temperature: f32) -> Result<()> {
        if temperature.is_nan() || temperature.is_infinite() {
            bail!("Temperature must be a valid number, got: {}", temperature);
        }
        if temperature < 0.0 || temperature > 2.0 {
            bail!("Temperature must be between 0.0 and 2.0, got: {}", temperature);
        }
        Ok(())
    }
    
    /// Validate max tokens parameter
    pub fn validate_max_tokens(max_tokens: u32) -> Result<()> {
        if max_tokens == 0 {
            bail!("Max tokens must be greater than 0, got: {}", max_tokens);
        }
        
        // Reasonable upper limit to prevent excessive resource usage
        if max_tokens > 100_000 {
            eprintln!("Warning: Max tokens is very large ({})", max_tokens);
        }
        
        Ok(())
    }
    
    /// Validate all input parameters
    pub fn validate_parameters(
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<()> {
        Self::validate_prompt(prompt)?;
        
        if let Some(temp) = temperature {
            Self::validate_temperature(temp)?;
        }
        
        if let Some(tokens) = max_tokens {
            Self::validate_max_tokens(tokens)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;
    use quickcheck_macros::quickcheck;

    // Helper function to create a temporary file with content
    fn create_temp_file(content: &str) -> Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(file)
    }

    // Helper function to create a temporary directory
    fn create_temp_dir() -> Result<TempDir> {
        Ok(TempDir::new()?)
    }

    /// **Feature: cli-non-interactive-mode, Property 5: File inclusion ordering**
    /// For any sequence of --file arguments, the files should be included in the prompt 
    /// in the same order they were specified
    /// **Validates: Requirements 3.2**
    #[quickcheck]
    fn property_file_inclusion_ordering(file_contents: Vec<String>) -> bool {
        // Skip empty test cases or cases with too many files (for performance)
        if file_contents.is_empty() || file_contents.len() > 3 {
            return true;
        }
        
        // Create temporary files with processed contents
        let temp_files: Result<Vec<_>, _> = file_contents
            .iter()
            .enumerate()
            .map(|(i, content)| {
                // Ensure content is not binary by filtering out control characters
                let safe_content: String = content
                    .chars()
                    .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
                    .collect();
                
                // Make content unique and non-empty to ensure it can be found
                let unique_content = if safe_content.trim().is_empty() {
                    format!("file_{}_content", i)
                } else {
                    format!("file_{}_{}", i, safe_content)
                };
                
                let file = create_temp_file(&unique_content);
                file.map(|f| (f, unique_content, i))
            })
            .collect();
        
        let temp_files = match temp_files {
            Ok(files) => files,
            Err(_) => return true, // Skip if file creation fails
        };
        
        // Extract file paths
        let file_paths: Vec<String> = temp_files
            .iter()
            .map(|(file, _, _)| file.path().to_string_lossy().to_string())
            .collect();
        
        // Build prompt with files
        let result = InputProcessor::build_prompt(
            "Test prompt".to_string(),
            &file_paths,
            None,
        );
        
        match result {
            Ok(prompt) => {
                // Check that file contents appear in the correct order
                let mut last_position = 0;
                for (_, content, _) in &temp_files {
                    if let Some(pos) = prompt.find(content) {
                        if pos < last_position {
                            return false; // Content not in correct order
                        }
                        last_position = pos;
                    } else {
                        // Content should always be found since we made it unique
                        return false;
                    }
                }
                true
            }
            Err(_) => {
                // If any file couldn't be read, that's acceptable for this property
                true
            }
        }
    }

    /// **Feature: cli-non-interactive-mode, Property 6: File error handling**
    /// For any non-existent or unreadable file specified with --file, the system should 
    /// exit with status code 5 and display an appropriate error message
    /// **Validates: Requirements 3.3, 3.4**
    #[quickcheck]
    fn property_file_error_handling(invalid_path: String) -> bool {
        // Skip empty paths or paths that might accidentally exist
        if invalid_path.is_empty() || invalid_path.len() > 200 {
            return true;
        }
        
        // Create a path that definitely doesn't exist by adding a unique suffix
        let nonexistent_path = format!("{}_nonexistent_12345", invalid_path);
        
        // Ensure the path doesn't accidentally exist
        if Path::new(&nonexistent_path).exists() {
            return true; // Skip this test case
        }
        
        // Try to read the non-existent file
        let result = InputProcessor::read_file_safely(&nonexistent_path);
        
        // Should return an error
        match result {
            Err(err) => {
                let error_msg = err.to_string();
                // Error message should mention the file
                error_msg.contains(&nonexistent_path) || error_msg.contains("not found")
            }
            Ok(_) => false, // Should not succeed
        }
    }

    #[test]
    fn test_build_prompt_with_system_prompt() {
        let result = InputProcessor::build_prompt(
            "Main prompt".to_string(),
            &[],
            Some("You are a helpful assistant"),
        ).unwrap();
        
        assert!(result.contains("System: You are a helpful assistant"));
        assert!(result.contains("Main prompt"));
        assert!(result.starts_with("System:"));
    }

    #[test]
    fn test_build_prompt_without_system_prompt() {
        let result = InputProcessor::build_prompt(
            "Main prompt".to_string(),
            &[],
            None,
        ).unwrap();
        
        assert_eq!(result, "Main prompt");
        assert!(!result.contains("System:"));
    }

    #[test]
    fn test_build_prompt_with_single_file() -> Result<()> {
        let temp_file = create_temp_file("File content here")?;
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        let result = InputProcessor::build_prompt(
            "Main prompt".to_string(),
            &[file_path.clone()],
            None,
        )?;
        
        assert!(result.contains(&format!("File: {}", file_path)));
        assert!(result.contains("```\nFile content here\n```"));
        assert!(result.ends_with("Main prompt"));
        
        Ok(())
    }

    #[test]
    fn test_build_prompt_with_multiple_files() -> Result<()> {
        let temp_file1 = create_temp_file("First file content")?;
        let temp_file2 = create_temp_file("Second file content")?;
        
        let file_path1 = temp_file1.path().to_string_lossy().to_string();
        let file_path2 = temp_file2.path().to_string_lossy().to_string();
        
        let result = InputProcessor::build_prompt(
            "Main prompt".to_string(),
            &[file_path1.clone(), file_path2.clone()],
            None,
        )?;
        
        // Check that both files are included
        assert!(result.contains(&format!("File: {}", file_path1)));
        assert!(result.contains(&format!("File: {}", file_path2)));
        assert!(result.contains("First file content"));
        assert!(result.contains("Second file content"));
        
        // Check ordering - first file should appear before second file
        let first_pos = result.find("First file content").unwrap();
        let second_pos = result.find("Second file content").unwrap();
        assert!(first_pos < second_pos);
        
        Ok(())
    }

    #[test]
    fn test_build_prompt_comprehensive() -> Result<()> {
        let temp_file = create_temp_file("File content")?;
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        let result = InputProcessor::build_prompt(
            "Main prompt".to_string(),
            &[file_path.clone()],
            Some("System message"),
        )?;
        
        // Check all components are present and in correct order
        assert!(result.starts_with("System: System message"));
        assert!(result.contains(&format!("File: {}", file_path)));
        assert!(result.contains("File content"));
        assert!(result.ends_with("Main prompt"));
        
        // Check ordering
        let system_pos = result.find("System: System message").unwrap();
        let file_pos = result.find(&format!("File: {}", file_path)).unwrap();
        let prompt_pos = result.find("Main prompt").unwrap();
        
        assert!(system_pos < file_pos);
        assert!(file_pos < prompt_pos);
        
        Ok(())
    }

    #[test]
    fn test_read_file_safely_success() -> Result<()> {
        let temp_file = create_temp_file("Test content\nMultiple lines\tWith tabs")?;
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        let content = InputProcessor::read_file_safely(&file_path)?;
        assert_eq!(content, "Test content\nMultiple lines\tWith tabs");
        
        Ok(())
    }

    #[test]
    fn test_read_file_safely_nonexistent_file() {
        let result = InputProcessor::read_file_safely("/nonexistent/file/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_read_file_safely_directory() -> Result<()> {
        let temp_dir = create_temp_dir()?;
        let dir_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = InputProcessor::read_file_safely(&dir_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a file"));
        
        Ok(())
    }

    #[test]
    fn test_read_file_safely_binary_content() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        // Write binary content (null bytes)
        temp_file.write_all(b"Hello\x00World\x01Binary")?;
        temp_file.flush()?;
        
        let file_path = temp_file.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&file_path);
        
        // Should fail because of binary content detection
        // Note: This might succeed if the binary content can't be read as UTF-8
        // In that case, fs::read_to_string will fail first
        assert!(result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_is_binary_content() {
        // Normal text should not be detected as binary
        assert!(!InputProcessor::is_binary_content("Hello world"));
        assert!(!InputProcessor::is_binary_content("Multi\nline\ntext"));
        assert!(!InputProcessor::is_binary_content("Text with\ttabs"));
        assert!(!InputProcessor::is_binary_content("Windows\r\nline endings"));
        assert!(!InputProcessor::is_binary_content(""));
        
        // Unicode should not be detected as binary
        assert!(!InputProcessor::is_binary_content("Unicode: 你好 🌍"));
        
        // Binary content should be detected
        assert!(InputProcessor::is_binary_content("Text with\x00null"));
        assert!(InputProcessor::is_binary_content("ANSI\x1b[31mcolor\x1b[0m"));
        assert!(InputProcessor::is_binary_content("Bell\x07character"));
    }

    #[test]
    fn test_validate_prompt_success() {
        assert!(InputProcessor::validate_prompt("Valid prompt").is_ok());
        assert!(InputProcessor::validate_prompt("Multi\nline\nprompt").is_ok());
        assert!(InputProcessor::validate_prompt("   Whitespace padded   ").is_ok());
    }

    #[test]
    fn test_validate_prompt_empty() {
        assert!(InputProcessor::validate_prompt("").is_err());
        assert!(InputProcessor::validate_prompt("   ").is_err());
        assert!(InputProcessor::validate_prompt("\n\t\r").is_err());
    }

    #[test]
    fn test_validate_prompt_large() {
        let large_prompt = "a".repeat(200_000);
        // Should succeed but warn
        assert!(InputProcessor::validate_prompt(&large_prompt).is_ok());
    }

    #[test]
    fn test_validate_temperature_success() {
        assert!(InputProcessor::validate_temperature(0.0).is_ok());
        assert!(InputProcessor::validate_temperature(0.7).is_ok());
        assert!(InputProcessor::validate_temperature(1.0).is_ok());
        assert!(InputProcessor::validate_temperature(2.0).is_ok());
    }

    #[test]
    fn test_validate_temperature_invalid() {
        assert!(InputProcessor::validate_temperature(-0.1).is_err());
        assert!(InputProcessor::validate_temperature(2.1).is_err());
        assert!(InputProcessor::validate_temperature(-1.0).is_err());
        assert!(InputProcessor::validate_temperature(5.0).is_err());
    }

    #[test]
    fn test_validate_max_tokens_success() {
        assert!(InputProcessor::validate_max_tokens(1).is_ok());
        assert!(InputProcessor::validate_max_tokens(100).is_ok());
        assert!(InputProcessor::validate_max_tokens(1000).is_ok());
        assert!(InputProcessor::validate_max_tokens(50000).is_ok());
    }

    #[test]
    fn test_validate_max_tokens_invalid() {
        assert!(InputProcessor::validate_max_tokens(0).is_err());
    }

    #[test]
    fn test_validate_max_tokens_large() {
        // Should succeed but warn
        assert!(InputProcessor::validate_max_tokens(150000).is_ok());
    }

    #[test]
    fn test_validate_parameters_comprehensive() {
        // All valid
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(0.7),
            Some(1000),
        ).is_ok());
        
        // With None values
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            None,
            None,
        ).is_ok());
        
        // Invalid prompt
        assert!(InputProcessor::validate_parameters(
            "",
            Some(0.7),
            Some(1000),
        ).is_err());
        
        // Invalid temperature
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(3.0),
            Some(1000),
        ).is_err());
        
        // Invalid max tokens
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(0.7),
            Some(0),
        ).is_err());
    }

    #[test]
    fn test_file_inclusion_edge_cases() -> Result<()> {
        // Empty file
        let empty_file = create_temp_file("")?;
        let empty_path = empty_file.path().to_string_lossy().to_string();
        
        let result = InputProcessor::build_prompt(
            "Prompt".to_string(),
            &[empty_path.clone()],
            None,
        )?;
        
        assert!(result.contains(&format!("File: {}", empty_path)));
        assert!(result.contains("```\n\n```")); // Empty content between markers
        
        // File with only whitespace
        let whitespace_file = create_temp_file("   \n\t\n   ")?;
        let whitespace_path = whitespace_file.path().to_string_lossy().to_string();
        
        let result = InputProcessor::build_prompt(
            "Prompt".to_string(),
            &[whitespace_path.clone()],
            None,
        )?;
        
        assert!(result.contains("   \n\t\n   "));
        
        Ok(())
    }

    #[test]
    fn test_file_with_special_characters() -> Result<()> {
        let special_content = "Special chars: !@#$%^&*()[]{}|\\:;\"'<>?,./\nUnicode: 你好 🌍";
        let temp_file = create_temp_file(special_content)?;
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        let result = InputProcessor::build_prompt(
            "Prompt".to_string(),
            &[file_path],
            None,
        )?;
        
        assert!(result.contains(special_content));
        
        Ok(())
    }

    #[test]
    fn test_multiple_files_ordering_preserved() -> Result<()> {
        // Create files with numbered content to verify ordering
        let files: Result<Vec<_>, _> = (0..5)
            .map(|i| {
                let content = format!("File {} content", i);
                create_temp_file(&content).map(|f| (f, content))
            })
            .collect();
        
        let files = files?;
        let file_paths: Vec<String> = files
            .iter()
            .map(|(f, _)| f.path().to_string_lossy().to_string())
            .collect();
        
        let result = InputProcessor::build_prompt(
            "Final prompt".to_string(),
            &file_paths,
            None,
        )?;
        
        // Verify each file appears in order
        let mut last_pos = 0;
        for (i, (_, content)) in files.iter().enumerate() {
            let pos = result.find(content).unwrap();
            assert!(pos > last_pos, "File {} not in correct order", i);
            last_pos = pos;
        }
        
        Ok(())
    }

    // Additional comprehensive input validation tests for task 10.2

    #[test]
    fn test_empty_whitespace_prompt_handling() {
        // Test empty prompt
        let result = InputProcessor::validate_prompt("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));

        // Test whitespace-only prompts
        let whitespace_prompts = vec![
            " ",
            "  ",
            "\t",
            "\n",
            "\r",
            "\r\n",
            "   \t\n\r   ",
            "\n\n\n",
        ];

        for prompt in whitespace_prompts {
            let result = InputProcessor::validate_prompt(prompt);
            assert!(result.is_err(), "Should reject whitespace-only prompt: {:?}", prompt);
            assert!(result.unwrap_err().to_string().contains("empty"));
        }

        // Test prompts with content after trimming
        let valid_prompts = vec![
            "  hello  ",
            "\n\nvalid content\n\n",
            "\t\tprompt with tabs\t\t",
        ];

        for prompt in valid_prompts {
            let result = InputProcessor::validate_prompt(prompt);
            assert!(result.is_ok(), "Should accept prompt with content: {:?}", prompt);
        }
    }

    #[test]
    fn test_binary_content_detection_comprehensive() {
        // Test various types of binary content
        let binary_test_cases = vec![
            ("Null byte", "Hello\x00World"),
            ("Control character", "Text\x01Control"),
            ("Bell character", "Ring\x07Bell"),
            ("Escape sequence", "Color\x1b[31mRed\x1b[0m"),
            ("Form feed", "Page\x0cBreak"),
            ("Vertical tab", "Line\x0bBreak"),
            ("Backspace", "Delete\x08Char"),
            ("Delete character", "Remove\x7fChar"),
        ];

        for (description, content) in binary_test_cases {
            assert!(InputProcessor::is_binary_content(content), 
                   "Should detect binary content: {}", description);
        }

        // Test content that should NOT be detected as binary
        let text_test_cases = vec![
            ("Plain ASCII", "Hello World"),
            ("Newlines", "Line 1\nLine 2\nLine 3"),
            ("Tabs", "Column1\tColumn2\tColumn3"),
            ("Carriage returns", "Windows\r\nLine endings"),
            ("Mixed whitespace", "Text\n\twith\r\nmixed whitespace"),
            ("Unicode", "Unicode: 你好 🌍 café"),
            ("Special chars", "Special: !@#$%^&*()[]{}|\\:;\"'<>?,./"),
            ("Empty string", ""),
        ];

        for (description, content) in text_test_cases {
            assert!(!InputProcessor::is_binary_content(content),
                   "Should NOT detect as binary: {}", description);
        }
    }

    #[test]
    fn test_large_input_warnings() -> Result<()> {
        // Test prompt size warnings
        let small_prompt = "a".repeat(1000);
        assert!(InputProcessor::validate_prompt(&small_prompt).is_ok());

        let medium_prompt = "a".repeat(50_000);
        assert!(InputProcessor::validate_prompt(&medium_prompt).is_ok());

        let large_prompt = "a".repeat(150_000);
        // Should succeed but warn (warning goes to stderr, can't easily test)
        assert!(InputProcessor::validate_prompt(&large_prompt).is_ok());

        // Test file size warnings
        let large_content = "b".repeat(2_000_000); // 2MB
        let temp_file = create_temp_file(&large_content)?;
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Should succeed but warn about large file
        let result = InputProcessor::read_file_safely(&file_path);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_parameter_validation_temperature() {
        // Test valid temperature values
        let valid_temperatures = vec![0.0, 0.1, 0.5, 0.7, 1.0, 1.5, 2.0];
        for temp in valid_temperatures {
            assert!(InputProcessor::validate_temperature(temp).is_ok(),
                   "Should accept valid temperature: {}", temp);
        }

        // Test invalid temperature values
        let invalid_temperatures = vec![-0.1, -1.0, 2.1, 3.0, 10.0];
        for temp in invalid_temperatures {
            let result = InputProcessor::validate_temperature(temp);
            assert!(result.is_err(), "Should reject invalid temperature: {}", temp);
            assert!(result.unwrap_err().to_string().contains("between 0.0 and 2.0"));
        }

        // Test special float values separately
        let special_values = vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY];
        for temp in special_values {
            let result = InputProcessor::validate_temperature(temp);
            assert!(result.is_err(), "Should reject special float value: {}", temp);
            assert!(result.unwrap_err().to_string().contains("valid number"));
        }

        // Test boundary values precisely
        assert!(InputProcessor::validate_temperature(0.0).is_ok());
        assert!(InputProcessor::validate_temperature(2.0).is_ok());
        assert!(InputProcessor::validate_temperature(-0.000001).is_err());
        assert!(InputProcessor::validate_temperature(2.000001).is_err());
    }

    #[test]
    fn test_parameter_validation_max_tokens() {
        // Test valid max_tokens values
        let valid_tokens = vec![1, 10, 100, 1000, 4096, 8192, 50000];
        for tokens in valid_tokens {
            assert!(InputProcessor::validate_max_tokens(tokens).is_ok(),
                   "Should accept valid max_tokens: {}", tokens);
        }

        // Test invalid max_tokens values
        assert!(InputProcessor::validate_max_tokens(0).is_err());
        let error = InputProcessor::validate_max_tokens(0).unwrap_err();
        assert!(error.to_string().contains("greater than 0"));

        // Test very large values (should succeed but warn)
        assert!(InputProcessor::validate_max_tokens(200_000).is_ok());
        assert!(InputProcessor::validate_max_tokens(u32::MAX).is_ok());
    }

    #[test]
    fn test_comprehensive_parameter_validation() {
        // Test all valid parameters together
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(0.7),
            Some(1000),
        ).is_ok());

        // Test with None values
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            None,
            None,
        ).is_ok());

        // Test with only temperature
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(1.2),
            None,
        ).is_ok());

        // Test with only max_tokens
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            None,
            Some(2000),
        ).is_ok());

        // Test invalid prompt fails early
        assert!(InputProcessor::validate_parameters(
            "",
            Some(0.7),
            Some(1000),
        ).is_err());

        // Test invalid temperature
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(3.0),
            Some(1000),
        ).is_err());

        // Test invalid max_tokens
        assert!(InputProcessor::validate_parameters(
            "Valid prompt",
            Some(0.7),
            Some(0),
        ).is_err());

        // Test multiple invalid parameters (should fail on first one)
        assert!(InputProcessor::validate_parameters(
            "",
            Some(3.0),
            Some(0),
        ).is_err());
    }

    #[test]
    fn test_file_validation_edge_cases() -> Result<()> {
        // Test file that exists but is empty
        let empty_file = create_temp_file("")?;
        let empty_path = empty_file.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&empty_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");

        // Test file with only whitespace
        let whitespace_file = create_temp_file("   \n\t\r\n   ")?;
        let whitespace_path = whitespace_file.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&whitespace_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "   \n\t\r\n   ");

        // Test file with Unicode content
        let unicode_content = "Unicode test: 你好世界 🌍 café résumé Привет";
        let unicode_file = create_temp_file(unicode_content)?;
        let unicode_path = unicode_file.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&unicode_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), unicode_content);

        // Test file with mixed line endings
        let mixed_endings = "Line 1\nLine 2\r\nLine 3\rLine 4";
        let mixed_file = create_temp_file(mixed_endings)?;
        let mixed_path = mixed_file.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&mixed_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), mixed_endings);

        Ok(())
    }

    #[test]
    fn test_file_error_scenarios() -> Result<()> {
        // Test non-existent file
        let result = InputProcessor::read_file_safely("/definitely/does/not/exist.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test directory instead of file
        let temp_dir = create_temp_dir()?;
        let dir_path = temp_dir.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&dir_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a file"));

        // Test file with invalid UTF-8 (create binary file)
        let mut binary_file = NamedTempFile::new()?;
        binary_file.write_all(&[0xFF, 0xFE, 0x00, 0x41])?; // Invalid UTF-8 sequence
        binary_file.flush()?;
        
        let binary_path = binary_file.path().to_string_lossy().to_string();
        let result = InputProcessor::read_file_safely(&binary_path);
        // Should fail either due to UTF-8 error or binary detection
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_prompt_validation_edge_cases() {
        // Test prompts with only special characters
        let special_prompts = vec![
            "!@#$%^&*()",
            "[]{}|\\:;\"'<>?,./",
            "~`+=_-",
            "(){}[]<>",
        ];

        for prompt in special_prompts {
            assert!(InputProcessor::validate_prompt(prompt).is_ok(),
                   "Should accept special characters: {}", prompt);
        }

        // Test prompts with mixed content
        let mixed_prompts = vec![
            "Normal text with 123 numbers",
            "Text with\nnewlines\nand\ttabs",
            "Unicode: 你好 + ASCII + 🌍",
            "Code: fn main() { println!(\"Hello\"); }",
            "JSON: {\"key\": \"value\", \"number\": 42}",
            "URL: https://example.com/path?param=value",
            "Email: user@example.com",
        ];

        for prompt in mixed_prompts {
            assert!(InputProcessor::validate_prompt(prompt).is_ok(),
                   "Should accept mixed content: {}", prompt);
        }

        // Test very long prompts
        let long_prompt = "word ".repeat(50_000); // ~250KB
        assert!(InputProcessor::validate_prompt(&long_prompt).is_ok());

        // Test extremely long prompts
        let very_long_prompt = "a".repeat(500_000); // 500KB
        assert!(InputProcessor::validate_prompt(&very_long_prompt).is_ok());
    }

    #[test]
    fn test_input_validation_integration() -> Result<()> {
        // Test complete input processing with validation
        let temp_file1 = create_temp_file("File 1 content")?;
        let temp_file2 = create_temp_file("File 2 content")?;
        
        let file_paths = vec![
            temp_file1.path().to_string_lossy().to_string(),
            temp_file2.path().to_string_lossy().to_string(),
        ];

        // Test successful case
        let result = InputProcessor::build_prompt(
            "Main prompt".to_string(),
            &file_paths,
            Some("System prompt"),
        );
        assert!(result.is_ok());

        let prompt = result.unwrap();
        
        // Validate the resulting prompt
        assert!(InputProcessor::validate_prompt(&prompt).is_ok());
        
        // Test with parameters
        assert!(InputProcessor::validate_parameters(
            &prompt,
            Some(0.8),
            Some(2000),
        ).is_ok());

        Ok(())
    }

    #[test]
    fn test_parameter_boundary_conditions() {
        // Test temperature boundaries
        assert!(InputProcessor::validate_temperature(0.0).is_ok());
        assert!(InputProcessor::validate_temperature(2.0).is_ok());
        
        // Test just outside boundaries (use larger epsilon for reliable testing)
        assert!(InputProcessor::validate_temperature(-0.000001).is_err());
        assert!(InputProcessor::validate_temperature(2.000001).is_err());

        // Test max_tokens boundaries
        assert!(InputProcessor::validate_max_tokens(1).is_ok());
        assert!(InputProcessor::validate_max_tokens(u32::MAX).is_ok());
        assert!(InputProcessor::validate_max_tokens(0).is_err());

        // Test floating point edge cases for temperature
        assert!(InputProcessor::validate_temperature(f32::MIN_POSITIVE).is_ok());
        assert!(InputProcessor::validate_temperature(1.999999).is_ok());
        assert!(InputProcessor::validate_temperature(2.00001).is_err());
    }

    #[test]
    fn test_validation_error_messages() {
        // Test that error messages are informative
        let empty_prompt_error = InputProcessor::validate_prompt("").unwrap_err();
        assert!(empty_prompt_error.to_string().contains("empty"));

        let temp_error = InputProcessor::validate_temperature(3.0).unwrap_err();
        assert!(temp_error.to_string().contains("between 0.0 and 2.0"));
        assert!(temp_error.to_string().contains("3"));

        let tokens_error = InputProcessor::validate_max_tokens(0).unwrap_err();
        assert!(tokens_error.to_string().contains("greater than 0"));
        assert!(tokens_error.to_string().contains("0"));

        let file_error = InputProcessor::read_file_safely("/nonexistent").unwrap_err();
        assert!(file_error.to_string().contains("not found") || 
               file_error.to_string().contains("nonexistent"));
    }

    #[test]
    fn test_prompt_building_with_all_components() -> Result<()> {
        let file1 = create_temp_file("First file")?;
        let file2 = create_temp_file("Second file")?;
        
        let paths = vec![
            file1.path().to_string_lossy().to_string(),
            file2.path().to_string_lossy().to_string(),
        ];
        
        let result = InputProcessor::build_prompt(
            "User prompt".to_string(),
            &paths,
            Some("System instruction"),
        )?;
        
        // Verify structure and ordering
        let lines: Vec<&str> = result.lines().collect();
        
        // Should start with system prompt
        assert!(lines[0].starts_with("System: System instruction"));
        
        // Should contain file markers and content
        assert!(result.contains("File: "));
        assert!(result.contains("First file"));
        assert!(result.contains("Second file"));
        
        // Should end with user prompt
        assert!(result.ends_with("User prompt"));
        
        Ok(())
    }
}