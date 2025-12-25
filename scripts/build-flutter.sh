#!/bin/bash

# SnapTo App Build Script
# Builds the macOS app for release

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
FLUTTER_APP="$PROJECT_ROOT/snapto_app"
SNAPTO_CLI="$PROJECT_ROOT/target/release/snapto"

echo "============================================"
echo "  SnapTo macOS App - Build"
echo "============================================"
echo ""

# Check if Flutter is installed
if ! command -v flutter &> /dev/null; then
    echo "ERROR: Flutter is not installed or not in PATH"
    exit 1
fi

# Navigate to Flutter app directory
cd "$FLUTTER_APP"

# Install dependencies
echo "Installing dependencies..."
flutter pub get
echo ""

# Build SnapTo CLI if not exists
if [ ! -f "$SNAPTO_CLI" ]; then
    echo "Building SnapTo CLI..."
    cd "$PROJECT_ROOT"
    cargo build --release
    cd "$FLUTTER_APP"
    echo ""
fi

# Build the macOS app
echo "Building macOS app (Release)..."
flutter build macos --release

echo ""
echo "============================================"
echo "  Build Complete!"
echo "============================================"
echo ""
echo "App location:"
echo "  $FLUTTER_APP/build/macos/Build/Products/Release/snapto_app.app"
echo ""
echo "To install to Applications:"
echo "  cp -r $FLUTTER_APP/build/macos/Build/Products/Release/snapto_app.app /Applications/SnapTo.app"
echo ""
echo "To run:"
echo "  open $FLUTTER_APP/build/macos/Build/Products/Release/snapto_app.app"
echo ""
