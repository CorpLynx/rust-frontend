# Archived Iced GUI

This directory contains the archived Iced-based graphical user interface for Prometheus. This GUI is no longer actively developed but is preserved for reference and potential future use.

## Status: Archived

⚠️ **This application is archived and not actively maintained.**

Active development has moved to:
- **Prometheus CLI** - Terminal-based interface (see main [README.md](../README.md))
- **Prometheus Tauri** - Modern desktop application with web UI (see [src-tauri/](../src-tauri/))

## Why Archived?

The Iced GUI was the original interface for Prometheus, featuring a cyberpunk-inspired design with neon green text and a lo-fi hacker aesthetic. Development shifted to the CLI and Tauri applications for several reasons:

1. **Better Cross-Platform Support** - Tauri provides more consistent behavior across platforms
2. **Modern Web Technologies** - Tauri's web-based UI is easier to style and maintain
3. **Server Environments** - CLI is better suited for headless servers and SSH sessions
4. **Development Velocity** - Web UI development is faster than native GUI frameworks
5. **Resource Usage** - CLI has minimal footprint; Tauri has better performance than Iced

## What's Preserved

This archive contains:

- **Complete source code** - All Iced GUI modules and logic
- **Assets** - Fonts and other resources
- **Documentation** - Original README and feature descriptions
- **Git history** - Full commit history via `git log --follow`
- **Build configuration** - Standalone Cargo.toml for independent builds

## Building the Archived GUI

The Iced GUI can still be built and run as a standalone application.

### Prerequisites

- Rust 1.70 or later
- Iced 0.12 dependencies (automatically downloaded by Cargo)
- Ollama running locally or accessible via network

### Build Instructions

```bash
# Navigate to the archive directory
cd archived-iced-gui

# Build the application
cargo build --release

# Run the application
cargo run --release
```

The binary will be created at `target/release/prometheus-gui`.

### Configuration

The Iced GUI uses the same `config.toml` file as the CLI and Tauri applications. Create or edit `../config.toml` in the repository root:

```toml
[app]
window_title = "Prometheus"
window_width = 900.0
window_height = 650.0

[backend]
url = "http://localhost:11434"
ollama_url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
```

## Features

The archived Iced GUI includes:

- ✅ **Chat Interface** - Scrollable chat history with cyberpunk styling
- ✅ **Markdown Rendering** - Syntax-highlighted code blocks
- ✅ **Dark Mode** - Lo-fi hacker aesthetic with neon green and cyan
- ✅ **Conversation Management** - Auto-save and history loading
- ✅ **Search Engine** - Full-text search across conversations
- ✅ **Copy to Clipboard** - Copy messages with one click
- ✅ **Ollama Integration** - Direct communication with Ollama backend

## Documentation

For detailed documentation about the Iced GUI:

- **[README-ICED-GUI.md](README-ICED-GUI.md)** - Complete original documentation
- **Features** - Detailed feature descriptions
- **Architecture** - Technical implementation details
- **Usage Guide** - How to use the GUI

## Known Issues

As an archived project, known issues will not be actively fixed:

- Some Iced 0.12 API deprecation warnings
- Potential compatibility issues with newer Rust versions
- Limited testing on recent operating system versions

## Restoring Active Development

If you want to restore active development of the Iced GUI:

1. **Create a branch:**
   ```bash
   git checkout -b restore-iced-gui
   ```

2. **Move back to main source:**
   ```bash
   git mv archived-iced-gui/src/* src/
   git mv archived-iced-gui/Cargo.toml ./
   ```

3. **Update workspace:**
   Edit root `Cargo.toml` to make Iced GUI the primary package

4. **Test thoroughly:**
   ```bash
   cargo build
   cargo test
   ```

## Git History

All git history is preserved. You can view the evolution of any file:

```bash
# View history of the main app file
git log --follow archived-iced-gui/src/app.rs

# View who authored specific lines
git blame archived-iced-gui/src/app.rs

# View the file at a specific point in time
git show pre-reorganization:src/app.rs
```

## Comparison with Active Applications

| Feature | Iced GUI (Archived) | CLI | Tauri |
|---------|---------------------|-----|-------|
| Status | Archived | ✅ Active | ✅ Active |
| UI Framework | Iced 0.12 | Terminal | Web (HTML/CSS/JS) |
| Startup Time | ~1-2s | < 500ms | ~1-2s |
| Memory Usage | ~100-200MB | < 50MB | ~100-200MB |
| Cross-Platform | Good | Excellent | Excellent |
| Styling | Native Iced | ANSI colors | CSS |
| Development | Rust only | Rust only | Rust + Web |
| Server Deployment | ❌ | ✅ | ❌ |
| Remote SSH | ❌ | ✅ | ❌ |

## Contributing

Since this is an archived project, contributions are not actively sought. However, if you find critical bugs or security issues, please:

1. Open an issue on GitHub
2. Tag it with `archived-iced-gui`
3. Explain the impact and severity

## License

Same as the main Prometheus project. See the root LICENSE file.

## Questions?

For questions about:
- **Using the archived GUI** - See [README-ICED-GUI.md](README-ICED-GUI.md)
- **Repository structure** - See [../STRUCTURE.md](../STRUCTURE.md)
- **Active applications** - See main [../README.md](../README.md)
- **Why it was archived** - See "Why Archived?" section above

## Acknowledgments

Thanks to all contributors who worked on the Iced GUI. Your work is preserved here and served as the foundation for the current CLI and Tauri applications.

---

**Archived:** November 2024  
**Last Active Version:** v0.2.0  
**Reason:** Development shifted to CLI and Tauri applications
