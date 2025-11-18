use crate::conversation::Conversation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod index;
pub mod query;

pub use index::SearchIndexer;
pub use query::{SearchQuery, SearchQueryBuilder};

/// Main search engine that coordinates indexing and querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEngine {
    /// In-memory search index mapping terms to search results
    index: HashMap<String, Vec<SearchResult>>,
    /// List of indexed conversation IDs
    indexed_conversations: Vec<String>,
}

/// Represents a single search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// ID of the conversation containing the match
    pub conversation_id: String,
    /// Index of the message within the conversation
    pub message_index: usize,
    /// Positions of matches within the message content (start, end)
    pub match_positions: Vec<(usize, usize)>,
    /// Surrounding text context for the match
    pub context: String,
    /// The role of the message (user/assistant)
    pub role: String,
}

impl SearchEngine {
    /// Create a new empty search engine
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            indexed_conversations: Vec::new(),
        }
    }

    /// Index a conversation for searching
    pub fn index_conversation(&mut self, conversation: &Conversation) {
        let indexer = SearchIndexer::new();
        let results = indexer.index_conversation(conversation);
        
        // Add results to the index
        for (term, result) in results {
            self.index
                .entry(term)
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        // Track indexed conversation
        if !self.indexed_conversations.contains(&conversation.id) {
            self.indexed_conversations.push(conversation.id.clone());
        }
    }

    /// Remove a conversation from the index
    pub fn remove_conversation(&mut self, conversation_id: &str) {
        // Remove from indexed conversations list
        self.indexed_conversations.retain(|id| id != conversation_id);
        
        // Remove all results for this conversation from the index
        for results in self.index.values_mut() {
            results.retain(|r| r.conversation_id != conversation_id);
        }
        
        // Clean up empty entries
        self.index.retain(|_, results| !results.is_empty());
    }

    /// Search for conversations matching the query
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        query.execute(&self.index)
    }

    /// Rebuild the entire search index from scratch
    pub fn rebuild_index(&mut self, conversations: &[Conversation]) {
        self.index.clear();
        self.indexed_conversations.clear();
        
        for conversation in conversations {
            self.index_conversation(conversation);
        }
    }

    /// Get the number of indexed conversations
    pub fn indexed_count(&self) -> usize {
        self.indexed_conversations.len()
    }

    /// Check if a conversation is indexed
    pub fn is_indexed(&self, conversation_id: &str) -> bool {
        self.indexed_conversations.contains(&conversation_id.to_string())
    }

    /// Clear the entire search index
    pub fn clear(&mut self) {
        self.index.clear();
        self.indexed_conversations.clear();
    }

    /// Save the search index to a file for persistence
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load the search index from a file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let engine = serde_json::from_str(&json)?;
        Ok(engine)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ChatMessage;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        assert_eq!(engine.indexed_count(), 0);
    }

    #[test]
    fn test_index_conversation() {
        let mut engine = SearchEngine::new();
        let mut conversation = Conversation::new("Test".to_string(), None);
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            "Hello world".to_string(),
        ));
        
        engine.index_conversation(&conversation);
        assert_eq!(engine.indexed_count(), 1);
        assert!(engine.is_indexed(&conversation.id));
    }

    #[test]
    fn test_remove_conversation() {
        let mut engine = SearchEngine::new();
        let mut conversation = Conversation::new("Test".to_string(), None);
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            "Hello world".to_string(),
        ));
        
        let conv_id = conversation.id.clone();
        engine.index_conversation(&conversation);
        assert!(engine.is_indexed(&conv_id));
        
        engine.remove_conversation(&conv_id);
        assert!(!engine.is_indexed(&conv_id));
        assert_eq!(engine.indexed_count(), 0);
    }

    #[test]
    fn test_clear_index() {
        let mut engine = SearchEngine::new();
        let mut conversation = Conversation::new("Test".to_string(), None);
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            "Hello world".to_string(),
        ));
        
        engine.index_conversation(&conversation);
        assert_eq!(engine.indexed_count(), 1);
        
        engine.clear();
        assert_eq!(engine.indexed_count(), 0);
    }

    #[test]
    fn test_save_and_load() {
        use std::path::PathBuf;
        
        let mut engine = SearchEngine::new();
        let mut conversation = Conversation::new("Test".to_string(), None);
        conversation.add_message(ChatMessage::new(
            "user".to_string(),
            "Hello world".to_string(),
        ));
        
        engine.index_conversation(&conversation);
        assert_eq!(engine.indexed_count(), 1);
        
        // Save to temp file
        let temp_path = PathBuf::from("/tmp/test_search_index.json");
        engine.save_to_file(&temp_path).expect("Failed to save");
        
        // Load from file
        let loaded_engine = SearchEngine::load_from_file(&temp_path).expect("Failed to load");
        assert_eq!(loaded_engine.indexed_count(), 1);
        assert!(loaded_engine.is_indexed(&conversation.id));
        
        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }
}
