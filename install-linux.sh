#!/bin/bash
#
# Prometheus CLI Installer for Linux
# Supports: Ubuntu/Debian, Fedora/RHEL/CentOS, Arch Linux
#
# Usage: curl -sSL https://raw.githubusercontent.com/your-repo/prometheus/main/install-linux.sh | bash
#        or: ./install-linux.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${INSTALL_DIR:-$HOME/.prometheus}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/prometheus}"
CONVERSATIONS_DIR="${CONVERSATIONS_DIR:-$HOME/.prometheus/conversations}"
REPO_URL="https://github.com/your-username/prometheus.git"
BINARY_NAME="prometheus-cli"

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        DISTRO_VERSION=$VERSION_ID
    elif [ -f /etc/lsb-release ]; then
        . /etc/lsb-release
        DISTRO=$DISTRIB_ID
        DISTRO_VERSION=$DISTRIB_RELEASE
    else
        DISTRO="unknown"
    fi
    
    DISTRO=$(echo "$DISTRO" | tr '[:upper:]' '[:lower:]')
}

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

# Check if running on Linux
check_os() {
    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
        print_error "This installer is designed for Linux. Detected OS: $OSTYPE"
        print_info "For macOS, use: curl -sSL <url>/install.sh | bash"
        exit 1
    fi
    
    detect_distro
    print_success "Running on Linux ($DISTRO)"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if user has sudo access
check_sudo() {
    if sudo -n true 2>/dev/null; then
        HAS_SUDO=true
    else
        print_warning "Some operations may require sudo access"
        HAS_SUDO=false
    fi
}

# Install system dependencies based on distro
install_system_deps() {
    print_info "Installing system dependencies for $DISTRO..."
    
    case "$DISTRO" in
        ubuntu|debian|pop|linuxmint)
            print_info "Using apt package manager..."
            sudo apt-get update
            sudo apt-get install -y \
                curl \
                git \
                build-essential \
                pkg-config \
                libssl-dev \
                ca-certificates
            ;;
            
        fedora|rhel|centos|rocky|almalinux)
            print_info "Using dnf/yum package manager..."
            if command_exists dnf; then
                sudo dnf install -y \
                    curl \
                    git \
                    gcc \
                    gcc-c++ \
                    make \
                    openssl-devel \
                    pkg-config \
                    ca-certificates
            else
                sudo yum install -y \
                    curl \
                    git \
                    gcc \
                    gcc-c++ \
                    make \
                    openssl-devel \
                    pkg-config \
                    ca-certificates
            fi
            ;;
            
        arch|manjaro|endeavouros)
            print_info "Using pacman package manager..."
            sudo pacman -Sy --noconfirm \
                curl \
                git \
                base-devel \
                openssl \
                pkg-config \
                ca-certificates
            ;;
            
        opensuse*|sles)
            print_info "Using zypper package manager..."
            sudo zypper install -y \
                curl \
                git \
                gcc \
                gcc-c++ \
                make \
                libopenssl-devel \
                pkg-config \
                ca-certificates
            ;;
            
        *)
            print_warning "Unknown distribution: $DISTRO"
            print_info "Please install manually: curl, git, gcc, make, openssl-dev, pkg-config"
            read -p "Continue anyway? (y/n) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
            ;;
    esac
    
    print_success "System dependencies installed"
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

# Setup Ollama as systemd service (if systemd is available)
setup_ollama_service() {
    if ! command_exists systemctl; then
        print_info "systemd not available, skipping service setup"
        return 0
    fi
    
    if systemctl --user list-units --type=service | grep -q ollama; then
        print_success "Ollama service already configured"
        return 0
    fi
    
    print_info "Setting up Ollama as a user service..."
    
    # Create user systemd directory
    mkdir -p "$HOME/.config/systemd/user"
    
    # Create service file
    cat > "$HOME/.config/systemd/user/ollama.service" << 'EOF'
[Unit]
Description=Ollama Service
After=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/ollama serve
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
    
    # Reload systemd and enable service
    systemctl --user daemon-reload
    systemctl --user enable ollama.service
    systemctl --user start ollama.service
    
    print_success "Ollama service configured and started"
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
    
    # Create bin directory if it doesn't exist
    mkdir -p "$BIN_DIR"
    
    # Copy binary
    cp "$INSTALL_DIR/target/release/$BINARY_NAME" "$BIN_DIR/"
    chmod +x "$BIN_DIR/$BINARY_NAME"
    
    print_success "Binary installed to $BIN_DIR/$BINARY_NAME"
    
    # Check if BIN_DIR is in PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        print_warning "$BIN_DIR is not in your PATH"
        print_info "Add it by running: export PATH=\"$BIN_DIR:\$PATH\""
    fi
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
        bash)
            SHELL_RC="$HOME/.bashrc"
            ;;
        zsh)
            SHELL_RC="$HOME/.zshrc"
            ;;
        fish)
            SHELL_RC="$HOME/.config/fish/config.fish"
            ;;
        *)
            print_warning "Unknown shell: $SHELL_NAME. Skipping shell integration."
            return 0
            ;;
    esac
    
    # Add PATH if needed
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        if [ "$SHELL_NAME" = "fish" ]; then
            echo "" >> "$SHELL_RC"
            echo "# Prometheus CLI" >> "$SHELL_RC"
            echo "set -gx PATH $BIN_DIR \$PATH" >> "$SHELL_RC"
        else
            echo "" >> "$SHELL_RC"
            echo "# Prometheus CLI" >> "$SHELL_RC"
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_RC"
        fi
        print_success "Added $BIN_DIR to PATH in $SHELL_RC"
    fi
    
    # Add alias if not already present
    if ! grep -q "alias prometheus=" "$SHELL_RC" 2>/dev/null; then
        if [ "$SHELL_NAME" = "fish" ]; then
            echo "alias prometheus='prometheus-cli'" >> "$SHELL_RC"
        else
            echo "alias prometheus='prometheus-cli'" >> "$SHELL_RC"
        fi
        print_success "Added 'prometheus' alias to $SHELL_RC"
    else
        print_success "Shell alias already configured"
    fi
}

# Pull default Ollama model
setup_ollama_model() {
    print_info "Checking Ollama setup..."
    
    # Check if Ollama is running
    if ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
        print_info "Starting Ollama..."
        if command_exists systemctl; then
            systemctl --user start ollama.service 2>/dev/null || ollama serve > /dev/null 2>&1 &
        else
            ollama serve > /dev/null 2>&1 &
        fi
        sleep 3
    fi
    
    # Check if llama2 model is available
    if ollama list 2>/dev/null | grep -q "llama2"; then
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
        
        # Try system-wide installation first
        if [ -w "/usr/local/share/man/man1" ] || sudo -n true 2>/dev/null; then
            MAN_DIR="/usr/local/share/man/man1"
            sudo mkdir -p "$MAN_DIR"
            sudo cp "$INSTALL_DIR/docs/prometheus-cli.1" "$MAN_DIR/"
            sudo mandb > /dev/null 2>&1 || true
            print_success "Man page installed (run 'man prometheus-cli')"
        else
            # Fall back to user installation
            MAN_DIR="$HOME/.local/share/man/man1"
            mkdir -p "$MAN_DIR"
            cp "$INSTALL_DIR/docs/prometheus-cli.1" "$MAN_DIR/"
            mandb > /dev/null 2>&1 || true
            print_success "Man page installed to user directory"
            
            # Add to MANPATH if needed
            if [[ ":$MANPATH:" != *":$HOME/.local/share/man:"* ]]; then
                SHELL_NAME=$(basename "$SHELL")
                case "$SHELL_NAME" in
                    bash) SHELL_RC="$HOME/.bashrc" ;;
                    zsh) SHELL_RC="$HOME/.zshrc" ;;
                    *) SHELL_RC="" ;;
                esac
                
                if [ -n "$SHELL_RC" ]; then
                    echo "export MANPATH=\"$HOME/.local/share/man:\$MANPATH\"" >> "$SHELL_RC"
                fi
            fi
        fi
    fi
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    # Source cargo env for verification
    source "$HOME/.cargo/env" 2>/dev/null || true
    
    if [ -f "$BIN_DIR/$BINARY_NAME" ]; then
        VERSION=$("$BIN_DIR/$BINARY_NAME" --version 2>&1 | head -n1 || echo "unknown")
        print_success "prometheus-cli is installed: $VERSION"
    else
        print_error "prometheus-cli not found at $BIN_DIR/$BINARY_NAME"
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
    echo "   1. Restart your terminal or run:"
    echo "      ${GREEN}source ~/.$(basename $SHELL)rc${NC}"
    echo ""
    echo "   2. Start Ollama (if not running):"
    if command_exists systemctl; then
        echo "      ${GREEN}systemctl --user start ollama${NC}"
    else
        echo "      ${GREEN}ollama serve${NC}"
    fi
    echo ""
    echo "   3. Launch Prometheus CLI:"
    echo "      ${GREEN}prometheus-cli${NC}"
    echo "      or simply: ${GREEN}prometheus${NC}"
    echo ""
    echo "   4. Start chatting:"
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
    echo "   ${GREEN}cp target/release/prometheus-cli $BIN_DIR/${NC}"
    echo ""
    echo "❓ Need help?"
    echo "   • GitHub: $REPO_URL"
    echo "   • Docs:   $INSTALL_DIR/README.md"
    echo ""
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
    print_header "Prometheus CLI Installer for Linux"
    
    echo "This script will install:"
    echo "  • System dependencies (gcc, make, openssl, etc.)"
    echo "  • Rust toolchain (if needed)"
    echo "  • Ollama (if needed)"
    echo "  • Prometheus CLI"
    echo ""
    echo "Installation directory: $INSTALL_DIR"
    echo "Binary directory: $BIN_DIR"
    echo "Distribution: $DISTRO"
    echo ""
    
    read -p "Continue with installation? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Installation cancelled"
        exit 0
    fi
    
    print_header "Step 1: System Check"
    check_os
    check_sudo
    
    print_header "Step 2: Installing System Dependencies"
    install_system_deps
    
    print_header "Step 3: Installing Rust"
    install_rust
    
    print_header "Step 4: Installing Ollama"
    install_ollama
    setup_ollama_service
    
    print_header "Step 5: Setting Up Repository"
    setup_repository
    
    print_header "Step 6: Building Prometheus CLI"
    build_cli
    
    print_header "Step 7: Installing Binary"
    install_binary
    
    print_header "Step 8: Configuration"
    setup_config
    setup_shell
    
    print_header "Step 9: Ollama Setup"
    setup_ollama_model
    
    print_header "Step 10: Documentation"
    install_man_page
    
    print_header "Step 11: Verification"
    verify_installation
    
    print_instructions
}

# Run main installation
main
