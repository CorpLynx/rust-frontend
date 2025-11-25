# Prometheus CLI Documentation

Educational documentation to help you understand how the Prometheus CLI application works.

## Documentation Files

### 📚 [ARCHITECTURE.md](ARCHITECTURE.md)
**High-level system design and architecture**

Learn about:
- Overall system architecture
- Module organization
- Data flow patterns
- Key design patterns
- Security features
- Extension points

Start here if you want to understand the big picture.

---

### 📖 [MODULE_GUIDE.md](MODULE_GUIDE.md)
**Detailed guide to each source file**

Comprehensive documentation for every module:
- What each file does
- Key structures and functions
- When you'd modify it
- Module dependencies
- Recommended learning path

Start here if you want to understand specific modules.

---

### 🔧 [HOW_IT_WORKS.md](HOW_IT_WORKS.md)
**Plain-language explanation of how everything works**

Step-by-step walkthroughs:
- What happens when you start the app
- How interactive mode works
- How non-interactive mode works
- Backend communication
- Configuration system
- Conversation storage
- Terminal rendering
- Common workflows

Start here if you want to understand the runtime behavior.

---

### ⚡ [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
**Fast lookup for common tasks**

Quick reference for:
- Module responsibilities (one-liners)
- Common tasks (how to add features)
- File locations
- Key data structures
- Exit codes
- CLI flags
- Configuration precedence
- Testing and build commands

Start here when you need to find something quickly.

---

### 📊 [FLOW_DIAGRAMS.md](FLOW_DIAGRAMS.md)
**Visual flow diagrams and architecture**

Visual representations of:
- Application startup flow
- Interactive mode flow
- Non-interactive mode flow
- Backend communication
- Configuration loading
- Conversation persistence
- Error handling
- URL validation
- Module dependencies

Start here if you're a visual learner.

---

### 📄 [prometheus-cli.1](prometheus-cli.1)
**Man page for the CLI application**

Official command-line reference:
- Complete flag documentation
- Usage examples
- Exit codes
- Configuration
- Troubleshooting

View with: `man ./docs/prometheus-cli.1`

---

## Learning Paths

### 🎯 I want to understand the codebase

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) - Get the big picture
2. Read [HOW_IT_WORKS.md](HOW_IT_WORKS.md) - Understand runtime behavior
3. Read [MODULE_GUIDE.md](MODULE_GUIDE.md) - Learn each module
4. Read the source code - Follow the learning path in MODULE_GUIDE.md

### 🔨 I want to add a feature

1. Check [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Find the relevant task
2. Check [MODULE_GUIDE.md](MODULE_GUIDE.md) - Find the relevant module
3. Read that module's documentation
4. Look at existing code for similar features
5. Implement your feature following the same patterns

### 🐛 I want to fix a bug

1. Identify which module has the bug
2. Read that module's section in [MODULE_GUIDE.md](MODULE_GUIDE.md)
3. Understand the data flow in [HOW_IT_WORKS.md](HOW_IT_WORKS.md)
4. Add a test that reproduces the bug
5. Fix the bug
6. Verify the test passes

### 📚 I want to learn Rust patterns

This codebase demonstrates:
- **Async/await** - See `backend.rs`, `streaming.rs`
- **Error handling** - See `error.rs`, `exit_codes.rs`
- **CLI parsing** - See `main.rs` with clap
- **Configuration** - See `config.rs` with serde
- **Testing** - See test modules and `tests/` directory
- **Module organization** - See overall structure

### 🚀 I want to deploy this

1. Read the main [README.md](../README.md) for installation
2. Read [HOW_IT_WORKS.md](HOW_IT_WORKS.md) for configuration
3. Check [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for troubleshooting
4. Review security features in [ARCHITECTURE.md](ARCHITECTURE.md)

## Documentation Maintenance

When modifying the codebase:

1. **Adding a module** - Update MODULE_GUIDE.md with new module documentation
2. **Changing architecture** - Update ARCHITECTURE.md with new patterns
3. **Adding features** - Update HOW_IT_WORKS.md with new workflows
4. **Adding flags** - Update QUICK_REFERENCE.md and prometheus-cli.1
5. **Changing behavior** - Update relevant documentation

## Additional Resources

### In the Repository

- **[../README.md](../README.md)** - Main user documentation
- **[../Cargo.toml](../Cargo.toml)** - Project dependencies
- **[../config.toml](../config.toml)** - Example configuration
- **[prometheus-cli/tests/](../prometheus-cli/tests/)** - Integration tests

### External Resources

- **Ollama API:** https://github.com/ollama/ollama/blob/main/docs/api.md
- **Rust Book:** https://doc.rust-lang.org/book/
- **Tokio Tutorial:** https://tokio.rs/tokio/tutorial
- **Clap Documentation:** https://docs.rs/clap/

## Getting Help

1. **Check the docs** - Start with this README
2. **Read the code** - It's well-commented
3. **Run the tests** - See how features are tested
4. **Experiment** - Make small changes and see what happens

## Contributing Documentation

Good documentation:
- Uses clear, simple language
- Includes examples
- Explains "why" not just "what"
- Is kept up-to-date with code changes
- Helps both beginners and experts

When writing documentation:
- Use markdown formatting
- Include code examples
- Add diagrams where helpful
- Link between related documents
- Keep it concise but complete

---

**Last Updated:** 2024
**Version:** 0.2.0
