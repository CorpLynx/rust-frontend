// Simple markdown parser for terminal display
export function parseMarkdown(text) {
  const segments = [];
  
  // Extract code blocks first
  const codeBlockRegex = /```(\w+)?\s*\n?([\s\S]*?)\n?```/g;
  let lastIndex = 0;
  let match;
  
  while ((match = codeBlockRegex.exec(text)) !== null) {
    // Add text before code block
    if (match.index > lastIndex) {
      const textBefore = text.substring(lastIndex, match.index);
      segments.push(...parseInlineFormatting(textBefore));
    }
    
    // Add code block
    segments.push({
      type: 'codeBlock',
      language: match[1] || 'code',
      code: match[2]
    });
    
    lastIndex = match.index + match[0].length;
  }
  
  // Add remaining text
  if (lastIndex < text.length) {
    const remaining = text.substring(lastIndex);
    segments.push(...parseInlineFormatting(remaining));
  }
  
  return segments.length > 0 ? segments : [{ type: 'text', content: text }];
}

function parseInlineFormatting(text) {
  const segments = [];
  const lines = text.split('\n');
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    
    // Check for list items
    if (line.trim().startsWith('- ') || line.trim().startsWith('* ')) {
      segments.push({
        type: 'listItem',
        content: line.trim().substring(2)
      });
    } else {
      // Parse inline code, bold, italic
      const lineSegments = parseInlineStyles(line);
      segments.push(...lineSegments);
    }
    
    // Add newline between lines (except last)
    if (i < lines.length - 1) {
      segments.push({ type: 'text', content: '\n' });
    }
  }
  
  return segments;
}

function parseInlineStyles(text) {
  const segments = [];
  let remaining = text;
  
  // Simple regex patterns
  const inlineCodeRegex = /`([^`]+)`/;
  const boldRegex = /\*\*([^*]+)\*\*/;
  const italicRegex = /\*([^*]+)\*/;
  
  while (remaining.length > 0) {
    let match = null;
    let type = null;
    let minIndex = remaining.length;
    
    // Find the earliest match
    const codeMatch = remaining.match(inlineCodeRegex);
    if (codeMatch && codeMatch.index < minIndex) {
      match = codeMatch;
      type = 'inlineCode';
      minIndex = codeMatch.index;
    }
    
    const boldMatch = remaining.match(boldRegex);
    if (boldMatch && boldMatch.index < minIndex) {
      match = boldMatch;
      type = 'bold';
      minIndex = boldMatch.index;
    }
    
    const italicMatch = remaining.match(italicRegex);
    if (italicMatch && italicMatch.index < minIndex && type !== 'bold') {
      match = italicMatch;
      type = 'italic';
      minIndex = italicMatch.index;
    }
    
    if (match) {
      // Add text before match
      if (match.index > 0) {
        segments.push({
          type: 'text',
          content: remaining.substring(0, match.index)
        });
      }
      
      // Add formatted segment
      segments.push({
        type,
        content: match[1]
      });
      
      // Continue with remaining text
      remaining = remaining.substring(match.index + match[0].length);
    } else {
      // No more matches, add remaining text
      if (remaining.length > 0) {
        segments.push({ type: 'text', content: remaining });
      }
      break;
    }
  }
  
  return segments.length > 0 ? segments : [{ type: 'text', content: text }];
}

export function renderSegmentToTerminal(segment, theme) {
  const { primary, secondary } = theme;
  
  switch (segment.type) {
    case 'text':
      return segment.content;
    
    case 'codeBlock':
      return `{${secondary}-fg}[${segment.language}]{/}\n{${primary}-fg}${segment.code}{/}`;
    
    case 'inlineCode':
      return `{${primary}-fg}\`${segment.content}\`{/}`;
    
    case 'bold':
      return `{bold}${segment.content}{/bold}`;
    
    case 'italic':
      return `{italic}${segment.content}{/italic}`;
    
    case 'listItem':
      return `{${secondary}-fg}•{/} ${segment.content}`;
    
    default:
      return segment.content || '';
  }
}
