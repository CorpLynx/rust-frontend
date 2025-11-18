use crate::search::SearchResult;
use std::collections::HashMap;

/// Represents a search query with various options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchQuery {
    /// The search text
    pub text: String,
    /// Whether the search should be case-sensitive
    pub case_sensitive: bool,
    /// Whether to match whole words only
    pub whole_word: bool,
}

impl SearchQuery {
    /// Create a new search query with default options
    pub fn new(text: String) -> Self {
        Self {
            text,
            case_sensitive: false,
            whole_word: false,
        }
    }

    /// Execute the search query against the index
    pub fn execute(&self, index: &HashMap<String, Vec<SearchResult>>) -> Vec<SearchResult> {
        if self.text.is_empty() {
            return Vec::new();
        }

        let search_term = if self.case_sensitive {
            self.text.clone()
        } else {
            self.text.to_lowercase()
        };

        // For whole word matching, look up exact term in index
        if self.whole_word {
            return index
                .get(&search_term)
                .cloned()
                .unwrap_or_default();
        }

        // For partial matching, find all terms containing the search term
        let mut results = Vec::new();
        for (term, term_results) in index.iter() {
            let matches = if self.case_sensitive {
                // Case-sensitive: compare as-is
                term.contains(&search_term)
            } else {
                // Case-insensitive: both already lowercase
                term.contains(&search_term)
            };
            
            if matches {
                results.extend(term_results.clone());
            }
        }

        // Sort results by conversation_id and message_index for consistent ordering
        results.sort_by(|a, b| {
            a.conversation_id
                .cmp(&b.conversation_id)
                .then(a.message_index.cmp(&b.message_index))
        });

        // Remove duplicates (same conversation and message)
        results.dedup_by(|a, b| {
            a.conversation_id == b.conversation_id && a.message_index == b.message_index
        });

        results
    }

    /// Set case sensitivity
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Set whole word matching
    pub fn with_whole_word(mut self, whole_word: bool) -> Self {
        self.whole_word = whole_word;
        self
    }
}

/// Builder for constructing search queries
pub struct SearchQueryBuilder {
    text: String,
    case_sensitive: bool,
    whole_word: bool,
}

impl SearchQueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            text: String::new(),
            case_sensitive: false,
            whole_word: false,
        }
    }

    /// Set the search text
    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    /// Enable case-sensitive search
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Enable whole word matching
    pub fn whole_word(mut self, enabled: bool) -> Self {
        self.whole_word = enabled;
        self
    }

    /// Build the search query
    pub fn build(self) -> SearchQuery {
        SearchQuery {
            text: self.text,
            case_sensitive: self.case_sensitive,
            whole_word: self.whole_word,
        }
    }
}

impl Default for SearchQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchResult;

    fn create_test_result(conv_id: &str, msg_idx: usize) -> SearchResult {
        SearchResult {
            conversation_id: conv_id.to_string(),
            message_index: msg_idx,
            match_positions: vec![(0, 5)],
            context: "test context".to_string(),
            role: "user".to_string(),
        }
    }

    #[test]
    fn test_query_creation() {
        let query = SearchQuery::new("test".to_string());
        assert_eq!(query.text, "test");
        assert!(!query.case_sensitive);
        assert!(!query.whole_word);
    }

    #[test]
    fn test_query_with_options() {
        let query = SearchQuery::new("test".to_string())
            .with_case_sensitive(true)
            .with_whole_word(true);
        
        assert!(query.case_sensitive);
        assert!(query.whole_word);
    }

    #[test]
    fn test_query_builder() {
        let query = SearchQueryBuilder::new()
            .text("test".to_string())
            .case_sensitive(true)
            .whole_word(false)
            .build();
        
        assert_eq!(query.text, "test");
        assert!(query.case_sensitive);
        assert!(!query.whole_word);
    }

    #[test]
    fn test_execute_empty_query() {
        let query = SearchQuery::new("".to_string());
        let index = HashMap::new();
        let results = query.execute(&index);
        assert!(results.is_empty());
    }

    #[test]
    fn test_execute_whole_word() {
        let query = SearchQuery::new("hello".to_string()).with_whole_word(true);
        
        let mut index = HashMap::new();
        index.insert("hello".to_string(), vec![create_test_result("conv1", 0)]);
        index.insert("helloworld".to_string(), vec![create_test_result("conv2", 0)]);
        
        let results = query.execute(&index);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].conversation_id, "conv1");
    }

    #[test]
    fn test_execute_partial_match() {
        let query = SearchQuery::new("hello".to_string());
        
        let mut index = HashMap::new();
        index.insert("hello".to_string(), vec![create_test_result("conv1", 0)]);
        index.insert("helloworld".to_string(), vec![create_test_result("conv2", 0)]);
        
        let results = query.execute(&index);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_execute_case_insensitive() {
        let query = SearchQuery::new("HELLO".to_string());
        
        let mut index = HashMap::new();
        index.insert("hello".to_string(), vec![create_test_result("conv1", 0)]);
        
        let results = query.execute(&index);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_execute_case_sensitive() {
        let query = SearchQuery::new("HELLO".to_string()).with_case_sensitive(true);
        
        let mut index = HashMap::new();
        index.insert("hello".to_string(), vec![create_test_result("conv1", 0)]);
        index.insert("HELLO".to_string(), vec![create_test_result("conv2", 0)]);
        
        let results = query.execute(&index);
        // Case sensitive search for "HELLO" should only match the "HELLO" key
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].conversation_id, "conv2");
    }

    #[test]
    fn test_results_deduplication() {
        let query = SearchQuery::new("test".to_string());
        
        let mut index = HashMap::new();
        // Add duplicate results for same conversation and message
        index.insert("test".to_string(), vec![
            create_test_result("conv1", 0),
            create_test_result("conv1", 0),
        ]);
        index.insert("testing".to_string(), vec![
            create_test_result("conv1", 0),
        ]);
        
        let results = query.execute(&index);
        // Should deduplicate to single result
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_results_sorting() {
        let query = SearchQuery::new("test".to_string());
        
        let mut index = HashMap::new();
        index.insert("test".to_string(), vec![
            create_test_result("conv2", 1),
            create_test_result("conv1", 2),
            create_test_result("conv1", 0),
        ]);
        
        let results = query.execute(&index);
        
        // Should be sorted by conversation_id, then message_index
        assert_eq!(results[0].conversation_id, "conv1");
        assert_eq!(results[0].message_index, 0);
        assert_eq!(results[1].conversation_id, "conv1");
        assert_eq!(results[1].message_index, 2);
        assert_eq!(results[2].conversation_id, "conv2");
        assert_eq!(results[2].message_index, 1);
    }
}
