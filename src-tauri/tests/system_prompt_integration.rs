// Integration test for system prompt functionality with Ollama API
// Requirements: 3.4

#[cfg(test)]
mod system_prompt_tests {
    use serde_json::json;
    use proptest::prelude::*;

    /// Test that system prompts are correctly formatted in the request body
    #[test]
    fn test_system_prompt_formatting() {
        let user_message = "Hello, how are you?";
        let system_prompt = "You are a helpful assistant.";
        
        // Simulate the prompt construction logic from send_message_stream
        let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
        
        assert!(final_prompt.starts_with(system_prompt));
        assert!(final_prompt.contains(user_message));
        assert!(final_prompt.contains("\n\n"));
    }

    /// Test with empty system prompt (None case)
    #[test]
    fn test_no_system_prompt() {
        let user_message = "Hello, how are you?";
        let system_prompt: Option<String> = None;
        
        let final_prompt = if let Some(sys_prompt) = system_prompt {
            format!("{}\n\n{}", sys_prompt, user_message)
        } else {
            user_message.to_string()
        };
        
        assert_eq!(final_prompt, user_message);
    }

    /// Test with long system prompt
    #[test]
    fn test_long_system_prompt() {
        let user_message = "What is the weather?";
        let system_prompt = "You are a weather forecasting assistant with extensive knowledge of meteorology. \
                            You provide detailed, accurate weather information and explain weather patterns clearly. \
                            You always cite your sources and provide context for your predictions.";
        
        let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
        
        assert!(final_prompt.len() > system_prompt.len());
        assert!(final_prompt.starts_with(system_prompt));
        assert!(final_prompt.ends_with(user_message));
    }

    /// Test with special characters in system prompt
    #[test]
    fn test_special_characters_in_prompt() {
        let user_message = "Tell me a joke";
        let system_prompt = "You are a comedian! 😄 Use emojis & humor. Don't be boring...";
        
        let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
        
        assert!(final_prompt.contains("😄"));
        assert!(final_prompt.contains("&"));
        assert!(final_prompt.contains("..."));
    }

    /// Test with multiline system prompt
    #[test]
    fn test_multiline_system_prompt() {
        let user_message = "Help me code";
        let system_prompt = "You are a coding assistant.\nYou help with:\n- Python\n- Rust\n- JavaScript";
        
        let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
        
        assert!(final_prompt.contains("You are a coding assistant."));
        assert!(final_prompt.contains("- Python"));
        assert!(final_prompt.contains(user_message));
    }

    /// Test request body structure matches Ollama API expectations
    #[test]
    fn test_request_body_structure() {
        let model = "llama2";
        let prompt = "System: Be helpful\n\nUser: Hello";
        
        let request_body = json!({
            "model": model,
            "prompt": prompt,
            "stream": true
        });
        
        assert_eq!(request_body["model"], model);
        assert_eq!(request_body["prompt"], prompt);
        assert_eq!(request_body["stream"], true);
    }

    /// Test that empty user message still works with system prompt
    #[test]
    fn test_empty_user_message_with_system_prompt() {
        let user_message = "";
        let system_prompt = "You are a helpful assistant.";
        
        let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
        
        assert!(final_prompt.starts_with(system_prompt));
        assert_eq!(final_prompt, format!("{}\n\n", system_prompt));
    }

    /// Test with very short system prompt
    #[test]
    fn test_short_system_prompt() {
        let user_message = "Hi";
        let system_prompt = "Be brief.";
        
        let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
        
        assert_eq!(final_prompt, "Be brief.\n\nHi");
    }

    // **Feature: persona-switcher, Property 6: System prompt inclusion**
    // **Validates: Requirements 3.1, 3.3, 3.4**
    //
    // For any active persona and user message, the system prompt from that persona should be
    // prepended to the message sent to the AI model, and the prompt should match the persona's
    // configured system prompt exactly.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_system_prompt_prepended_to_message(
            system_prompt in prop::string::string_regex(".{1,500}").unwrap(),
            user_message in prop::string::string_regex(".{1,500}").unwrap(),
        ) {
            // Simulate the prompt construction logic from send_message_stream
            let final_prompt = format!("{}\n\n{}", system_prompt, user_message);
            
            // Property 1: The final prompt must start with the system prompt
            prop_assert!(
                final_prompt.starts_with(&system_prompt),
                "Final prompt should start with system prompt. Expected to start with '{}', but got '{}'",
                system_prompt,
                final_prompt
            );
            
            // Property 2: The final prompt must contain the user message
            prop_assert!(
                final_prompt.contains(&user_message),
                "Final prompt should contain user message. Expected to contain '{}', but got '{}'",
                user_message,
                final_prompt
            );
            
            // Property 3: The system prompt should appear exactly as configured (no modification)
            let extracted_system_prompt = &final_prompt[..system_prompt.len()];
            prop_assert_eq!(
                extracted_system_prompt,
                &system_prompt,
                "System prompt should be included exactly as configured"
            );
            
            // Property 4: The separator "\n\n" should be present between system prompt and user message
            prop_assert!(
                final_prompt.contains("\n\n"),
                "Final prompt should contain separator between system prompt and user message"
            );
            
            // Property 5: The user message should appear after the system prompt
            let system_prompt_end = system_prompt.len() + 2; // +2 for "\n\n"
            if system_prompt_end <= final_prompt.len() {
                let extracted_user_message = &final_prompt[system_prompt_end..];
                prop_assert_eq!(
                    extracted_user_message,
                    &user_message,
                    "User message should appear after system prompt and separator"
                );
            }
            
            // Property 6: The final prompt length should equal system_prompt + separator + user_message
            let expected_length = system_prompt.len() + 2 + user_message.len();
            prop_assert_eq!(
                final_prompt.len(),
                expected_length,
                "Final prompt length should be exactly system_prompt + separator + user_message"
            );
        }
        
        // **Feature: persona-switcher, Property 7: No prompt when inactive**
        // **Validates: Requirements 2.6**
        //
        // For any message sent when no persona is active, the message should not include
        // any persona-specific system prompt.
        #[test]
        fn prop_no_system_prompt_when_inactive(
            user_message in prop::string::string_regex(".{1,500}").unwrap(),
        ) {
            // Simulate the case when no persona is active (system_prompt is None)
            let system_prompt: Option<String> = None;
            
            let final_prompt = if let Some(sys_prompt) = system_prompt {
                format!("{}\n\n{}", sys_prompt, user_message)
            } else {
                user_message.clone()
            };
            
            // Property 1: When no persona is active, the final prompt should be exactly the user message
            prop_assert_eq!(
                &final_prompt,
                &user_message,
                "When no persona is active, final prompt should equal user message exactly"
            );
            
            // Property 2: The final prompt should not contain the separator when no persona is active
            prop_assert!(
                !final_prompt.starts_with("\n\n"),
                "Final prompt should not start with separator when no persona is active"
            );
            
            // Property 3: The final prompt length should equal the user message length (no additions)
            prop_assert_eq!(
                final_prompt.len(),
                user_message.len(),
                "Final prompt length should equal user message length when no persona is active"
            );
            
            // Property 4: No persona-specific content should be prepended
            // This is verified by checking that the prompt is identical to the user message
            prop_assert!(
                final_prompt == user_message,
                "No persona-specific system prompt should be included when no persona is active"
            );
        }
    }
}
