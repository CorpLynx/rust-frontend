#!/bin/bash

# Install prometheus-cli man page
# Run this script after updating the man page to reinstall it

echo "Installing prometheus-cli man page..."

# Check if man page exists
if [ ! -f "docs/prometheus-cli.1" ]; then
    echo "Error: Man page not found at docs/prometheus-cli.1"
    exit 1
fi

# Create man directory if it doesn't exist
sudo mkdir -p /opt/homebrew/share/man/man1

# Copy the man page
sudo cp docs/prometheus-cli.1 /opt/homebrew/share/man/man1/

# Set proper permissions
sudo chmod 644 /opt/homebrew/share/man/man1/prometheus-cli.1

echo "Man page installed successfully!"
echo "You can now use: man prometheus-cli"

# Test the installation
echo ""
echo "Testing installation..."
if man prometheus-cli >/dev/null 2>&1; then
    echo "✓ Man page is working correctly"
else
    echo "✗ Man page installation failed"
    exit 1
fi