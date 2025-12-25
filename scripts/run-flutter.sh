#!/bin/bash

# SnapTo App Runner Script
# Cross-platform script to run the SnapTo Flutter app

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
echo "  SnapTo Desktop App"
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
            echo "  Or: https://docs.flutter.dev/get-started/install/macos"
            ;;
        linux)
            echo "  sudo snap install flutter --classic"
            echo "  Or: https://docs.flutter.dev/get-started/install/linux"
            ;;
        windows)
            echo "  choco install flutter"
            echo "  Or: https://docs.flutter.dev/get-started/install/windows"
            ;;
    esac
    exit 1
fi

echo "Flutter version:"
flutter --version | head -n 1
echo ""

# Check if SnapTo CLI is built
if [ ! -f "$SNAPTO_CLI" ]; then
    warning "SnapTo CLI not found at: $SNAPTO_CLI"
    echo ""
    read -p "Build SnapTo CLI now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Building SnapTo CLI..."
        cd "$PROJECT_ROOT"
        cargo build --release
        success "Build complete!"
        echo ""
    else
        warning "Skipping CLI build. Upload functionality may not work."
        echo ""
    fi
else
    success "SnapTo CLI found: $SNAPTO_CLI"
    echo ""
fi

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

# Check if dependencies are installed
if [ ! -d "$FLUTTER_APP/.dart_tool" ]; then
    info "Installing Flutter dependencies..."
    flutter pub get
    echo ""
fi

# Run the app
info "Starting SnapTo app on $OS..."
echo "Press 'r' for hot reload, 'R' for hot restart, 'q' to quit"
echo ""
echo "============================================"
echo ""

case "$OS" in
    macos)
        flutter run -d macos "$@"
        ;;
    linux)
        flutter run -d linux "$@"
        ;;
    windows)
        flutter run -d windows "$@"
        ;;
    *)
        error "Unsupported OS: $OS"
        exit 1
        ;;
esac
