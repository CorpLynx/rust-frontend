#!/bin/bash

echo "Testing prometheus-cli setup..."
echo ""

# Test 1: Check if binary is in PATH
echo "1. Testing binary availability..."
if command -v prometheus-cli >/dev/null 2>&1; then
    echo "   ✓ prometheus-cli is available in PATH"
    prometheus-cli --version
else
    echo "   ✗ prometheus-cli not found in PATH"
    exit 1
fi

echo ""

# Test 2: Check if man page is available
echo "2. Testing man page..."
if man prometheus-cli >/dev/null 2>&1; then
    echo "   ✓ Man page is available (man prometheus-cli)"
else
    echo "   ✗ Man page not found"
fi

echo ""

# Test 3: Check if completions are available
echo "3. Testing shell completions..."
if [ -f ~/.zsh/completions/_prometheus-cli ]; then
    echo "   ✓ Zsh completions installed"
else
    echo "   ✗ Zsh completions not found"
fi

if [ -f ~/.bash_completions/prometheus-cli ]; then
    echo "   ✓ Bash completions available"
else
    echo "   - Bash completions not installed (optional)"
fi

echo ""

# Test 4: Test completion generation
echo "4. Testing completion generation..."
if prometheus-cli --generate-completions zsh >/dev/null 2>&1; then
    echo "   ✓ Completion generation works"
else
    echo "   ✗ Completion generation failed"
fi

echo ""

# Test 5: Test HTTPS enforcement
echo "5. Testing HTTPS enforcement..."
if prometheus-cli --url http://example.com "test" 2>&1 | grep -q "HTTPS"; then
    echo "   ✓ HTTPS enforcement is working"
else
    echo "   ✗ HTTPS enforcement not working"
fi

echo ""
echo "Setup verification complete!"
echo ""
echo "Available commands:"
echo "  - prometheus-cli --help"
echo "  - man prometheus-cli"
echo "  - Tab completion (restart shell if needed)"