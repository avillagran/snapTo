#!/bin/bash

# SnapTo App Runner Script
# This script helps you quickly run the SnapTo macOS app

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
FLUTTER_APP="$PROJECT_ROOT/snapto_app"
SNAPTO_CLI="$PROJECT_ROOT/target/release/snapto"

echo "============================================"
echo "  SnapTo macOS App"
echo "============================================"
echo ""

# Check if Flutter is installed
if ! command -v flutter &> /dev/null; then
    echo "ERROR: Flutter is not installed or not in PATH"
    echo ""
    echo "Install Flutter:"
    echo "  brew install --cask flutter"
    echo ""
    echo "Or download from: https://docs.flutter.dev/get-started/install/macos"
    exit 1
fi

echo "Flutter version:"
flutter --version | head -n 1
echo ""

# Check if SnapTo CLI is built
if [ ! -f "$SNAPTO_CLI" ]; then
    echo "WARNING: SnapTo CLI not found at: $SNAPTO_CLI"
    echo ""
    read -p "Build SnapTo CLI now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Building SnapTo CLI..."
        cd "$PROJECT_ROOT"
        cargo build --release
        echo "Build complete!"
        echo ""
    else
        echo "Skipping CLI build. Upload functionality may not work."
        echo ""
    fi
else
    echo "SnapTo CLI found: $SNAPTO_CLI"
    echo ""
fi

# Navigate to Flutter app directory
cd "$FLUTTER_APP"

# Check if dependencies are installed
if [ ! -d "$FLUTTER_APP/.dart_tool" ]; then
    echo "Installing Flutter dependencies..."
    flutter pub get
    echo ""
fi

# Run the app
echo "Starting SnapTo app..."
echo "Press 'r' for hot reload, 'R' for hot restart, 'q' to quit"
echo ""
echo "============================================"
echo ""

flutter run -d macos "$@"
