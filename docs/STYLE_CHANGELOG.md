# Style Update - Lo-Fi Hacker Aesthetic

## What Changed

Your AI Chat app now has a **cyberpunk/hacker terminal aesthetic** that transforms the interface into a futuristic command center.

## Visual Changes

### Dark Mode (The Star of the Show)

#### Before
- Standard dark theme
- Gray text on dark background
- Rounded corners
- Emoji icons

#### After
- **Neon green terminal text** - Classic Matrix/hacker style
- **Cyan glowing borders** - Cyberpunk accent color
- **Sharp corners** - Terminal/command-line aesthetic
- **Hot pink user messages** - Neon accent for contrast
- **Bracketed labels** - `[SEND]`, `[COPY]`, `[CLEAR]`
- **Terminal-style prompts** - `USER>`, `AI>`, `> ENTER COMMAND...`
- **Header:** `[ NEURAL INTERFACE v0.2.0 ]`

### Color Scheme

```
Background:    Deep dark blue-black (#0D0D14)
Primary Text:  Neon green (#00FF99) - Terminal green
Borders:       Cyan (#00CCFF) - Cyberpunk blue
User:          Hot pink (#FF4D99) - Neon accent
AI:            Neon green (#00FF99) - Matrix style
Errors:        Hot pink/red (#FF3366) - Alert color
```

### Typography Changes

**Dark Mode:**
- `USER>` instead of "You:"
- `AI>` instead of "AI:"
- `[HH:MM:SS]` timestamp format
- `[COPY]` instead of 📋 emoji
- `[SEND]` instead of "Send"
- `[CLEAR]` instead of "Clear Chat"
- `[LIGHT]` instead of ☀️ emoji
- `> ENTER COMMAND...` placeholder
- `> SYSTEM READY. AWAITING INPUT...` empty state
- `⚠ ERROR:` prefix for errors

**Light Mode:**
- Unchanged - maintains clean, professional look

## UI Element Updates

### Message Containers
- Semi-transparent cyan background (5% opacity)
- Cyan borders (30% opacity)
- Sharp corners (no border radius)
- Holographic layered effect

### Input Field
- Dark background with cyan border
- Neon green text when typing
- Glowing cyan border when focused
- Terminal-style placeholder

### Buttons
- Bracketed text labels
- Secondary style for consistency
- Sharp corners

### Main Container
- Outer cyan border framing entire interface
- Minimal padding for edge-to-edge feel

### Header
- Centered title: `[ NEURAL INTERFACE v0.2.0 ]`
- Consistent bracketed button style

## Technical Implementation

### Custom Styles Added
```rust
- HackerContainerStyle   // Main container with cyan border
- HackerMessageStyle     // Message boxes with transparent cyan
- HackerErrorStyle       // Error boxes with red tint
- HackerInputStyle       // Input field with cyan border
```

### Custom Theme
```rust
Theme::custom(
    "hacker",
    Palette {
        background: Deep dark blue-black,
        text: Neon green,
        primary: Cyan,
        success: Neon green,
        danger: Hot pink/red,
    }
)
```

## Design Inspiration

- **Classic Terminals** - Green phosphor CRT monitors
- **The Matrix** - Cascading green code
- **Cyberpunk 2077** - Neon-lit interfaces
- **Blade Runner** - Dystopian tech aesthetic
- **Hacker Culture** - Command-line interfaces
- **Retro Computing** - 80s/90s computer terminals

## Before & After Comparison

### Header
```
Before: Prometheus  ☀️  Clear Chat
After:  [ NEURAL INTERFACE v0.2.0 ]  [LIGHT]  [CLEAR]
```

### Messages
```
Before: You: 10:30:45 📋
        Hello, how are you?

After:  USER> [10:30:45] [COPY]
        Hello, how are you?
```

### Input
```
Before: Enter your prompt...  [Send]
After:  > ENTER COMMAND...  [SEND]
```

### Empty State
```
Before: No messages yet. Start a conversation!
After:  > SYSTEM READY. AWAITING INPUT...
```

## Configuration Updates

### Window Title
```toml
Before: window_title = "AI Chat"
After:  window_title = "NEURAL INTERFACE v0.2.0"
```

### Window Size
```toml
Before: window_width = 800, window_height = 600
After:  window_width = 900, window_height = 650
```

## Features Preserved

All functionality remains intact:
- ✅ Dark mode toggle (now with hacker aesthetic)
- ✅ Auto-scroll
- ✅ Copy to clipboard
- ✅ Clear chat
- ✅ Message history
- ✅ Error handling
- ✅ Backend communication

## Light Mode

Light mode is **unchanged** and maintains a clean, professional appearance for users who prefer standard interfaces.

## Performance Impact

**Zero performance impact** - All changes are purely visual:
- Custom styles are compiled at build time
- No additional runtime overhead
- Same rendering performance

## Accessibility

- **High contrast** - Neon green on dark background
- **Clear visual hierarchy** - Distinct colors for different elements
- **Light mode available** - For users who prefer it
- **Readable text** - Large, clear fonts

## User Experience

### Benefits
- **Immersive** - Feels like using a real hacker terminal
- **Cool factor** - Cyberpunk aesthetic is visually striking
- **Focus** - Dark background reduces eye strain
- **Unique** - Stands out from standard chat interfaces

### Toggle Anytime
Users can switch between:
- **Hacker mode** - Full cyberpunk experience
- **Light mode** - Professional, clean interface

## Future Enhancements

Potential additions:
- Scanline effect overlay
- CRT monitor curvature
- Typing animation for AI responses
- Glitch effects on errors
- ASCII art decorations
- Custom monospace font
- Animated borders
- Sound effects

## Files Modified

- `src/app.rs` - Added custom styles and color scheme
- `config.toml` - Updated window title and size
- `STYLE_GUIDE.md` - New comprehensive style documentation
- `README.md` - Added style guide reference

## How to Use

Just run the app and click the dark mode toggle:
```bash
cargo run
```

Then click `🌙` to enter **NEURAL INTERFACE** mode! 🚀

---

**Style Version:** 1.0  
**Theme Name:** "Neural Interface"  
**Aesthetic:** Lo-fi hacker / Cyberpunk terminal  
**Status:** ✅ Fully implemented and tested
