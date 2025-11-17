# Project Files Overview

## 📂 Complete File Listing

### Source Code
| File | Purpose | Lines |
|------|---------|-------|
| `src/main.rs` | Application entry point and initialization | ~40 |
| `src/app.rs` | Main application logic, GUI, and features | ~400 |
| `src/config.rs` | Configuration management | ~60 |

### Configuration
| File | Purpose |
|------|---------|
| `config.toml` | Application settings (window, backend, UI) |
| `Cargo.toml` | Rust project manifest and dependencies |
| `Cargo.lock` | Locked dependency versions |

### Documentation
| File | Purpose | Audience |
|------|---------|----------|
| `README.md` | Main documentation | All users |
| `QUICK_REFERENCE.md` | Quick reference card | End users |
| `FEATURES.md` | Feature descriptions | End users |
| `USAGE_GUIDE.md` | Step-by-step guide | End users |
| `ARCHITECTURE.md` | Technical architecture | Developers |
| `CHANGELOG.md` | Version history | All users |
| `SUMMARY.md` | Latest changes | All users |
| `BUILD_STATUS.md` | Build information | Developers |
| `FILES.md` | This file | Developers |

### Generated Files (Not in Git)
| File/Directory | Purpose |
|----------------|---------|
| `target/` | Build artifacts and compiled binaries |
| `logs/` | Error logs directory |
| `logs/error.log` | Detailed error logging |
| `chat_history.json` | Saved conversation history |

### Git Configuration
| File | Purpose |
|------|---------|
| `.gitignore` | Files to exclude from version control |
| `.git/` | Git repository data |
| `.github/` | GitHub-specific configuration |
| `.vscode/` | VS Code workspace settings |

## 📊 File Statistics

### Source Code
- **Total Lines:** ~500 lines of Rust code
- **Files:** 3 source files
- **Modules:** 3 (main, app, config)

### Documentation
- **Total Files:** 9 documentation files
- **Total Words:** ~15,000 words
- **Coverage:** Complete feature and usage documentation

### Dependencies
- **Direct Dependencies:** 12 crates
- **Total Dependencies:** ~100+ crates (including transitive)

## 🗂️ File Organization

```
rust-frontend/
│
├── 📁 Source Code
│   └── src/
│       ├── main.rs      (Entry point)
│       ├── app.rs       (Main logic)
│       └── config.rs    (Configuration)
│
├── ⚙️ Configuration
│   ├── config.toml      (App settings)
│   ├── Cargo.toml       (Project manifest)
│   └── Cargo.lock       (Dependency lock)
│
├── 📚 Documentation
│   ├── README.md        (Main docs)
│   ├── QUICK_REFERENCE.md
│   ├── FEATURES.md
│   ├── USAGE_GUIDE.md
│   ├── ARCHITECTURE.md
│   ├── CHANGELOG.md
│   ├── SUMMARY.md
│   ├── BUILD_STATUS.md
│   └── FILES.md
│
├── 🔧 Build Output (Generated)
│   ├── target/          (Compiled binaries)
│   ├── logs/            (Error logs)
│   └── chat_history.json
│
└── 🔨 Development
    ├── .git/            (Version control)
    ├── .github/         (GitHub config)
    ├── .vscode/         (Editor config)
    └── .gitignore       (Git ignore rules)
```

## 📝 File Descriptions

### src/main.rs
**Purpose:** Application entry point
**Key Functions:**
- Initialize logger
- Create log directory
- Load configuration
- Launch Iced application

**Dependencies:**
- `anyhow` - Error handling
- `log` - Logging
- `std::fs` - File system operations

### src/app.rs
**Purpose:** Main application logic
**Key Components:**
- `ChatMessage` - Message data structure
- `Message` - Event enum
- `ChatApp` - Main application struct
- GUI rendering and layout
- Backend communication
- History management

**Dependencies:**
- `iced` - GUI framework
- `reqwest` - HTTP client
- `serde` - Serialization
- `chrono` - Timestamps
- `arboard` - Clipboard

### src/config.rs
**Purpose:** Configuration management
**Key Structures:**
- `AppConfig` - Root configuration
- `AppSettings` - Window settings
- `BackendSettings` - Backend configuration
- `UISettings` - UI preferences

**Dependencies:**
- `config` - Config file loading
- `serde` - Deserialization

### config.toml
**Purpose:** User-editable settings
**Sections:**
- `[app]` - Window configuration
- `[backend]` - Backend API settings
- `[ui]` - UI preferences

### Cargo.toml
**Purpose:** Rust project manifest
**Contents:**
- Package metadata
- Dependencies list
- Build configuration

### Documentation Files

#### README.md
- Complete project documentation
- Installation instructions
- Usage guide
- Feature list
- Development guide

#### QUICK_REFERENCE.md
- Quick reference card
- Common commands
- Keyboard shortcuts
- Troubleshooting tips

#### FEATURES.md
- Detailed feature descriptions
- Technical implementation details
- Future enhancements

#### USAGE_GUIDE.md
- Step-by-step usage instructions
- Visual interface guide
- Tips and tricks

#### ARCHITECTURE.md
- System architecture
- Component breakdown
- Data flow diagrams
- Technical details

#### CHANGELOG.md
- Version history
- Feature additions
- Bug fixes
- Roadmap

#### SUMMARY.md
- Quick summary of latest changes
- What was added
- How to use new features

#### BUILD_STATUS.md
- Current build status
- Recent changes
- Warnings and fixes

## 🎯 File Purposes by Role

### For End Users
1. Start with: `README.md`
2. Quick help: `QUICK_REFERENCE.md`
3. Learn features: `FEATURES.md` and `USAGE_GUIDE.md`
4. Check updates: `CHANGELOG.md`

### For Developers
1. Start with: `README.md`
2. Architecture: `ARCHITECTURE.md`
3. Build info: `BUILD_STATUS.md`
4. Changes: `CHANGELOG.md` and `SUMMARY.md`
5. Code: `src/` directory

### For Contributors
1. Read: `README.md` and `ARCHITECTURE.md`
2. Check: `CHANGELOG.md` for roadmap
3. Review: Source code in `src/`
4. Update: Documentation when adding features

## 📦 Distribution Files

### Minimal Distribution
For end users, include:
- Compiled binary (from `target/release/`)
- `config.toml`
- `README.md`
- `QUICK_REFERENCE.md`

### Full Distribution
For developers, include:
- All source files
- All documentation
- `Cargo.toml` and `Cargo.lock`
- `.gitignore`

### Excluded from Distribution
- `target/` directory
- `logs/` directory
- `chat_history.json`
- `.git/` directory
- `.vscode/` directory

## 🔄 File Maintenance

### Regular Updates
- `CHANGELOG.md` - Update with each version
- `BUILD_STATUS.md` - Update after builds
- `README.md` - Update when features change

### As Needed
- `FEATURES.md` - When adding features
- `USAGE_GUIDE.md` - When UI changes
- `ARCHITECTURE.md` - When structure changes

### Generated Automatically
- `Cargo.lock` - Updated by Cargo
- `target/` - Built by Cargo
- `chat_history.json` - Created by app

## 📏 File Size Estimates

| File Type | Approximate Size |
|-----------|------------------|
| Source code | ~50 KB |
| Documentation | ~150 KB |
| Configuration | ~1 KB |
| Compiled binary | ~10-20 MB |
| Dependencies | ~500 MB (in target/) |

## 🔍 Finding Files

### By Purpose
- **Configuration:** `config.toml`, `Cargo.toml`
- **Source Code:** `src/*.rs`
- **Documentation:** `*.md` files
- **Logs:** `logs/error.log`
- **History:** `chat_history.json`

### By Audience
- **End Users:** `README.md`, `QUICK_REFERENCE.md`, `USAGE_GUIDE.md`
- **Developers:** `ARCHITECTURE.md`, `BUILD_STATUS.md`, `src/`
- **Contributors:** All files

---

**Total Project Files:** ~20 files (excluding generated)  
**Total Documentation:** 9 comprehensive guides  
**Total Source Files:** 3 Rust modules  
**Last Updated:** Current Session
