#!/bin/bash
set -e


# Chiral Package Manager — Installer
# Usage: curl -sSL https://raw.githubusercontent.com/Amaterus1125/chpm/main/install.sh | bash


REPO="https://github.com/Amaterus1125/Chiral-CrossDistro-Package-Manager"
BINARY_URL="https://github.com/Amaterus1125/chpm/releases/latest/download/chiral"
INSTALL_DIR="$HOME/.local/bin"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Chiral Package Manager — Installer"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create ~/.local/bin if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Check if running as root — install system-wide instead
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
    echo "Running as root — installing system-wide to $INSTALL_DIR"
else
    echo "Installing to $INSTALL_DIR"
fi

# Download the pre-built binary from GitHub Releases
echo "Downloading chiral..."
if command -v curl &>/dev/null; then
    curl -sSL "$BINARY_URL" -o "$INSTALL_DIR/chiral"
elif command -v wget &>/dev/null; then
    wget -q "$BINARY_URL" -O "$INSTALL_DIR/chiral"
else
    echo "Error: curl or wget is required."
    exit 1
fi

# Make it executable
chmod +x "$INSTALL_DIR/chiral"

echo "Chiral installed to $INSTALL_DIR/chiral"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo " Add chiral to your PATH by running:"
    echo ""
    echo "   echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
    echo "   source ~/.bashrc"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
else
    echo ""
    echo "Run 'chiral install <package>' to get started!"
fi
