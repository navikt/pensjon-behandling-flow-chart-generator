#!/usr/bin/env bash

set -e

echo "🚀 Installing behandling-flow..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo not found!"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if Graphviz is installed
if ! command -v dot &> /dev/null; then
    echo "⚠️  Warning: Graphviz not found!"
    echo "The tool requires Graphviz to generate diagrams."
    echo ""
    echo "Install it with:"
    echo "  macOS:   brew install graphviz"
    echo "  Ubuntu:  sudo apt-get install graphviz"
    echo "  Windows: choco install graphviz"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build the binary
echo "🔨 Building release binary..."
cargo build --release

# Determine install location
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Create install directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo "📁 Creating directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

# Copy binary
echo "📦 Installing to $INSTALL_DIR/behandling-flow"
cp target/release/behandling-flow "$INSTALL_DIR/behandling-flow"
chmod +x "$INSTALL_DIR/behandling-flow"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  Note: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
    echo "Or run the tool with full path:"
    echo "  $INSTALL_DIR/behandling-flow"
else
    echo ""
    echo "✅ Installation complete!"
    echo ""
    echo "Run 'behandling-flow --help' to get started."
fi

echo ""
echo "📚 For more information, see README.md"
