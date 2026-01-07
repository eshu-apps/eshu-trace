#!/bin/bash
# Build script for eshu-trace

set -e

echo "🔨 Building eshu-trace..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build release binary
echo "Building release binary..."
cargo build --release

# Get binary size
SIZE=$(du -h target/release/eshu-trace | cut -f1)
echo "✅ Built successfully!"
echo "   Binary: target/release/eshu-trace ($SIZE)"
echo ""
echo "Install with:"
echo "   sudo cp target/release/eshu-trace /usr/local/bin/"
