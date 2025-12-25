#!/bin/bash
# Development helper script for SnapTo CLI

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Navigate to project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
cd "$PROJECT_ROOT"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    error "Could not find project root with Cargo.toml"
    exit 1
fi

# Show help
show_help() {
    cat << EOF
SnapTo CLI Development Helper

Usage: $0 <command>

Commands:
  build         Build debug version
  release       Build release version
  run           Run with default command
  upload        Run upload command
  watch         Run watch command
  config        Run config show command
  history       Run history command
  test          Run tests
  check         Check code without building
  fmt           Format code
  clippy        Run clippy linter
  clean         Clean build artifacts
  install       Install to cargo bin
  size          Show binary size
  deps          Show dependency tree
  audit         Run security audit
  help          Show this help

Examples:
  $0 build              # Build debug version
  $0 run upload -v      # Run upload with verbose
  $0 release            # Build optimized release
  $0 clippy             # Run linter

EOF
}

# Commands
case "$1" in
    build)
        info "Building debug version..."
        cargo build -p snapto-cli
        success "Build complete: target/debug/snapto"
        ;;

    release)
        info "Building release version..."
        cargo build --release -p snapto-cli
        success "Build complete: target/release/snapto"

        if [ -f "target/release/snapto" ]; then
            SIZE=$(du -h target/release/snapto | cut -f1)
            info "Binary size: $SIZE"
        fi
        ;;

    run)
        shift
        info "Running snapto $*"
        cargo run -p snapto-cli -- "$@"
        ;;

    upload)
        shift
        info "Running upload command..."
        cargo run -p snapto-cli -- upload "$@"
        ;;

    watch)
        shift
        info "Running watch command..."
        cargo run -p snapto-cli -- watch "$@"
        ;;

    config)
        shift
        ACTION=${1:-show}
        info "Running config $ACTION..."
        cargo run -p snapto-cli -- config "$ACTION"
        ;;

    history)
        shift
        info "Running history command..."
        cargo run -p snapto-cli -- history "$@"
        ;;

    test)
        info "Running tests..."
        cargo test -p snapto-cli
        ;;

    check)
        info "Checking code..."
        cargo check -p snapto-cli
        success "Check complete"
        ;;

    fmt)
        info "Formatting code..."
        cargo fmt -p snapto-cli
        success "Format complete"
        ;;

    clippy)
        info "Running clippy..."
        cargo clippy -p snapto-cli -- -D warnings
        success "Clippy complete"
        ;;

    clean)
        info "Cleaning build artifacts..."
        cargo clean -p snapto-cli
        success "Clean complete"
        ;;

    install)
        info "Installing snapto..."
        cargo install --path .
        success "Installed to $(which snapto)"
        ;;

    size)
        if [ -f "target/release/snapto" ]; then
            SIZE=$(du -h target/release/snapto | cut -f1)
            info "Release binary size: $SIZE"

            SIZE_BYTES=$(stat -f%z target/release/snapto 2>/dev/null || stat -c%s target/release/snapto)
            info "Exact size: $SIZE_BYTES bytes"
        else
            warning "Release binary not found. Run: $0 release"
        fi

        if [ -f "target/debug/snapto" ]; then
            SIZE=$(du -h target/debug/snapto | cut -f1)
            info "Debug binary size: $SIZE"
        fi
        ;;

    deps)
        info "Dependency tree..."
        cargo tree -p snapto-cli
        ;;

    audit)
        info "Running security audit..."
        if command -v cargo-audit &> /dev/null; then
            cargo audit
            success "Audit complete"
        else
            warning "cargo-audit not installed"
            info "Install with: cargo install cargo-audit"
        fi
        ;;

    help|--help|-h)
        show_help
        ;;

    *)
        error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
