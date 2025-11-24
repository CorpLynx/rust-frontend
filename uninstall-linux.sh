#!/bin/bash
#
# Prometheus CLI Uninstaller for Linux
# Removes prometheus-cli and optionally its data
#
# Usage: ./uninstall-linux.sh

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

# Remove binary
remove_binary() {
    if [ -f "$BIN_DIR/$BINARY_NAME" ]; then
        print_info "Removing binary from $BIN_DIR..."
        rm "$BIN_DIR/$BINARY_NAME"
        print_success "Binary removed"
    else
        print_info "Binary not found at $BIN_DIR/$BINARY_NAME"
    fi
}

# Remove installation directory
remove_install_dir() {
    if [ -d "$INSTALL_DIR" ]; then
        print_info "Removing installation directory..."
        rm -rf "$INSTALL_DIR"
        print_success "Installation directory removed"
    else
        print_info "Installation directory not found"
    fi
}

# Remove configuration
remove_config() {
    if [ -d "$CONFIG_DIR" ]; then
        print_warning "Configuration directory contains: $CONFIG_DIR"
        read -p "Remove configuration? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$CONFIG_DIR"
            print_success "Configuration removed"
        else
            print_info "Configuration preserved"
        fi
    else
        print_info "Configuration directory not found"
    fi
    
    # Remove symlink if exists
    if [ -L "$HOME/config.toml" ]; then
        rm "$HOME/config.toml"
        print_success "Removed config symlink"
    fi
}

# Remove conversations
remove_conversations() {
    if [ -d "$CONVERSATIONS_DIR" ]; then
        # Count conversations
        CONV_COUNT=$(find "$CONVERSATIONS_DIR" -name "*.json" 2>/dev/null | wc -l | tr -d ' ')
        
        if [ "$CONV_COUNT" -gt 0 ]; then
            print_warning "Found $CONV_COUNT conversation(s) in $CONVERSATIONS_DIR"
            read -p "Remove all conversations? This cannot be undone! (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                rm -rf "$CONVERSATIONS_DIR"
                print_success "Conversations removed"
            else
                print_info "Conversations preserved at $CONVERSATIONS_DIR"
            fi
        else
            rm -rf "$CONVERSATIONS_DIR"
            print_success "Empty conversations directory removed"
        fi
    else
        print_info "Conversations directory not found"
    fi
}

# Remove shell integration
remove_shell_integration() {
    print_info "Checking shell integration..."
    
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
            print_info "Unknown shell, skipping"
            return 0
            ;;
    esac
    
    if [ -f "$SHELL_RC" ] && grep -q "prometheus" "$SHELL_RC"; then
        print_info "Removing shell integration from $SHELL_RC..."
        
        # Create backup
        cp "$SHELL_RC" "${SHELL_RC}.backup"
        
        # Remove Prometheus CLI related lines
        sed -i.tmp '/# Prometheus CLI/d' "$SHELL_RC"
        sed -i.tmp '/prometheus-cli/d' "$SHELL_RC"
        sed -i.tmp '/alias prometheus=/d' "$SHELL_RC"
        sed -i.tmp "s|$BIN_DIR:\$PATH|\$PATH|g" "$SHELL_RC"
        sed -i.tmp "s|set -gx PATH $BIN_DIR||g" "$SHELL_RC"
        rm -f "${SHELL_RC}.tmp"
        
        print_success "Shell integration removed (backup at ${SHELL_RC}.backup)"
    else
        print_info "No shell integration found"
    fi
}

# Remove man page
remove_man_page() {
    # Check system-wide location
    if [ -f "/usr/local/share/man/man1/prometheus-cli.1" ]; then
        print_info "Removing system man page..."
        sudo rm "/usr/local/share/man/man1/prometheus-cli.1" 2>/dev/null || true
        sudo mandb > /dev/null 2>&1 || true
        print_success "System man page removed"
    fi
    
    # Check user location
    if [ -f "$HOME/.local/share/man/man1/prometheus-cli.1" ]; then
        print_info "Removing user man page..."
        rm "$HOME/.local/share/man/man1/prometheus-cli.1"
        mandb > /dev/null 2>&1 || true
        print_success "User man page removed"
    fi
}

# Remove Ollama service
remove_ollama_service() {
    if command -v systemctl >/dev/null 2>&1; then
        if systemctl --user list-units --type=service | grep -q ollama; then
            print_warning "Ollama systemd service is configured"
            read -p "Stop and disable Ollama service? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                systemctl --user stop ollama.service 2>/dev/null || true
                systemctl --user disable ollama.service 2>/dev/null || true
                rm -f "$HOME/.config/systemd/user/ollama.service"
                systemctl --user daemon-reload
                print_success "Ollama service removed"
            fi
        fi
    fi
}

# Ask about Ollama
ask_about_ollama() {
    if command -v ollama >/dev/null 2>&1; then
        print_warning "Ollama is still installed on your system"
        echo "Ollama can be used by other applications."
        read -p "Do you want to uninstall Ollama? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_info "To uninstall Ollama:"
            echo ""
            echo "  1. Stop the service:"
            echo "     sudo systemctl stop ollama"
            echo ""
            echo "  2. Remove the binary:"
            echo "     sudo rm /usr/local/bin/ollama"
            echo ""
            echo "  3. Remove models and data:"
            echo "     rm -rf ~/.ollama"
            echo ""
            echo "  Or follow instructions at: https://ollama.com"
        fi
    fi
}

# Main uninstall flow
main() {
    print_header "Prometheus CLI Uninstaller for Linux"
    
    echo "This script will remove:"
    echo "  • Prometheus CLI binary"
    echo "  • Installation directory"
    echo "  • Shell integration"
    echo "  • Man page"
    echo ""
    echo "You will be asked about:"
    echo "  • Configuration files"
    echo "  • Conversation history"
    echo "  • Ollama service"
    echo "  • Ollama (optional)"
    echo ""
    
    read -p "Continue with uninstallation? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled"
        exit 0
    fi
    
    print_header "Removing Prometheus CLI"
    
    remove_binary
    remove_install_dir
    remove_shell_integration
    remove_man_page
    
    print_header "Data Cleanup"
    
    remove_config
    remove_conversations
    
    print_header "Optional Cleanup"
    
    remove_ollama_service
    ask_about_ollama
    
    print_header "Uninstallation Complete"
    
    echo ""
    print_success "Prometheus CLI has been uninstalled"
    echo ""
    echo "If you preserved any data, it's located at:"
    [ -d "$CONFIG_DIR" ] && echo "  • Config: $CONFIG_DIR"
    [ -d "$CONVERSATIONS_DIR" ] && echo "  • Conversations: $CONVERSATIONS_DIR"
    echo ""
    print_info "To reinstall, run the install script again"
    print_warning "Please restart your terminal for changes to take effect"
    echo ""
}

# Run main uninstallation
main
