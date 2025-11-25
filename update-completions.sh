#!/bin/bash

# Update shell completions and man page for prometheus-cli
# Run this script after updating the CLI to regenerate completions and reinstall man page

echo "Updating prometheus-cli shell completions and man page..."

# Create completions directory if it doesn't exist
mkdir -p ~/.zsh/completions

# Generate zsh completions
echo "Generating zsh completions..."
prometheus-cli --generate-completions zsh > ~/.zsh/completions/_prometheus-cli

# Generate bash completions (optional)
mkdir -p ~/.bash_completions
echo "Generating bash completions..."
prometheus-cli --generate-completions bash > ~/.bash_completions/prometheus-cli

echo "Completions updated!"

# Install man page
echo ""
echo "Installing man page..."
if [ -f "docs/prometheus-cli.1" ]; then
    sudo mkdir -p /opt/homebrew/share/man/man1
    sudo cp docs/prometheus-cli.1 /opt/homebrew/share/man/man1/
    sudo chmod 644 /opt/homebrew/share/man/man1/prometheus-cli.1
    echo "Man page installed successfully!"
else
    echo "Warning: Man page not found at docs/prometheus-cli.1"
fi

echo ""
echo "Setup complete! Available resources:"
echo "  - Zsh completions: ~/.zsh/completions/_prometheus-cli"
echo "  - Bash completions: ~/.bash_completions/prometheus-cli"
echo "  - Man page: man prometheus-cli"
echo ""
echo "To enable bash completions, add this to your ~/.bashrc:"
echo "  source ~/.bash_completions/prometheus-cli"
echo ""
echo "Restart your shell or run 'source ~/.zshrc' to reload completions."