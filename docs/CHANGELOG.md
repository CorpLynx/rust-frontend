# Changelog

All notable changes to the Rust AI Chat Frontend project.

## [0.2.0] - Current Session

### Added
- **Dark Mode Toggle** - Switch between light and dark themes with 🌙/☀️ button in header
- **Auto-scroll** - Chat automatically scrolls to the latest message when sending or receiving
- **Copy to Clipboard** - Copy any message content with the 📋 button next to each message
- **Clear Chat Button** - Clear all chat history with one click from the header
- **New Documentation:**
  - `FEATURES.md` - Detailed feature descriptions
  - `USAGE_GUIDE.md` - User guide for new features
  - `CHANGELOG.md` - This file

### Changed
- Updated `view()` function signature to use explicit lifetime: `Element<'_, Message>`
- Enhanced header with new control buttons
- Improved message display with copy buttons
- Updated README with new features and documentation links

### Fixed
- Removed unused `LoadHistory` message variant (fixed compiler warning)
- Fixed lifetime syntax warning in `view()` function
- All compiler warnings resolved

### Dependencies
- Added `arboard` v3.6.1 for clipboard functionality

## [0.1.0] - Previous Session

### Added
- Initial project setup with Rust and Iced GUI framework
- Configuration system with `config.toml`
- Chat interface with scrollable history
- Text input field with Enter key support
- Send button with loading states
- Backend integration via HTTP POST requests
- Flexible JSON response parsing
- Chat history persistence to `chat_history.json`
- Automatic history loading on startup
- Timestamp generation for messages
- Error handling and logging to `logs/error.log`
- User-friendly error messages
- Configurable window size, backend URL, and UI settings

### Dependencies
- `iced` v0.12 - GUI framework
- `reqwest` v0.11 - HTTP client
- `serde` / `serde_json` - Serialization
- `tokio` - Async runtime
- `config` v0.14 - Configuration management
- `chrono` v0.4 - Timestamps
- `log` / `env_logger` - Logging
- `anyhow` v1.0 - Error handling

---

## Version History

### Version 0.2.0 - UI Enhancements
Focus: User experience improvements with dark mode, clipboard, and better navigation

### Version 0.1.0 - Initial Release
Focus: Core functionality with chat interface and backend integration

---

## Upgrade Guide

### From 0.1.0 to 0.2.0

**No Breaking Changes** - This is a backward-compatible update.

**Steps:**
1. Pull the latest code
2. Run `cargo build` to download new dependencies
3. Run `cargo run` to start the application
4. Enjoy the new features!

**New Dependencies:**
- `arboard` will be automatically downloaded during build

**Configuration:**
- No changes to `config.toml` required
- All existing settings remain compatible

**Data:**
- Existing `chat_history.json` files are fully compatible
- No migration needed

---

## Future Roadmap

### Version 0.3.0 (Planned)
- Theme persistence (save dark mode preference)
- Confirmation dialog for clear chat
- Keyboard shortcuts (Ctrl+D for dark mode, etc.)
- Message editing capability
- Message deletion (individual messages)

### Version 0.4.0 (Planned)
- Markdown rendering support
- Syntax highlighting for code blocks
- Export conversation to file
- Search functionality
- Multiple conversation management

### Version 0.5.0 (Planned)
- Streaming responses support
- Multiple backend endpoints
- Model selection dropdown
- API key authentication
- Custom theme editor

---

## Contributing

When adding new features, please:
1. Update this CHANGELOG.md
2. Add documentation to FEATURES.md if applicable
3. Update USAGE_GUIDE.md with usage instructions
4. Update README.md with feature list
5. Increment version number appropriately

## Versioning

This project follows [Semantic Versioning](https://semver.org/):
- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backward-compatible manner
- **PATCH** version for backward-compatible bug fixes

---

**Last Updated:** Current Session
