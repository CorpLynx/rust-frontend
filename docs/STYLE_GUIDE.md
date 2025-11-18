# Lo-Fi Hacker Aesthetic Style Guide

## Design Philosophy

The app now features a **lo-fi hacker/cyberpunk aesthetic** inspired by classic terminal interfaces, retro computing, and cyberpunk culture.

## Color Palette

### Dark Mode (Hacker Theme)
- **Background:** Deep dark blue-black `rgb(0.05, 0.05, 0.08)`
- **Primary Text:** Neon green `rgb(0.0, 1.0, 0.6)` - Classic terminal green
- **Secondary Text:** Dimmer green `rgb(0.0, 0.7, 0.5)`
- **Accent/Borders:** Cyan `rgb(0.0, 0.8, 1.0)` - Cyberpunk blue
- **User Messages:** Hot pink `rgb(1.0, 0.3, 0.6)` - Neon accent
- **AI Messages:** Neon green `rgb(0.0, 1.0, 0.6)` - Matrix-style
- **Errors:** Hot pink/red `rgb(1.0, 0.2, 0.4)` - Alert color

### Light Mode (Standard)
- Clean, minimal design with standard colors
- Maintains readability and professionalism

## Typography

### Text Style
- **Monospace feel** - Terminal-inspired text
- **All caps for labels** in dark mode (USER>, AI>, [SEND], etc.)
- **Brackets for emphasis** - [COPY], [CLEAR], [LIGHT]
- **Angle brackets for prompts** - `> ENTER COMMAND...`

### Text Prefixes
- **Dark Mode:**
  - User: `USER>`
  - AI: `AI>`
  - Timestamps: `[HH:MM:SS]`
  - Errors: `⚠ ERROR:`
  - Empty state: `> SYSTEM READY. AWAITING INPUT...`

- **Light Mode:**
  - User: `You:`
  - AI: `AI:`
  - Standard formatting

## UI Elements

### Borders
- **Sharp corners** (no border radius) - Terminal aesthetic
- **Thin cyan borders** on containers and inputs
- **Glowing effect** on focused elements (thicker border)

### Buttons
- **Bracketed text** in dark mode: `[SEND]`, `[COPY]`, `[CLEAR]`
- **Secondary style** for most buttons
- **Primary style** for send button

### Input Field
- **Dark background** with cyan border
- **Neon green text** when typing
- **Glowing border** when focused
- **Terminal-style placeholder** text

### Message Containers
- **Semi-transparent cyan background** `rgba(0.0, 0.8, 1.0, 0.05)`
- **Cyan border** with low opacity
- **Compact padding** for dense information display

### Main Container
- **Outer cyan border** framing the entire interface
- **Minimal padding** for edge-to-edge feel

## Visual Effects

### Transparency
- Message backgrounds: 5% opacity
- Borders: 30% opacity
- Creates layered, holographic effect

### Contrast
- High contrast between text and background
- Neon colors pop against dark background
- Easy to read for extended periods

### Spacing
- Tighter spacing in dark mode (8px vs 10px)
- Dense information layout
- Terminal-like compactness

## Header

### Dark Mode
```
[ NEURAL INTERFACE v0.2.0 ]  [LIGHT]  [CLEAR]
```

### Light Mode
```
Prometheus  🌙  Clear Chat
```

## Empty State

### Dark Mode
```
> SYSTEM READY. AWAITING INPUT...
```

### Light Mode
```
No messages yet. Start a conversation!
```

## Error Messages

### Dark Mode
```
⚠ ERROR: Network error: Is the backend server running...
```
- Hot pink text
- Red-tinted border
- Warning symbol prefix

## Theme Toggle

- **Dark Mode Button:** `[LIGHT]` - Switches to light mode
- **Light Mode Button:** `🌙` - Switches to dark mode

## Inspiration

This design draws from:
- **Classic terminals** - Green phosphor CRT monitors
- **Cyberpunk aesthetics** - Neon colors, dark backgrounds
- **Hacker culture** - Command-line interfaces, technical feel
- **Retro computing** - 80s/90s computer interfaces
- **The Matrix** - Green cascading code aesthetic
- **Blade Runner** - Neon-lit dystopian tech

## Design Principles

1. **Functionality First** - Style enhances, doesn't hinder
2. **High Contrast** - Easy to read in any lighting
3. **Minimal Distractions** - Focus on content
4. **Terminal Authenticity** - Feels like a real command interface
5. **Cyberpunk Cool** - Looks futuristic and edgy

## Color Psychology

- **Neon Green** - Technology, terminals, "online" status
- **Cyan** - Digital, futuristic, cool technology
- **Hot Pink** - Energy, alerts, user interaction
- **Dark Background** - Focus, immersion, reduced eye strain

## Accessibility

- High contrast ratios for readability
- Clear visual hierarchy
- Distinct colors for different message types
- Light mode available for those who prefer it

## Future Enhancements

Potential additions to the aesthetic:
- Scanline effect overlay
- CRT monitor curvature simulation
- Typing animation for AI responses
- Glitch effects on errors
- ASCII art decorations
- Custom monospace font (e.g., Fira Code, JetBrains Mono)
- Animated borders or pulsing effects
- Sound effects (keyboard clicks, beeps)

---

**Style Version:** 1.0  
**Theme Name:** "Neural Interface"  
**Inspired by:** Cyberpunk, terminals, and hacker culture
