# Ollama-Style Layout

## Overview

The app now features an **Ollama-inspired layout** with dark mode only and your hacker color palette!

## Key Changes

### 1. Dark Mode Only
- ✅ Removed light/dark mode toggle
- ✅ Always uses hacker aesthetic
- ✅ Cleaner, more focused experience

### 2. Ollama-Style Layout

#### Header
```
NEURAL INTERFACE          New Chat
```
- Minimal, clean design
- Simple title (no brackets)
- "New Chat" button instead of "Clear Chat"
- Text button style (subtle)

#### Messages
```
┌─────────────────────────────────────┐
│ You                              ⎘  │
│                                     │
│ Your message here...                │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ AI                               ⎘  │
│                                     │
│ AI response here...                 │
└─────────────────────────────────────┘
```
- Clean role labels: "You" and "AI" (not "USER>" or "AI>")
- Copy icon: ⎘ (cleaner than [COPY])
- More padding (20px)
- Larger spacing between messages (20px)
- Subtle borders and backgrounds
- 750px max width (slightly wider)

#### Input Box
```
╔═══════════════════════════════════════╗
║ Ask anything...                  ↑   ║
╚═══════════════════════════════════════╝
```
- Placeholder: "Ask anything..." (friendly, like Ollama)
- Larger border radius (16px)
- Solid background with glowing border
- Arrow send button (↑)
- 750px max width

## Visual Details

### Colors (Preserved)
- 🟢 Neon green text: `#00FF99`
- 🔵 Cyan borders: `#00CCFF`
- 🟣 Hot pink user: `#FF4D99`
- ⬛ Dark background: `#0D0D14`

### Spacing
- Message padding: 20px (was 16px)
- Message spacing: 20px (was 16px)
- Input padding: 16px (was 14px)
- Container radius: 16px (was 12px)
- Message radius: 12px (was 8px)

### Typography
- Role labels: "You" / "AI" (simple, clean)
- Copy button: ⎘ (copy icon)
- No timestamps visible (cleaner)
- No brackets (cleaner)

### Borders
- Message borders: 1px, 20% opacity (subtle)
- Input border: 1.5px, 50% opacity (more prominent)
- All rounded corners (modern)

### Backgrounds
- Messages: 3% cyan tint (very subtle)
- Input: Solid dark with cyan border
- Main: Deep dark blue-black

## Comparison

### Before (ChatGPT-style with toggle)
```
[ NEURAL INTERFACE v0.2.0 ]  [LIGHT]  [CLEAR]

┌─────────────────────────────────┐
│ USER> [10:30:45] [COPY]         │
│ Message...                      │
└─────────────────────────────────┘

╔═══════════════════════════════════╗
║ > ENTER COMMAND...          [SEND]║
╚═══════════════════════════════════╝
```

### After (Ollama-style, dark only)
```
NEURAL INTERFACE          New Chat

┌─────────────────────────────────┐
│ You                          ⎘  │
│                                 │
│ Message...                      │
└─────────────────────────────────┘

╔═══════════════════════════════════╗
║ Ask anything...               ↑  ║
╚═══════════════════════════════════╝
```

## Features Removed

- ❌ Light mode toggle
- ❌ Dark mode state management
- ❌ Timestamps in message header
- ❌ Bracketed labels ([COPY], [SEND], etc.)
- ❌ Terminal-style prefixes (USER>, AI>)

## Features Kept

- ✅ Hacker color palette
- ✅ Neon green text
- ✅ Cyan glowing borders
- ✅ Copy to clipboard
- ✅ Clear chat (now "New Chat")
- ✅ Auto-scroll
- ✅ Centered layout
- ✅ Floating input

## Layout Specs

### Message Container
- Width: 750px (fixed)
- Padding: 20px
- Border radius: 12px
- Border: 1px cyan (20% opacity)
- Background: Cyan tint (3% opacity)
- Spacing: 20px between messages

### Input Container
- Width: 750px (fixed)
- Padding: 14px
- Border radius: 16px
- Border: 1.5px cyan (50% opacity)
- Background: Solid dark (#141418)
- Bottom margin: 20px

### Send Button
- Width: 50px (fixed)
- Icon: ↑ (up arrow)
- Loading: ↻ (circular arrow)
- Style: Primary (cyan)

### Copy Button
- Icon: ⎘ (copy symbol)
- Style: Text (subtle)
- Padding: 6px

## User Experience

### Cleaner
- No mode switching confusion
- Simpler header
- Less visual noise
- Cleaner labels

### More Focused
- Dark mode only = consistent experience
- Centered content draws attention
- Subtle borders don't distract
- More whitespace

### Familiar
- Matches Ollama's layout
- Similar to ChatGPT
- Intuitive for users
- Modern chat interface

## Technical Changes

### Removed
```rust
- dark_mode: bool field
- ToggleDarkMode message
- Light mode conditionals
- Dark mode toggle button
```

### Simplified
```rust
- Always use dark colors
- Single theme (no switching)
- Cleaner view logic
- Less conditional rendering
```

### Updated
```rust
- Role labels: "You" / "AI"
- Copy icon: ⎘
- Placeholder: "Ask anything..."
- Button: "New Chat"
- Header: "NEURAL INTERFACE"
```

## Try It

```bash
cargo run
```

The app now opens directly in dark mode with the clean Ollama-style layout!

---

**Layout:** Ollama-inspired  
**Mode:** Dark only  
**Width:** 750px  
**Style:** Clean, minimal, focused  
**Status:** ✅ Complete
