# Visual Reference - Hacker Aesthetic

## Dark Mode Preview (Text Representation)

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║  [ NEURAL INTERFACE v0.2.0 ]      [LIGHT]      [CLEAR]      ║
║                                                               ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  ┌───────────────────────────────────────────────────────┐  ║
║  │ USER> [10:30:45] [COPY]                               │  ║
║  │ Hello, how are you?                                   │  ║
║  └───────────────────────────────────────────────────────┘  ║
║                                                               ║
║  ┌───────────────────────────────────────────────────────┐  ║
║  │ AI> [10:30:47] [COPY]                                 │  ║
║  │ I'm doing well, thank you! How can I help you today? │  ║
║  └───────────────────────────────────────────────────────┘  ║
║                                                               ║
║  ┌───────────────────────────────────────────────────────┐  ║
║  │ USER> [10:31:02] [COPY]                               │  ║
║  │ What's the weather like?                              │  ║
║  └───────────────────────────────────────────────────────┘  ║
║                                                               ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  > ENTER COMMAND...                              [SEND]      ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

## Color Legend

### Dark Mode Colors

**Background:**
```
████████  Deep Dark Blue-Black (#0D0D14)
```

**Primary Text (Neon Green):**
```
████████  #00FF99 - Terminal green, AI messages
```

**User Messages (Hot Pink):**
```
████████  #FF4D99 - Neon pink accent
```

**Borders (Cyan):**
```
████████  #00CCFF - Cyberpunk blue
```

**Muted Text (Dim Green):**
```
████████  #00B377 - Timestamps, secondary info
```

**Error (Hot Red):**
```
████████  #FF3366 - Alert color
```

## Typography Examples

### Header
```
Light Mode:  AI Chat Interface
Dark Mode:   [ NEURAL INTERFACE v0.2.0 ]
```

### Message Prefixes
```
Light Mode:  You:  |  AI:
Dark Mode:   USER> |  AI>
```

### Timestamps
```
Light Mode:  10:30:45
Dark Mode:   [10:30:45]
```

### Buttons
```
Light Mode:  🌙  |  Clear Chat  |  📋
Dark Mode:   [LIGHT]  |  [CLEAR]  |  [COPY]
```

### Input Placeholder
```
Light Mode:  Enter your prompt...
Dark Mode:   > ENTER COMMAND...
```

### Send Button
```
Light Mode:  Send  |  Sending...
Dark Mode:   [SEND]  |  [TRANSMITTING...]
```

### Empty State
```
Light Mode:  No messages yet. Start a conversation!
Dark Mode:   > SYSTEM READY. AWAITING INPUT...
```

### Error Messages
```
Light Mode:  Error: Network error...
Dark Mode:   ⚠ ERROR: Network error...
```

## UI Element Styles

### Message Container (Dark Mode)
```
┌─────────────────────────────────┐  ← Cyan border (1px)
│ USER> [10:30:45] [COPY]         │  ← Hot pink + dim green + button
│ Message content in neon green   │  ← Neon green text
└─────────────────────────────────┘
  ↑ Semi-transparent cyan background (5% opacity)
```

### Input Field (Dark Mode)
```
┌─────────────────────────────────┐  ← Cyan border (1px)
│ > ENTER COMMAND...              │  ← Dim green placeholder
└─────────────────────────────────┘
  ↑ Dark background (#141418)

When focused:
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓  ← Thicker cyan border (2px)
┃ > Your text here                ┃  ← Neon green text
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

### Error Container (Dark Mode)
```
┌─────────────────────────────────┐  ← Hot pink border
│ ⚠ ERROR: Connection failed      │  ← Hot pink text
└─────────────────────────────────┘
  ↑ Red-tinted background (10% opacity)
```

### Main Container (Dark Mode)
```
╔═══════════════════════════════════╗  ← Outer cyan border
║                                   ║
║  [ NEURAL INTERFACE v0.2.0 ]     ║  ← Neon green header
║                                   ║
║  ┌─────────────────────────────┐ ║
║  │ Messages...                 │ ║
║  └─────────────────────────────┘ ║
║                                   ║
║  > ENTER COMMAND...    [SEND]    ║
║                                   ║
╚═══════════════════════════════════╝
```

## Comparison Chart

| Element | Light Mode | Dark Mode |
|---------|-----------|-----------|
| Background | Light gray | Deep dark blue-black |
| Text | Dark gray | Neon green |
| User prefix | "You:" | "USER>" |
| AI prefix | "AI:" | "AI>" |
| Timestamp | Plain | Bracketed [HH:MM:SS] |
| Copy button | 📋 emoji | [COPY] text |
| Clear button | "Clear Chat" | [CLEAR] |
| Theme toggle | 🌙 emoji | [LIGHT] |
| Send button | "Send" | [SEND] |
| Placeholder | "Enter your prompt..." | "> ENTER COMMAND..." |
| Empty state | Friendly message | Terminal prompt |
| Error prefix | "Error:" | "⚠ ERROR:" |
| Borders | Rounded | Sharp corners |
| Border color | Gray | Cyan |

## Aesthetic Keywords

**Dark Mode:**
- Cyberpunk
- Terminal
- Hacker
- Matrix
- Neon
- Retro-futuristic
- Command-line
- Lo-fi
- Dystopian tech

**Light Mode:**
- Clean
- Professional
- Modern
- Minimal
- Friendly

## Mood Board

**Dark Mode Inspiration:**
```
🟢 Terminal green text
🔵 Cyan neon lights
🟣 Hot pink accents
⬛ Deep dark backgrounds
💻 Command-line interfaces
🌃 Cyberpunk cityscapes
🎮 Retro gaming aesthetics
📟 Old-school CRT monitors
```

## Font Characteristics

**Ideal fonts for this aesthetic:**
- Monospace (terminal feel)
- Clean, readable
- Slightly futuristic
- Examples: Fira Code, JetBrains Mono, Source Code Pro, Courier New

**Current:** System default (works well)
**Future:** Could add custom monospace font

## Animation Ideas (Future)

- **Typing effect** - Characters appear one by one
- **Scanlines** - Horizontal lines moving down screen
- **Glitch effect** - Brief distortion on errors
- **Pulsing borders** - Subtle glow animation
- **Cursor blink** - Classic terminal cursor
- **Matrix rain** - Background effect (subtle)

## Sound Effects (Future)

- **Keyboard clicks** - When typing
- **Send beep** - When sending message
- **Receive ping** - When AI responds
- **Error buzz** - On errors
- **Startup sound** - When app launches

## Accessibility Notes

✅ **High Contrast** - Neon green on dark background
✅ **Clear Hierarchy** - Distinct colors for different elements
✅ **Light Mode Option** - For users who prefer it
✅ **Readable Text** - Large, clear fonts
✅ **Color Coding** - User (pink) vs AI (green)

⚠ **Considerations:**
- Some users may find neon colors too bright
- Light mode provides alternative
- Could add brightness/contrast controls in future

## Platform Consistency

**macOS:** Looks great with native window chrome
**Windows:** Sharp corners match Windows 11 aesthetic
**Linux:** Fits well with most desktop environments

---

**Visual Style:** Lo-fi hacker / Cyberpunk terminal  
**Primary Colors:** Neon green, cyan, hot pink  
**Mood:** Futuristic, technical, immersive  
**Inspiration:** The Matrix, Blade Runner, classic terminals
