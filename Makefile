.PHONY: help install uninstall build clean test run dev update check-deps

# Default target
help:
	@echo "Prometheus CLI - Available Commands"
	@echo ""
	@echo "Installation:"
	@echo "  make install        - Install prometheus-cli and dependencies"
	@echo "  make uninstall      - Remove prometheus-cli"
	@echo "  make update         - Update to latest version"
	@echo ""
	@echo "Development:"
	@echo "  make build          - Build release binary"
	@echo "  make dev            - Build debug binary"
	@echo "  make run            - Run in development mode"
	@echo "  make test           - Run tests"
	@echo "  make check-deps     - Check if dependencies are installed"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make format         - Format code with rustfmt"
	@echo "  make lint           - Run clippy linter"
	@echo ""

# Installation
install:
	@echo "Running installer..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		chmod +x install.sh && ./install.sh; \
	elif [ "$$(uname)" = "Linux" ]; then \
		chmod +x install-linux.sh && ./install-linux.sh; \
	else \
		echo "Unsupported OS: $$(uname)"; \
		exit 1; \
	fi

uninstall:
	@echo "Running uninstaller..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		chmod +x uninstall.sh && ./uninstall.sh; \
	elif [ "$$(uname)" = "Linux" ]; then \
		chmod +x uninstall-linux.sh && ./uninstall-linux.sh; \
	else \
		echo "Unsupported OS: $$(uname)"; \
		exit 1; \
	fi

# Building
build:
	@echo "Building release binary..."
	cargo build --release -p prometheus-cli
	@echo "Binary available at: target/release/prometheus-cli"

dev:
	@echo "Building debug binary..."
	cargo build -p prometheus-cli
	@echo "Binary available at: target/debug/prometheus-cli"

# Running
run:
	@echo "Running prometheus-cli in development mode..."
	cargo run -p prometheus-cli

# Testing
test:
	@echo "Running tests..."
	cargo test -p prometheus-cli

# Maintenance
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

format:
	@echo "Formatting code..."
	cargo fmt --all

lint:
	@echo "Running clippy..."
	cargo clippy -p prometheus-cli -- -D warnings

# Update
update:
	@echo "Updating prometheus-cli..."
	git pull origin main
	cargo build --release -p prometheus-cli
	@if [ -w /usr/local/bin ]; then \
		cp target/release/prometheus-cli /usr/local/bin/; \
	else \
		sudo cp target/release/prometheus-cli /usr/local/bin/; \
	fi
	@echo "Update complete!"

# Check dependencies
check-deps:
	@echo "Checking dependencies..."
	@command -v rustc >/dev/null 2>&1 && echo "✓ Rust installed" || echo "✗ Rust not found"
	@command -v cargo >/dev/null 2>&1 && echo "✓ Cargo installed" || echo "✗ Cargo not found"
	@command -v ollama >/dev/null 2>&1 && echo "✓ Ollama installed" || echo "✗ Ollama not found"
	@command -v git >/dev/null 2>&1 && echo "✓ Git installed" || echo "✗ Git not found"

# Quick install for CI/CD
ci-install:
	cargo build --release -p prometheus-cli
	mkdir -p $(HOME)/.local/bin
	cp target/release/prometheus-cli $(HOME)/.local/bin/
	@echo "Installed to $(HOME)/.local/bin/prometheus-cli"
