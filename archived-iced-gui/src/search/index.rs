use crate::conversation::Conversation;
use crate::search::SearchResult;
use regex::Regex;
use std::collections::HashMap;

/// Handles the indexing of conversations for search
pub struct SearchIndexer {
    /// Minimum word length to index
    min_word_length: usize,
}

impl SearchIndexer {
    /// Create a new search indexer with default settings
    pub fn new() -> Self {
        Self {
            min_word_length: 2,
        }
    }

    /// Create a new search indexer with custom minimum word length
    pub fn with_min_word_length(min_word_length: usize) -> Self {
        Self { min_word_length }
    }

    /// Index a conversation and return a map of terms to search results
    pub fn index_conversation(&self, conversation: &Conversation) -> HashMap<String, SearchResult> {
        let mut results = HashMap::new();

        for (message_index, message) in conversation.messages.iter().enumerate() {
            let tokens = self.tokenize(&message.content);
            
            for (term, positions) in tokens {
                // Skip terms that are too short
                if term.len() < self.min_word_length {
                    continue;
                }

                let context = self.extract_context(&message.content, &positions);
                
                let result = SearchResult {
                    conversation_id: conversation.id.clone(),
                    message_index,
                    match_positions: positions,
                    context,
                    role: message.role.clone(),
                };

                // Use lowercase term as key for case-insensitive indexing
                results.insert(term.to_lowercase(), result);
            }
        }

        results
    }

    /// Tokenize text into terms and their positions
    fn tokenize(&self, text: &str) -> HashMap<String, Vec<(usize, usize)>> {
        let mut tokens: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        
        // Match word characters (letters, numbers, underscores)
        let word_regex = Regex::new(r"\w+").unwrap();
        
        for mat in word_regex.find_iter(text) {
            let term = mat.as_str().to_string();
            let start = mat.start();
            let end = mat.end();
            
            tokens
                .entry(term)
                .or_insert_with(Vec::new)
                .push((start, end));
        }
        
        tokens
    }

    /// Extract context around match positions
    fn extract_context(&self, text: &str, positions: &[(usize, usize)]) -> String {
        if positions.is_empty() {
            return String::new();
        }

        // Use the first match position for context
        let (start, end) = positions[0];
        
        // Context window: 50 characters before and after
        let context_window = 50;
        
        let context_start = if start > context_window {
            start - context_window
        } else {
            0
        };
        
        let context_end = std::cmp::min(end + context_window, text.len());
        
        // Ensure we're slicing at valid UTF-8 character boundaries
        let safe_start = self.find_char_boundary(text, context_start);
        let safe_end = self.find_char_boundary(text, context_end);
        
        let mut context = text[safe_start..safe_end].to_string();
        
        // Add ellipsis if we're not at the boundaries
        if safe_start > 0 {
            context = format!("...{}", context);
        }
        if safe_end < text.len() {
            context = format!("{}...", context);
        }
        
        context
    }

    /// Find the nearest valid UTF-8 character boundary at or before the given byte index
    fn find_char_boundary(&self, text: &str, byte_index: usize) -> usize {
        let mut index = byte_index;
        
        // Clamp to text length
        if index > text.len() {
            return text.len();
        }
        
        // Move backwards until we find a valid character boundary
        while index > 0 && !text.is_char_boundary(index) {
            index -= 1;
        }
        
        index
    }

    /// Normalize text for indexing (lowercase, trim)
    pub fn normalize_text(text: &str) -> String {
        text.trim().to_lowercase()
    }
}

impl Default for SearchIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ChatMessage;

    #[test]
    fn test_tokenize() {
        let indexer = SearchIndexer::new();
        let tokens = indexer.tokenize("Hello world! This is a test.");
        
        assert!(tokens.contains_key("Hello"));
        assert!(tokens.contains_key("world"));
        assert!(tokens.contains_key("test"));
    }

    #[test]
    fn test_tokenize_with_positions() {
        let indexer = SearchIndexer::new();
        let tokens = indexer.tokenize("Hello world");
        
        let hello_positions = tokens.get("Hello").unwrap();
        assert_eq!(hello_positions[0], (0, 5));
        
        let world_positions = tokens.get("world").unwrap();
        assert_eq!(world_positions[0], (6, 11));
    }

    #[test]
    fn test_extract_context() {
        let indexer = SearchIndexer::new();
        let text = "This is a long text with many words that we want to search through";
        let positions = vec![(30, 35)]; // "words"
        
        let context = indexer.extract_context(text, &positions);
        assert!(context.contains("words"));
    }

    #[test]
    fn test_extract_context_with_ellipsis() {
        let indexer = SearchIndexer::new();
        let text = "a".repeat(200); // Long text
        let positions = vec![(100, 105)];
        
        let context = indexer.extract_context(&text, &positions);
        assert!(context.starts_with("..."));
        assert!(context.ends_with("..."));
    }

    #[test]
    fn test_index_conversation() {
        let indexer = SearchIndexer::new();
        let mut conversation = Conversation::new("Test".to_string(), None);
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            "Hello world".to_string(),
        ));
        
        let results = indexer.index_conversation(&conversation);
        
        // Should have indexed "hello" and "world" (lowercase keys)
        assert!(results.contains_key("hello"));
        assert!(results.contains_key("world"));
    }

    #[test]
    fn test_min_word_length() {
        let indexer = SearchIndexer::with_min_word_length(3);
        let mut conversation = Conversation::new("Test".to_string(), None);
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            "I am testing".to_string(),
        ));
        
        let results = indexer.index_conversation(&conversation);
        
        // "I" and "am" should be filtered out (< 3 chars)
        assert!(!results.contains_key("i"));
        assert!(!results.contains_key("am"));
        assert!(results.contains_key("testing"));
    }

    #[test]
    fn test_normalize_text() {
        assert_eq!(SearchIndexer::normalize_text("  Hello World  "), "hello world");
        assert_eq!(SearchIndexer::normalize_text("UPPERCASE"), "uppercase");
    }

    #[test]
    fn test_extract_context_with_unicode() {
        let indexer = SearchIndexer::new();
        // Text with multi-byte UTF-8 characters (emoji, special quotes)
        let text = "This is a test with special quotes and emoji 🎉 in the middle of text";
        let positions = vec![(20, 27)]; // "special"
        
        let context = indexer.extract_context(text, &positions);
        // Should not panic and should contain the match
        assert!(context.contains("special"));
    }

    #[test]
    fn test_extract_context_unicode_boundary() {
        let indexer = SearchIndexer::new();
        // Create text where a match position might land near multi-byte characters
        let text = "* **ESXi Advanced:**  Includes additional features for larger deployments.4. **VMware Cloud on AWS:**  VMware Cloud on AWS provides a managed ESXi environment";
        let positions = vec![(50, 58)]; // somewhere in the middle
        
        let context = indexer.extract_context(text, &positions);
        // Should not panic
        assert!(!context.is_empty());
    }

    #[test]
    fn test_extract_context_exact_panic_case() {
        let indexer = SearchIndexer::new();
        // This is the exact text from the panic error with special quotes (multi-byte UTF-8)
        let text = "* **ESXi Advanced:**  Includes additional features for larger deployments.4. **VMware Cloud on AWS:**  VMware Cloud on AWS provides a managed ESXi environment, and you can choose from different pricing tiers based on your needs. This is a good option if";
        
        // The panic occurred at byte index 408, which is inside the " character (3 bytes in UTF-8)
        // Let's test with a position that would cause context extraction around that problematic area
        // The word "option" starts around byte 240
        let positions = vec![(240, 246)]; // "option"
        
        let context = indexer.extract_context(text, &positions);
        // Should not panic - this is the main test
        assert!(!context.is_empty());
        // The context should contain the matched word
        assert!(context.contains("option"));
    }

    #[test]
    fn test_find_char_boundary() {
        let indexer = SearchIndexer::new();
        let text = "Hello 世界"; // "Hello World" in Chinese
        
        // The Chinese characters are 3 bytes each
        // "Hello " is 6 bytes, then "世" is bytes 6-9, "界" is bytes 9-12
        
        // Test finding boundary at valid position
        assert_eq!(indexer.find_char_boundary(text, 6), 6);
        
        // Test finding boundary in middle of multi-byte char (should move back)
        assert_eq!(indexer.find_char_boundary(text, 7), 6);
        assert_eq!(indexer.find_char_boundary(text, 8), 6);
        
        // Test at end
        assert_eq!(indexer.find_char_boundary(text, text.len()), text.len());
        
        // Test beyond end (should clamp)
        assert_eq!(indexer.find_char_boundary(text, text.len() + 10), text.len());
    }
}
