#!/bin/bash
# Prometheus CLI - macOS Full Installer
# This script installs Prometheus CLI and all dependencies

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

BIN_DIR="${BIN_DIR:-/usr/local/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/prometheus}"
INSTALL_DIR="$HOME/.prometheus"

print_header "Prometheus CLI Installer for macOS"
print_info "Version: __VERSION__"

# Check macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This installer is for macOS only"
    exit 1
fi

# Install Homebrew
print_header "Installing Homebrew"
if ! command -v brew >/dev/null 2>&1; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    if [[ $(uname -m) == "arm64" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
    print_success "Homebrew installed"
else
    print_success "Homebrew already installed"
fi

# Install Rust
print_header "Installing Rust"
if ! command -v rustc >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    print_success "Rust installed"
else
    print_success "Rust already installed"
fi

# Install Ollama
print_header "Installing Ollama"
if ! command -v ollama >/dev/null 2>&1; then
    curl -fsSL https://ollama.com/install.sh | sh
    print_success "Ollama installed"
else
    print_success "Ollama already installed"
fi

# Extract and install binary
print_header "Installing Prometheus CLI"
ARCHIVE_LINE=$(awk '/^__ARCHIVE_BELOW__/ {print NR + 1; exit 0; }' "$0")
mkdir -p "$INSTALL_DIR"
tail -n+$ARCHIVE_LINE "$0" | tar xzf - -C "$INSTALL_DIR"

if [ -w "$BIN_DIR" ]; then
    cp "$INSTALL_DIR/prometheus-cli" "$BIN_DIR/"
else
    sudo cp "$INSTALL_DIR/prometheus-cli" "$BIN_DIR/"
fi
chmod +x "$BIN_DIR/prometheus-cli"
print_success "Binary installed to $BIN_DIR/prometheus-cli"

# Setup configuration
print_header "Setting Up Configuration"
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cp "$INSTALL_DIR/config.toml" "$CONFIG_DIR/"
    print_success "Configuration created"
else
    print_success "Configuration already exists"
fi

# Setup shell integration
SHELL_NAME=$(basename "$SHELL")
case "$SHELL_NAME" in
    zsh) SHELL_RC="$HOME/.zshrc" ;;
    bash) SHELL_RC="$HOME/.bashrc" ;;
    *) SHELL_RC="" ;;
esac

if [ -n "$SHELL_RC" ]; then
    if ! grep -q "alias prometheus=" "$SHELL_RC" 2>/dev/null; then
        echo "" >> "$SHELL_RC"
        echo "# Prometheus CLI" >> "$SHELL_RC"
        echo "alias prometheus='prometheus-cli'" >> "$SHELL_RC"
    fi
    print_success "Shell integration configured"
fi

# Install man page
if [ -f "$INSTALL_DIR/docs/prometheus-cli.1" ]; then
    MAN_DIR="/usr/local/share/man/man1"
    sudo mkdir -p "$MAN_DIR"
    sudo cp "$INSTALL_DIR/docs/prometheus-cli.1" "$MAN_DIR/"
    print_success "Man page installed"
fi

print_header "Installation Complete!"
echo "🚀 Quick start:"
echo "   1. Restart your terminal or run: source ~/.${SHELL_NAME}rc"
echo "   2. Start Ollama: ollama serve"
echo "   3. Run: prometheus-cli"
echo ""
echo "📚 Documentation: man prometheus-cli"

exit 0

__ARCHIVE_BELOW__
