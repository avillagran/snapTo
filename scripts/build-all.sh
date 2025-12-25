#!/bin/bash
# Master build script for SnapTo
# Builds Rust binaries and Flutter app, then bundles everything

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
cd "$PROJECT_ROOT"

echo "=========================================="
echo "  SnapTo - Full Build Script"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Build Rust binaries
echo -e "${YELLOW}Step 1: Building Rust binaries...${NC}"
echo ""

cargo build --release --bin snapto --bin snapto-tui

if [ -f "target/release/snapto" ] && [ -f "target/release/snapto-tui" ]; then
    echo -e "${GREEN}✓ Rust binaries built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build Rust binaries${NC}"
    exit 1
fi
echo ""

# Step 2: Build Flutter app
echo -e "${YELLOW}Step 2: Building Flutter app...${NC}"
echo ""

cd snapto_app

# Get dependencies
flutter pub get

# Build for macOS
flutter build macos --release

if [ -d "build/macos/Build/Products/Release/snapto_app.app" ]; then
    echo -e "${GREEN}✓ Flutter app built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build Flutter app${NC}"
    exit 1
fi
echo ""

# Step 3: Bundle Rust binaries into Flutter app
echo -e "${YELLOW}Step 3: Bundling Rust binaries into app...${NC}"
echo ""

APP_BUNDLE="build/macos/Build/Products/Release/snapto_app.app"
RESOURCES_DIR="$APP_BUNDLE/Contents/Resources"

mkdir -p "$RESOURCES_DIR"

cp "../target/release/snapto" "$RESOURCES_DIR/"
cp "../target/release/snapto-tui" "$RESOURCES_DIR/"
chmod +x "$RESOURCES_DIR/snapto"
chmod +x "$RESOURCES_DIR/snapto-tui"

echo -e "${GREEN}✓ Binaries bundled successfully${NC}"
echo ""

# Step 4: Create distributable
echo -e "${YELLOW}Step 4: Creating distributable...${NC}"
echo ""

cd ..
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Copy app bundle to dist
cp -r "snapto_app/$APP_BUNDLE" "$DIST_DIR/SnapTo.app"

# Rename the app
mv "$DIST_DIR/SnapTo.app/Contents/MacOS/snapto_app" "$DIST_DIR/SnapTo.app/Contents/MacOS/SnapTo" 2>/dev/null || true

echo -e "${GREEN}✓ Distributable created at: $DIST_DIR/SnapTo.app${NC}"
echo ""

# Summary
echo "=========================================="
echo -e "${GREEN}  Build Complete!${NC}"
echo "=========================================="
echo ""
echo "Files created:"
echo "  - target/release/snapto       (CLI)"
echo "  - target/release/snapto-tui   (TUI)"
echo "  - dist/SnapTo.app             (macOS App)"
echo ""
echo "To run the app:"
echo "  open dist/SnapTo.app"
echo ""
echo "To run in development mode:"
echo "  cd snapto_app && flutter run -d macos"
echo ""
