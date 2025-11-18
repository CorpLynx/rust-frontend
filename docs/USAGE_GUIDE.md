# Usage Guide - New Features

## Quick Start

After running `cargo run`, you'll see the Prometheus interface with new controls in the header.

## Interface Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Prometheus                 🌙  [Clear Chat]                │  ← Header with new buttons
├─────────────────────────────────────────────────────────────┤
│  [Error messages appear here if any]                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  You: 10:30:45 📋                                           │  ← Copy button
│  Hello, how are you?                                        │
│                                                              │
│  AI: 10:30:47 📋                                            │  ← Copy button
│  I'm doing well, thank you! How can I help you today?      │
│                                                              │
│  [Chat messages scroll here]                                │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│  [Enter your prompt...]                          [Send]     │  ← Input area
└─────────────────────────────────────────────────────────────┘
```

## Feature Walkthrough

### 1. Sending Messages
1. Type your message in the input field at the bottom
2. Press **Enter** or click **Send**
3. Your message appears in the chat
4. The chat automatically scrolls to show your message
5. Wait for the AI response
6. The chat automatically scrolls to show the response

### 2. Dark Mode
**Light Mode (Default):**
- White background
- Dark text
- Shows 🌙 (moon) icon

**To Switch to Dark Mode:**
1. Click the 🌙 button in the header
2. Interface switches to dark theme
3. Button changes to ☀️ (sun) icon

**To Switch Back to Light Mode:**
1. Click the ☀️ button
2. Interface switches back to light theme

**Benefits:**
- Easier on the eyes in low-light conditions
- Reduces screen glare
- Modern, sleek appearance

### 3. Copying Messages
**To Copy a Message:**
1. Find the message you want to copy
2. Click the 📋 button next to the timestamp
3. The message content is copied to your clipboard
4. Paste it anywhere using:
   - **Windows/Linux:** Ctrl+V
   - **macOS:** Cmd+V

**Use Cases:**
- Save AI responses for later
- Share interesting responses with others
- Copy prompts to reuse them
- Extract information from the conversation

### 4. Clearing Chat
**To Clear All Messages:**
1. Click the **Clear Chat** button in the header
2. All messages are immediately removed
3. The chat history file is cleared
4. You'll see "No messages yet. Start a conversation!"

**Important Notes:**
- ⚠️ This action cannot be undone
- All messages are permanently deleted
- The `chat_history.json` file is cleared
- Consider copying important messages before clearing

## Keyboard Shortcuts

Currently supported:
- **Enter** - Send message (when input field is focused)

Future shortcuts (not yet implemented):
- Ctrl+D - Toggle dark mode
- Ctrl+L - Clear chat
- Ctrl+C - Copy selected message

## Tips & Tricks

### Efficient Workflow
1. **Keep Dark Mode On** - If you work in low-light conditions
2. **Copy Important Responses** - Use 📋 to save valuable AI outputs
3. **Clear Regularly** - Keep your chat clean by clearing old conversations
4. **Auto-scroll** - No need to manually scroll, it happens automatically

### Best Practices
- Copy important information before clearing chat
- Use dark mode to reduce eye strain during long sessions
- Clear chat when starting a new topic or conversation
- The chat history persists between sessions (until you clear it)

## Troubleshooting

### Copy Button Not Working
- **Issue:** Clicking 📋 doesn't copy the message
- **Solution:** Ensure clipboard permissions are granted to the application
- **macOS:** May need to grant accessibility permissions
- **Linux:** Ensure X11 or Wayland clipboard is available

### Dark Mode Looks Wrong
- **Issue:** Colors are hard to read in dark mode
- **Solution:** This uses Iced's default dark theme. Custom themes can be added in future updates

### Auto-scroll Not Working
- **Issue:** Chat doesn't scroll to new messages
- **Solution:** This is a known limitation if you manually scroll up. Sending a new message will re-enable auto-scroll

### Clear Chat Doesn't Work
- **Issue:** Messages don't clear
- **Solution:** Check file permissions for `chat_history.json`

## Configuration

Currently, the new features don't have configuration options. Future enhancements may include:

```toml
[ui]
font_size = 16
max_chat_history = 1000
dark_mode_default = false        # Future: Start in dark mode
confirm_clear = true              # Future: Show confirmation dialog
auto_scroll = true                # Future: Disable auto-scroll
show_copy_buttons = true          # Future: Hide copy buttons
```

## Accessibility

### Current Features
- Large, clear emoji icons
- High contrast in both light and dark modes
- Keyboard support for sending messages

### Future Improvements
- Screen reader support
- Keyboard navigation for all buttons
- Customizable font sizes
- High contrast mode
- Text labels for icon buttons

## Performance

All new features are optimized for performance:
- **Dark Mode:** Instant theme switching, no lag
- **Auto-scroll:** Smooth scrolling animation
- **Copy:** Fast clipboard access, non-blocking
- **Clear:** Instant message removal

## Privacy & Security

- **Clipboard:** Only copies when you click the button
- **Chat History:** Stored locally in `chat_history.json`
- **No Telemetry:** No data is sent anywhere except to your configured backend
- **Clear Chat:** Permanently deletes local history

## Getting Help

If you encounter issues:
1. Check `logs/error.log` for detailed error messages
2. Verify your `config.toml` settings
3. Ensure all dependencies are installed
4. Try rebuilding with `cargo clean && cargo build`

---

**Enjoy your enhanced AI Chat experience!** 🚀
