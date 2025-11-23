# Repository Structure

This document explains the organization of the Prometheus repository and the rationale behind its structure.

## Overview

Prometheus is organized as a Cargo workspace containing three independent applications:

1. **prometheus-cli** - Terminal-based CLI interface (primary, actively developed)
2. **src-tauri** - Desktop application with web UI (primary, actively developed)
3. **archived-iced-gui** - Legacy Iced-based GUI (archived, not actively developed)

## Workspace Organization

```
prometheus/                    # Cargo workspace root
├── Cargo.toml                 # Workspace definition
├── README.md                  # Main documentation (CLI + Tauri focused)
├── STRUCTURE.md               # This file
├── config.toml                # Shared configuration file
├── conversations/             # Shared conversation storage
├── personas.json              # Shared persona definitions
│
├── prometheus-cli/            # CLI package (standalone)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── commands.rs
│       ├── error.rs
│       ├── markdown_renderer.rs
│       ├── streaming.rs
│       ├── terminal.rs
│       ├── backend.rs
│       ├── config.rs
│       └── conversation.rs
│
├── src-tauri/                 # Tauri package (standalone)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands.rs
│       ├── persona.rs
│       ├── config.rs
│       ├── conversation.rs
│       ├── markdown.rs
│       └── network/
│
├── ui/                        # Tauri web UI
│   ├── index.html
│   ├── app.js
│   └── styles.css
│
└── archived-iced-gui/         # Archived Iced GUI package
    ├── README.md              # Archive explanation
    ├── README-ICED-GUI.md     # Original Iced GUI documentation
    ├── Cargo.toml             # Iced dependencies
    ├── assets/                # Iced fonts
    └── src/
        ├── main.rs
        ├── app.rs
        ├── lib.rs
        ├── backend.rs
        ├── config.rs
        ├── conversation.rs
        ├── markdown.rs
        ├── icons.rs
        └── search/
```

## Rationale

### Why a Workspace?

The workspace structure provides several benefits:

1. **Clear Separation** - Each application is a standalone package with its own dependencies
2. **Independent Builds** - Build only what you need: `cargo build -p prometheus-cli`
3. **Shared Configuration** - Common workspace dependencies ensure version consistency
4. **Git History** - All applications share the same repository and history
5. **Flexibility** - Easy to add new packages or remove old ones

### Why Archive the Iced GUI?

The Iced GUI was the original interface for Prometheus, but development has shifted to:

1. **CLI** - Better for server environments, SSH sessions, and automation
2. **Tauri** - Modern web-based UI with better cross-platform support and richer features

The Iced GUI is preserved in the archive because:

- It represents significant development effort
- It may be useful for reference or future restoration
- Git history is preserved for all files
- It can still be built and run if needed

### Why Two Active Applications?

The CLI and Tauri applications serve different use cases:

**CLI is ideal for:**
- Server environments without GUI
- Remote SSH sessions
- Scripting and automation
- Minimal resource usage
- Fast startup times

**Tauri is ideal for:**
- Desktop users who prefer graphical interfaces
- Users who want conversation browsing and search
- Persona management with visual feedback
- Settings configuration without editing files
- Rich markdown rendering with HTML

## Building Applications

### Build Everything

```bash
# Build all workspace members
cargo build --workspace
```

### Build Specific Applications

```bash
# Build only the CLI
cargo build -p prometheus-cli

# Build only Tauri (from src-tauri directory)
cd src-tauri && cargo build

# Build the archived Iced GUI (from archive directory)
cd archived-iced-gui && cargo build
```

### Run Applications

```bash
# Run the CLI
cargo run -p prometheus-cli

# Run Tauri in development mode
cd src-tauri && cargo tauri dev

# Run the archived Iced GUI
cd archived-iced-gui && cargo run
```

## Shared Resources

### Configuration File (`config.toml`)

All three applications can read from the same `config.toml` file in the repository root. Each application uses the fields relevant to it and ignores others.

**Shared fields:**
- `backend.url` / `backend.ollama_url` - Backend server URL
- `backend.timeout_seconds` - Request timeout

**CLI-specific fields:**
- None currently (uses shared fields)

**Tauri-specific fields:**
- `app.window_title`, `app.window_width`, `app.window_height` - Window settings
- `ui.theme` - UI theme selection

**Iced GUI-specific fields:**
- Same as Tauri (window and UI settings)

### Conversation Storage (`conversations/`)

All applications save conversations to the same `conversations/` directory using a compatible JSON format. This allows:

- Starting a conversation in the CLI and continuing in Tauri
- Browsing CLI conversations in the Tauri UI
- Consistent conversation history across interfaces

### Persona Definitions (`personas.json`)

The Tauri application uses `personas.json` for persona management. The CLI currently doesn't use personas but could be extended to support them.

## Dependency Isolation

Each package has independent dependencies:

**CLI dependencies:**
- `crossterm` - Terminal manipulation
- `clap` - Command-line argument parsing
- `termimad` - Terminal markdown rendering
- Common: `reqwest`, `serde`, `tokio`, `config`

**Tauri dependencies:**
- `tauri` - Desktop application framework
- Common: `reqwest`, `serde`, `tokio`, `config`

**Iced GUI dependencies (archived):**
- `iced` - GUI framework
- `arboard` - Clipboard access
- `syntect` - Syntax highlighting
- Common: `reqwest`, `serde`, `tokio`, `config`

Building the CLI does **not** download Iced or Tauri dependencies, and vice versa.

## Git History Preservation

All file moves during the reorganization used `git mv` to preserve history:

```bash
# View history of a moved file
git log --follow prometheus-cli/src/main.rs

# View history of archived file
git log --follow archived-iced-gui/src/app.rs

# Check authorship
git blame prometheus-cli/src/app.rs
```

The reorganization is marked with git tags:
- `pre-reorganization` - State before reorganization
- `v0.3.0-reorganized` - State after reorganization

## Adding New Packages

To add a new package to the workspace:

1. Create the package directory: `mkdir new-package`
2. Create `new-package/Cargo.toml` with package metadata
3. Add to workspace members in root `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       "prometheus-cli",
       "src-tauri",
       "archived-iced-gui",
       "new-package",  # Add here
   ]
   ```
4. Build: `cargo build -p new-package`

## Removing Archived Packages

If you want to completely remove the archived Iced GUI:

1. Remove from workspace members in root `Cargo.toml`
2. Delete the directory: `rm -rf archived-iced-gui`
3. Commit the changes

The git history will still be preserved and accessible via `git log`.

## Documentation

- **README.md** - Main documentation covering CLI and Tauri
- **archived-iced-gui/README.md** - Archive explanation and build instructions
- **archived-iced-gui/README-ICED-GUI.md** - Original Iced GUI documentation
- **docs/** - Additional documentation (architecture, changelog, etc.)

## Questions?

For questions about the repository structure or organization:

1. Check this document first
2. Review the main README.md
3. Check the docs/ directory for additional documentation
4. Open an issue on GitHub

## Future Considerations

### Potential Improvements

- **Shared library crate** - Extract common code (config, conversation, backend) into a shared library
- **Feature flags** - Use Cargo features to enable/disable functionality
- **Monorepo tools** - Consider tools like `cargo-workspaces` for release management

### Why Not a Shared Library Now?

Currently, each application has its own implementations of common functionality (config, conversation, backend). While there is some code duplication, each implementation is tailored to its specific use case:

- CLI uses synchronous file I/O and terminal-specific error handling
- Tauri uses async Tauri commands and web-specific serialization
- Iced GUI uses Iced-specific async patterns

Creating a shared library would require:
- Abstracting over different async runtimes
- Supporting multiple error handling patterns
- Maintaining backward compatibility across all applications

This may be worthwhile in the future as the codebase matures, but for now, the duplication is manageable and allows each application to evolve independently.
