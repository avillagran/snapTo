#!/bin/bash
# Script to copy Rust binaries into the Flutter app bundle
# This is called during the build process

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/../../../.."
RUST_TARGET="$PROJECT_ROOT/target/release"
RESOURCES_DIR="$BUILT_PRODUCTS_DIR/$CONTENTS_FOLDER_PATH/Resources"

echo "Copying SnapTo binaries to app bundle..."

# Create Resources directory if it doesn't exist
mkdir -p "$RESOURCES_DIR"

# Copy snapto CLI binary
if [ -f "$RUST_TARGET/snapto" ]; then
    cp "$RUST_TARGET/snapto" "$RESOURCES_DIR/"
    chmod +x "$RESOURCES_DIR/snapto"
    echo "✓ Copied snapto CLI"
else
    echo "⚠ snapto binary not found at $RUST_TARGET/snapto"
    echo "  Run 'cargo build --release' first"
fi

# Copy snapto-tui binary
if [ -f "$RUST_TARGET/snapto-tui" ]; then
    cp "$RUST_TARGET/snapto-tui" "$RESOURCES_DIR/"
    chmod +x "$RESOURCES_DIR/snapto-tui"
    echo "✓ Copied snapto-tui"
else
    echo "⚠ snapto-tui binary not found at $RUST_TARGET/snapto-tui"
fi

echo "Done copying binaries"
