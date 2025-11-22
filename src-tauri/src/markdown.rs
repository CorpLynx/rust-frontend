use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub enum MessageSegment {
    Text(String),
    CodeBlock { language: Option<String>, code: String },
    InlineCode(String),
    Bold(String),
    Italic(String),
    ListItem(String),
    Highlighted(String), // For search result highlighting
}

static CODE_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Match code blocks with optional language, handling various newline scenarios
    Regex::new(r"```(\w+)?\s*\n?([\s\S]*?)\n?```").unwrap()
});

static INLINE_CODE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"`([^`]+)`").unwrap()
});

static BOLD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\*\*([^\*]+)\*\*").unwrap()
});

static ITALIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\*([^\*]+)\*").unwrap()
});

pub fn parse_message(content: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();

    // First, extract code blocks
    let mut code_blocks = Vec::new();
    for cap in CODE_BLOCK_REGEX.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let language = cap.get(1).map(|m| m.as_str().to_string());
        let code = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
        
        code_blocks.push((full_match.start(), full_match.end(), language, code));
    }

    // Process content, skipping code blocks
    let mut current_pos = 0;
    for (start, end, language, code) in code_blocks {
        // Process text before code block
        if current_pos < start {
            let text_segment = &content[current_pos..start];
            segments.extend(parse_inline_formatting(text_segment));
        }

        // Add code block
        segments.push(MessageSegment::CodeBlock { language, code });
        current_pos = end;
    }

    // Process remaining text after last code block
    if current_pos < content.len() {
        let text_segment = &content[current_pos..];
        segments.extend(parse_inline_formatting(text_segment));
    }

    // If no segments were created, return the original text
    if segments.is_empty() {
        segments.push(MessageSegment::Text(content.to_string()));
    }

    segments
}

fn parse_inline_formatting(text: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        // Check if it's a list item
        if line.trim_start().starts_with("- ") || line.trim_start().starts_with("* ") {
            let item_text = line.trim_start().trim_start_matches("- ").trim_start_matches("* ");
            segments.push(MessageSegment::ListItem(item_text.to_string()));
            continue;
        }

        // Parse inline code, bold, and italic
        let current_line = line.to_string();
        let mut line_segments = Vec::new();
        let mut last_pos = 0;

        // Find all inline code matches
        let mut inline_code_matches = Vec::new();
        for cap in INLINE_CODE_REGEX.captures_iter(&current_line) {
            if let Some(m) = cap.get(0) {
                inline_code_matches.push((m.start(), m.end(), cap.get(1).unwrap().as_str().to_string()));
            }
        }

        // Process inline code
        for (start, end, code) in inline_code_matches {
            if start > last_pos {
                let text_before = &current_line[last_pos..start];
                if !text_before.is_empty() {
                    line_segments.extend(parse_bold_italic(text_before));
                }
            }
            line_segments.push(MessageSegment::InlineCode(code));
            last_pos = end;
        }

        // Process remaining text
        if last_pos < current_line.len() {
            let remaining_text = &current_line[last_pos..];
            if !remaining_text.is_empty() {
                line_segments.extend(parse_bold_italic(remaining_text));
            }
        }

        // If no inline formatting was found, just add the line as text
        if line_segments.is_empty() {
            line_segments.push(MessageSegment::Text(line.to_string()));
        }

        segments.extend(line_segments);
        
        // Add newline between lines (except for the last one)
        if idx < lines.len() - 1 {
            segments.push(MessageSegment::Text("\n".to_string()));
        }
    }

    segments
}

fn parse_bold_italic(text: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();
    let mut last_pos = 0;

    // Find bold matches
    let mut bold_matches = Vec::new();
    for cap in BOLD_REGEX.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            bold_matches.push((m.start(), m.end(), cap.get(1).unwrap().as_str().to_string()));
        }
    }

    for (start, end, bold_text) in bold_matches {
        if start > last_pos {
            let text_before = &text[last_pos..start];
            if !text_before.is_empty() {
                segments.extend(parse_italic(text_before));
            }
        }
        segments.push(MessageSegment::Bold(bold_text));
        last_pos = end;
    }

    if last_pos < text.len() {
        let remaining = &text[last_pos..];
        if !remaining.is_empty() {
            segments.extend(parse_italic(remaining));
        }
    }

    if segments.is_empty() {
        segments.push(MessageSegment::Text(text.to_string()));
    }

    segments
}

fn parse_italic(text: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();
    let mut last_pos = 0;

    // Find italic matches (but not bold)
    for cap in ITALIC_REGEX.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            // Skip if this is part of a bold marker (**)
            if m.start() > 0 && text.chars().nth(m.start() - 1) == Some('*') {
                continue;
            }
            if m.end() < text.len() && text.chars().nth(m.end()) == Some('*') {
                continue;
            }

            if m.start() > last_pos {
                let text_before = &text[last_pos..m.start()];
                if !text_before.is_empty() {
                    segments.push(MessageSegment::Text(text_before.to_string()));
                }
            }
            segments.push(MessageSegment::Italic(cap.get(1).unwrap().as_str().to_string()));
            last_pos = m.end();
        }
    }

    if last_pos < text.len() {
        let remaining = &text[last_pos..];
        if !remaining.is_empty() {
            segments.push(MessageSegment::Text(remaining.to_string()));
        }
    }

    if segments.is_empty() {
        segments.push(MessageSegment::Text(text.to_string()));
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block() {
        let input = "Here's some code:\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\nThat's it!";
        let segments = parse_message(input);
        
        assert!(segments.iter().any(|s| matches!(s, MessageSegment::CodeBlock { .. })));
    }

    #[test]
    fn test_inline_code() {
        let input = "Use `println!` to print.";
        let segments = parse_message(input);
        
        assert!(segments.iter().any(|s| matches!(s, MessageSegment::InlineCode(_))));
    }

    #[test]
    fn test_bold() {
        let input = "This is **bold** text.";
        let segments = parse_message(input);
        
        assert!(segments.iter().any(|s| matches!(s, MessageSegment::Bold(_))));
    }

    #[test]
    fn test_italic() {
        let input = "This is *italic* text.";
        let segments = parse_message(input);
        
        assert!(segments.iter().any(|s| matches!(s, MessageSegment::Italic(_))));
    }

    #[test]
    fn test_list() {
        let input = "- Item 1\n- Item 2";
        let segments = parse_message(input);
        
        assert!(segments.iter().any(|s| matches!(s, MessageSegment::ListItem(_))));
    }

    #[test]
    fn test_highlight_matches() {
        let text = "Hello world, hello again";
        let positions = vec![(0, 5), (13, 18)];
        let segments = super::highlight_matches(text, &positions);
        
        // Should have highlighted segments
        assert!(segments.iter().any(|s| matches!(s, MessageSegment::Highlighted(_))));
    }

    #[test]
    fn test_highlight_matches_empty() {
        let text = "Hello world";
        let positions = vec![];
        let segments = super::highlight_matches(text, &positions);
        
        // Should return original text
        assert_eq!(segments.len(), 1);
        assert!(matches!(segments[0], MessageSegment::Text(_)));
    }

    #[test]
    fn test_highlight_matches_overlapping() {
        let text = "Hello world";
        let positions = vec![(0, 5), (3, 8)];
        let segments = super::highlight_matches(text, &positions);
        
        // Should handle overlapping positions
        assert!(!segments.is_empty());
    }
}

/// Highlight search matches in text by creating segments with highlighted regions
pub fn highlight_matches(text: &str, positions: &[(usize, usize)]) -> Vec<MessageSegment> {
    if positions.is_empty() {
        return vec![MessageSegment::Text(text.to_string())];
    }

    let mut segments = Vec::new();
    let mut sorted_positions = positions.to_vec();
    
    // Sort positions by start index
    sorted_positions.sort_by_key(|&(start, _)| start);
    
    // Merge overlapping positions
    let mut merged_positions = Vec::new();
    let mut current_start = sorted_positions[0].0;
    let mut current_end = sorted_positions[0].1;
    
    for &(start, end) in sorted_positions.iter().skip(1) {
        if start <= current_end {
            // Overlapping or adjacent, merge them
            current_end = current_end.max(end);
        } else {
            // Non-overlapping, save current and start new
            merged_positions.push((current_start, current_end));
            current_start = start;
            current_end = end;
        }
    }
    merged_positions.push((current_start, current_end));
    
    // Build segments with highlighted regions
    let mut last_pos = 0;
    
    for (start, end) in merged_positions {
        // Ensure positions are within bounds
        let start = start.min(text.len());
        let end = end.min(text.len());
        
        if start >= end {
            continue;
        }
        
        // Add text before highlight
        if start > last_pos {
            let before_text = &text[last_pos..start];
            if !before_text.is_empty() {
                segments.push(MessageSegment::Text(before_text.to_string()));
            }
        }
        
        // Add highlighted text
        let highlighted_text = &text[start..end];
        if !highlighted_text.is_empty() {
            segments.push(MessageSegment::Highlighted(highlighted_text.to_string()));
        }
        
        last_pos = end;
    }
    
    // Add remaining text after last highlight
    if last_pos < text.len() {
        let remaining_text = &text[last_pos..];
        if !remaining_text.is_empty() {
            segments.push(MessageSegment::Text(remaining_text.to_string()));
        }
    }
    
    // If no segments were created, return original text
    if segments.is_empty() {
        segments.push(MessageSegment::Text(text.to_string()));
    }
    
    segments
}
