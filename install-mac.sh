#!/bin/bash
#
# Prometheus CLI Installer
# Installs prometheus-cli and all dependencies on a fresh macOS device
#
# Usage: curl -sSL https://raw.githubusercontent.com/your-repo/prometheus/main/install.sh | bash
#        or: ./install.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${INSTALL_DIR:-$HOME/.prometheus}"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/prometheus}"
CONVERSATIONS_DIR="${CONVERSATIONS_DIR:-$HOME/.prometheus/conversations}"
REPO_URL="https://github.com/your-username/prometheus.git"
BINARY_NAME="prometheus-cli"

# Print functions
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Check if running on macOS
check_os() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        print_error "This installer is designed for macOS. Detected OS: $OSTYPE"
        print_info "For other operating systems, please build from source."
        exit 1
    fi
    print_success "Running on macOS"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Homebrew if not present
install_homebrew() {
    if command_exists brew; then
        print_success "Homebrew already installed"
        return 0
    fi
    
    print_info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add Homebrew to PATH for Apple Silicon Macs
    if [[ $(uname -m) == "arm64" ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
    
    print_success "Homebrew installed"
}

# Install Rust if not present
install_rust() {
    if command_exists rustc && command_exists cargo; then
        RUST_VERSION=$(rustc --version | awk '{print $2}')
        print_success "Rust already installed (version $RUST_VERSION)"
        return 0
    fi
    
    print_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source cargo env
    source "$HOME/.cargo/env"
    
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    print_success "Rust installed (version $RUST_VERSION)"
}

# Install Ollama if not present
install_ollama() {
    if command_exists ollama; then
        print_success "Ollama already installed"
        return 0
    fi
    
    print_info "Installing Ollama..."
    curl -fsSL https://ollama.com/install.sh | sh
    
    print_success "Ollama installed"
}

# Clone or update repository
setup_repository() {
    if [ -d "$INSTALL_DIR" ]; then
        print_info "Updating existing installation at $INSTALL_DIR..."
        cd "$INSTALL_DIR"
        git pull origin main
    else
        print_info "Cloning repository to $INSTALL_DIR..."
        git clone "$REPO_URL" "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi
    
    print_success "Repository ready"
}

# Build prometheus-cli
build_cli() {
    print_info "Building prometheus-cli (this may take a few minutes)..."
    cd "$INSTALL_DIR"
    
    # Source cargo env to ensure it's in PATH
    source "$HOME/.cargo/env"
    
    cargo build --release -p prometheus-cli
    
    print_success "Build completed"
}

# Install binary
install_binary() {
    print_info "Installing binary to $BIN_DIR..."
    
    # Check if we need sudo
    if [ -w "$BIN_DIR" ]; then
        cp "$INSTALL_DIR/target/release/$BINARY_NAME" "$BIN_DIR/"
    else
        print_warning "Need sudo access to install to $BIN_DIR"
        sudo cp "$INSTALL_DIR/target/release/$BINARY_NAME" "$BIN_DIR/"
    fi
    
    chmod +x "$BIN_DIR/$BINARY_NAME"
    
    print_success "Binary installed to $BIN_DIR/$BINARY_NAME"
}

# Setup configuration
setup_config() {
    print_info "Setting up configuration..."
    
    # Create config directory
    mkdir -p "$CONFIG_DIR"
    
    # Create default config if it doesn't exist
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        cat > "$CONFIG_DIR/config.toml" << 'EOF'
[app]
window_title = "Prometheus v0.2.0"
window_width = 900.0
window_height = 650.0

[backend]
url = "http://localhost:11434"
ollama_url = "http://localhost:11434"
timeout_seconds = 30
saved_urls = []

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
EOF
        print_success "Created default configuration at $CONFIG_DIR/config.toml"
    else
        print_success "Configuration already exists at $CONFIG_DIR/config.toml"
    fi
    
    # Create conversations directory
    mkdir -p "$CONVERSATIONS_DIR"
    print_success "Created conversations directory at $CONVERSATIONS_DIR"
    
    # Create symlink in home directory for easy access
    if [ ! -L "$HOME/config.toml" ]; then
        ln -s "$CONFIG_DIR/config.toml" "$HOME/config.toml" 2>/dev/null || true
    fi
}

# Setup shell integration
setup_shell() {
    print_info "Setting up shell integration..."
    
    # Detect shell
    SHELL_NAME=$(basename "$SHELL")
    
    case "$SHELL_NAME" in
        zsh)
            SHELL_RC="$HOME/.zshrc"
            ;;
        bash)
            SHELL_RC="$HOME/.bashrc"
            ;;
        *)
            print_warning "Unknown shell: $SHELL_NAME. Skipping shell integration."
            return 0
            ;;
    esac
    
    # Add alias if not already present
    if ! grep -q "alias prometheus=" "$SHELL_RC" 2>/dev/null; then
        echo "" >> "$SHELL_RC"
        echo "# Prometheus CLI" >> "$SHELL_RC"
        echo "alias prometheus='prometheus-cli'" >> "$SHELL_RC"
        print_success "Added 'prometheus' alias to $SHELL_RC"
    else
        print_success "Shell alias already configured"
    fi
}

# Pull default Ollama model
setup_ollama_model() {
    print_info "Checking Ollama setup..."
    
    # Start Ollama service if not running
    if ! pgrep -x "ollama" > /dev/null; then
        print_info "Starting Ollama service..."
        ollama serve > /dev/null 2>&1 &
        sleep 2
    fi
    
    # Check if llama2 model is available
    if ollama list | grep -q "llama2"; then
        print_success "Ollama model already available"
    else
        print_info "Pulling llama2 model (this may take several minutes)..."
        print_warning "This will download ~4GB of data"
        
        read -p "Do you want to pull the llama2 model now? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            ollama pull llama2
            print_success "Model downloaded"
        else
            print_warning "Skipped model download. Run 'ollama pull llama2' later."
        fi
    fi
}

# Create man page
install_man_page() {
    if [ -f "$INSTALL_DIR/docs/prometheus-cli.1" ]; then
        print_info "Installing man page..."
        
        MAN_DIR="/usr/local/share/man/man1"
        sudo mkdir -p "$MAN_DIR"
        sudo cp "$INSTALL_DIR/docs/prometheus-cli.1" "$MAN_DIR/"
        sudo mandb > /dev/null 2>&1 || true
        
        print_success "Man page installed (run 'man prometheus-cli')"
    fi
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    if command_exists prometheus-cli; then
        VERSION=$(prometheus-cli --version 2>&1 | head -n1 || echo "unknown")
        print_success "prometheus-cli is installed: $VERSION"
    else
        print_error "prometheus-cli not found in PATH"
        return 1
    fi
    
    if command_exists ollama; then
        print_success "Ollama is installed"
    else
        print_warning "Ollama not found"
    fi
    
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        print_success "Configuration file exists"
    else
        print_warning "Configuration file not found"
    fi
}

# Print post-install instructions
print_instructions() {
    print_header "Installation Complete!"
    
    echo "Prometheus CLI has been successfully installed!"
    echo ""
    echo "📍 Installation locations:"
    echo "   Binary:        $BIN_DIR/$BINARY_NAME"
    echo "   Config:        $CONFIG_DIR/config.toml"
    echo "   Conversations: $CONVERSATIONS_DIR"
    echo "   Source:        $INSTALL_DIR"
    echo ""
    echo "🚀 Quick start:"
    echo "   1. Start Ollama (if not running):"
    echo "      ${GREEN}ollama serve${NC}"
    echo ""
    echo "   2. Launch Prometheus CLI:"
    echo "      ${GREEN}prometheus-cli${NC}"
    echo "      or simply: ${GREEN}prometheus${NC}"
    echo ""
    echo "   3. Start chatting:"
    echo "      ${GREEN}> What is Rust?${NC}"
    echo ""
    echo "📚 Documentation:"
    echo "   • Help:        ${GREEN}prometheus-cli --help${NC}"
    echo "   • Man page:    ${GREEN}man prometheus-cli${NC}"
    echo "   • Commands:    Type ${GREEN}/help${NC} in the CLI"
    echo "   • README:      $INSTALL_DIR/README.md"
    echo ""
    echo "⚙️  Configuration:"
    echo "   Edit: ${GREEN}$CONFIG_DIR/config.toml${NC}"
    echo ""
    echo "🔄 To update later:"
    echo "   ${GREEN}cd $INSTALL_DIR && git pull && cargo build --release -p prometheus-cli${NC}"
    echo ""
    echo "❓ Need help?"
    echo "   • GitHub: $REPO_URL"
    echo "   • Docs:   $INSTALL_DIR/README.md"
    echo ""
    
    # Remind to reload shell
    print_warning "Please restart your terminal or run: ${GREEN}source ~/.${SHELL_NAME}rc${NC}"
}

# Cleanup on error
cleanup() {
    if [ $? -ne 0 ]; then
        print_error "Installation failed!"
        print_info "Check the error messages above for details."
        print_info "You can try running the installer again or install manually."
    fi
}

trap cleanup EXIT

# Main installation flow
main() {
    print_header "Prometheus CLI Installer"
    
    echo "This script will install:"
    echo "  • Homebrew (if needed)"
    echo "  • Rust toolchain (if needed)"
    echo "  • Ollama (if needed)"
    echo "  • Prometheus CLI"
    echo ""
    echo "Installation directory: $INSTALL_DIR"
    echo "Binary directory: $BIN_DIR"
    echo ""
    
    read -p "Continue with installation? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Installation cancelled"
        exit 0
    fi
    
    print_header "Step 1: System Check"
    check_os
    
    print_header "Step 2: Installing Dependencies"
    install_homebrew
    install_rust
    install_ollama
    
    print_header "Step 3: Setting Up Repository"
    setup_repository
    
    print_header "Step 4: Building Prometheus CLI"
    build_cli
    
    print_header "Step 5: Installing Binary"
    install_binary
    
    print_header "Step 6: Configuration"
    setup_config
    setup_shell
    
    print_header "Step 7: Ollama Setup"
    setup_ollama_model
    
    print_header "Step 8: Documentation"
    install_man_page
    
    print_header "Step 9: Verification"
    verify_installation
    
    print_instructions
}

# Run main installation
main
