#!/bin/bash
#
# Create a release package for Prometheus CLI
# This creates a distributable .tar.gz with the binary and necessary files
#
# Usage: ./scripts/create-release.sh [version]

set -e

VERSION=${1:-$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "prometheus-cli") | .version')}
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
RELEASE_NAME="prometheus-cli-${VERSION}-${OS}-${ARCH}"
RELEASE_DIR="releases/${RELEASE_NAME}"

echo "Creating release package: ${RELEASE_NAME}"

# Create release directory
mkdir -p "${RELEASE_DIR}"

# Build release binary
echo "Building release binary..."
cargo build --release -p prometheus-cli

# Copy binary
echo "Copying binary..."
cp target/release/prometheus-cli "${RELEASE_DIR}/"

# Copy documentation
echo "Copying documentation..."
cp README.md "${RELEASE_DIR}/"
cp INSTALL.md "${RELEASE_DIR}/"
cp LICENSE "${RELEASE_DIR}/" 2>/dev/null || echo "No LICENSE file found"

# Copy man page if it exists
if [ -f "docs/prometheus-cli.1" ]; then
    mkdir -p "${RELEASE_DIR}/man"
    cp docs/prometheus-cli.1 "${RELEASE_DIR}/man/"
fi

# Create default config
echo "Creating default config..."
cat > "${RELEASE_DIR}/config.toml" << 'EOF'
[app]
window_title = "Prometheus v0.2.0"

[backend]
url = "http://localhost:11434"
ollama_url = "http://localhost:11434"
timeout_seconds = 30

[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
EOF

# Create install script for the package
cat > "${RELEASE_DIR}/install.sh" << 'EOF'
#!/bin/bash
set -e

BIN_DIR="${BIN_DIR:-/usr/local/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/prometheus}"

echo "Installing Prometheus CLI..."

# Install binary
if [ -w "$BIN_DIR" ]; then
    cp prometheus-cli "$BIN_DIR/"
else
    sudo cp prometheus-cli "$BIN_DIR/"
fi

chmod +x "$BIN_DIR/prometheus-cli"
echo "✓ Binary installed to $BIN_DIR/prometheus-cli"

# Install config
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cp config.toml "$CONFIG_DIR/"
    echo "✓ Config installed to $CONFIG_DIR/config.toml"
else
    echo "✓ Config already exists at $CONFIG_DIR/config.toml"
fi

# Install man page
if [ -d "man" ] && [ -f "man/prometheus-cli.1" ]; then
    MAN_DIR="/usr/local/share/man/man1"
    sudo mkdir -p "$MAN_DIR"
    sudo cp man/prometheus-cli.1 "$MAN_DIR/"
    echo "✓ Man page installed"
fi

echo ""
echo "Installation complete!"
echo "Run: prometheus-cli --help"
EOF

chmod +x "${RELEASE_DIR}/install.sh"

# Create README for the package
cat > "${RELEASE_DIR}/PACKAGE_README.md" << EOF
# Prometheus CLI ${VERSION}

Binary release for ${OS} (${ARCH})

## Quick Install

\`\`\`bash
./install.sh
\`\`\`

This will:
- Install the binary to /usr/local/bin
- Create default config at ~/.config/prometheus/config.toml
- Install man page (if available)

## Manual Install

\`\`\`bash
# Copy binary
sudo cp prometheus-cli /usr/local/bin/

# Create config directory
mkdir -p ~/.config/prometheus

# Copy config
cp config.toml ~/.config/prometheus/
\`\`\`

## Requirements

- Ollama (for AI model runtime)
  Install from: https://ollama.com

## Usage

\`\`\`bash
# Start Ollama
ollama serve

# Run Prometheus CLI
prometheus-cli
\`\`\`

## Documentation

See README.md and INSTALL.md for full documentation.

## Support

- GitHub: https://github.com/your-username/prometheus
- Issues: https://github.com/your-username/prometheus/issues
EOF

# Create tarball
echo "Creating tarball..."
cd releases
tar -czf "${RELEASE_NAME}.tar.gz" "${RELEASE_NAME}"
cd ..

# Create checksum
echo "Creating checksum..."
cd releases
shasum -a 256 "${RELEASE_NAME}.tar.gz" > "${RELEASE_NAME}.tar.gz.sha256"
cd ..

# Print summary
echo ""
echo "Release package created successfully!"
echo ""
echo "Package: releases/${RELEASE_NAME}.tar.gz"
echo "Checksum: releases/${RELEASE_NAME}.tar.gz.sha256"
echo ""
echo "Contents:"
ls -lh "releases/${RELEASE_NAME}/"
echo ""
echo "To test the package:"
echo "  cd releases/${RELEASE_NAME}"
echo "  ./install.sh"
echo ""
echo "To distribute:"
echo "  Upload releases/${RELEASE_NAME}.tar.gz"
echo "  Upload releases/${RELEASE_NAME}.tar.gz.sha256"
