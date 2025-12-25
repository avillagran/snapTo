#!/bin/bash

# SnapTo App Build Script
# Cross-platform script to build the SnapTo Flutter app

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
FLUTTER_APP="$PROJECT_ROOT/snapto_app"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin*)  echo "macos" ;;
        Linux*)   echo "linux" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)        echo "unknown" ;;
    esac
}

OS=$(detect_os)
SNAPTO_CLI="$PROJECT_ROOT/target/release/snapto"
if [ "$OS" = "windows" ]; then
    SNAPTO_CLI="$PROJECT_ROOT/target/release/snapto.exe"
fi

echo "============================================"
echo "  SnapTo Desktop App - Build"
echo "============================================"
echo ""

info "Detected OS: $OS"
echo ""

# Check if Flutter is installed
if ! command -v flutter &> /dev/null; then
    error "Flutter is not installed or not in PATH"
    echo ""
    echo "Install Flutter:"
    case "$OS" in
        macos)
            echo "  brew install --cask flutter"
            ;;
        linux)
            echo "  sudo snap install flutter --classic"
            ;;
        windows)
            echo "  choco install flutter"
            ;;
    esac
    exit 1
fi

echo "Flutter version:"
flutter --version | head -n 1
echo ""

# Navigate to Flutter app directory
cd "$FLUTTER_APP"

# Check if platform is configured
check_platform_configured() {
    case "$OS" in
        macos)
            [ -f "$FLUTTER_APP/macos/Runner.xcworkspace/contents.xcworkspacedata" ] || \
            [ -d "$FLUTTER_APP/macos/Runner.xcodeproj" ]
            ;;
        linux)
            [ -f "$FLUTTER_APP/linux/CMakeLists.txt" ]
            ;;
        windows)
            [ -f "$FLUTTER_APP/windows/CMakeLists.txt" ]
            ;;
        *)
            return 1
            ;;
    esac
}

if ! check_platform_configured; then
    warning "Platform '$OS' is not configured for this Flutter project"
    info "Configuring $OS platform automatically..."
    echo ""

    flutter create --platforms="$OS" .

    success "Platform '$OS' configured successfully!"
    echo ""
fi

# Install dependencies
info "Installing dependencies..."
flutter pub get
echo ""

# Build SnapTo CLI if not exists
if [ ! -f "$SNAPTO_CLI" ]; then
    info "Building SnapTo CLI..."
    cd "$PROJECT_ROOT"
    cargo build --release
    cd "$FLUTTER_APP"
    success "CLI build complete!"
    echo ""
fi

# Build the app
info "Building $OS app (Release)..."
case "$OS" in
    macos)
        flutter build macos --release
        BUILD_PATH="$FLUTTER_APP/build/macos/Build/Products/Release/snapto_app.app"
        ;;
    linux)
        flutter build linux --release
        BUILD_PATH="$FLUTTER_APP/build/linux/x64/release/bundle"
        ;;
    windows)
        flutter build windows --release
        BUILD_PATH="$FLUTTER_APP/build/windows/x64/runner/Release"
        ;;
esac

echo ""
echo "============================================"
success "Build Complete!"
echo "============================================"
echo ""
echo "App location:"
echo "  $BUILD_PATH"
echo ""

case "$OS" in
    macos)
        echo "To install to Applications:"
        echo "  cp -r \"$BUILD_PATH\" /Applications/SnapTo.app"
        echo ""
        echo "To run:"
        echo "  open \"$BUILD_PATH\""
        ;;
    linux)
        echo "To run:"
        echo "  $BUILD_PATH/snapto_app"
        ;;
    windows)
        echo "To run:"
        echo "  $BUILD_PATH/snapto_app.exe"
        ;;
esac
echo ""
