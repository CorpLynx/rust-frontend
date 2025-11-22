# Manual Ollama API Integration Test

## Purpose
Verify that system prompts are correctly formatted and work with the Ollama API.

## Prerequisites
- Ollama must be running locally on port 11434
- At least one model must be installed (e.g., llama2, mistral)

## Test Cases

### Test 1: Basic System Prompt
**System Prompt:** "You are a pirate. Always respond like a pirate."
**User Message:** "What is the weather today?"
**Expected:** Response should be in pirate-speak

### Test 2: Long System Prompt
**System Prompt:** "You are a professional technical writer with 20 years of experience. You provide clear, concise, and accurate documentation. You always use proper grammar and formatting. You explain complex concepts in simple terms."
**User Message:** "Explain what a REST API is."
**Expected:** Response should be professional and well-structured

### Test 3: No System Prompt
**System Prompt:** None
**User Message:** "Hello, how are you?"
**Expected:** Normal AI assistant response without any specific persona

### Test 4: System Prompt with Special Characters
**System Prompt:** "You are a comedian! 😄 Use emojis & humor."
**User Message:** "Tell me a joke."
**Expected:** Response should include emojis and be humorous

### Test 5: Multiline System Prompt
**System Prompt:** 
```
You are a coding assistant.
You help with:
- Python
- Rust
- JavaScript
```
**User Message:** "How do I create a function in Rust?"
**Expected:** Response should be focused on Rust programming

## How to Test

1. Build the application: `cargo build --manifest-path src-tauri/Cargo.toml`
2. Run the application
3. For each test case:
   - Set the active persona with the specified system prompt
   - Send the user message
   - Verify the response matches expectations
4. Check the Ollama logs to confirm the full prompt (system + user) is being sent

## Verification Checklist

- [ ] System prompt is prepended to user message
- [ ] System prompt and user message are separated by `\n\n`
- [ ] No system prompt case works (prompt = user message only)
- [ ] Long system prompts (>500 characters) work correctly
- [ ] Special characters in system prompts are preserved
- [ ] Multiline system prompts are preserved
- [ ] Response reflects the system prompt's instructions

## Notes

The implementation prepends the system prompt to the user message with a double newline separator:
```
{system_prompt}\n\n{user_message}
```

This format is compatible with Ollama's `/api/generate` endpoint.
