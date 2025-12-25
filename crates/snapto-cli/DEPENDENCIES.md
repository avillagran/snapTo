# SnapTo CLI Dependencies

## Overview

This document explains the dependencies used in the SnapTo CLI and their purposes.

## Core Dependencies

### `snapto-core`
- **Path**: `../snapto-core`
- **Purpose**: Core functionality library containing:
  - Clipboard management
  - Configuration handling
  - SSH/SFTP upload logic
  - History storage
  - Image processing
- **Note**: This is the main dependency that provides all business logic

## Runtime Dependencies

### `tokio` (workspace)
- **Purpose**: Async runtime for handling asynchronous operations
- **Features**: `full`
- **Usage**:
  - Async command execution
  - SSH connection handling
  - Concurrent clipboard monitoring

### `clap` (workspace)
- **Purpose**: Command-line argument parsing
- **Features**: `derive`
- **Usage**:
  - CLI structure definition
  - Subcommand handling
  - Argument validation
  - Help text generation

### `anyhow` (workspace)
- **Purpose**: Flexible error handling
- **Usage**:
  - Context-aware error messages
  - Error propagation with `.context()`
  - User-friendly error reporting

### `tracing` (workspace)
- **Purpose**: Structured logging
- **Usage**:
  - Debug logging
  - Performance monitoring
  - Error tracking

### `tracing-subscriber` (workspace)
- **Purpose**: Tracing implementation
- **Usage**:
  - Log formatting
  - Environment-based log filtering
  - Console output

### `serde` (workspace)
- **Purpose**: Serialization/deserialization
- **Features**: `derive`
- **Usage**:
  - Configuration parsing
  - Data structure serialization

### `serde_json` (workspace)
- **Purpose**: JSON support for serde
- **Usage**:
  - JSON configuration support
  - API response parsing

### `chrono` (workspace)
- **Purpose**: Date and time handling
- **Usage**:
  - Timestamp formatting in history
  - Upload time tracking

### `directories` (workspace)
- **Purpose**: Platform-specific directory paths
- **Usage**:
  - Config file location
  - History database location
  - Cross-platform path handling

## CLI-Specific Dependencies

### `colored`
- **Version**: `2.1`
- **Purpose**: Terminal color and styling
- **Usage**:
  - Success messages (green)
  - Error messages (red)
  - Warning messages (yellow)
  - Info messages (blue)
  - Formatted output

**Features used**:
```rust
use colored::*;

println!("{}", "Success!".green().bold());
println!("{}", "Error!".red());
println!("{}", "Warning".yellow());
```

### `indicatif`
- **Version**: `0.17`
- **Purpose**: Progress bars and spinners
- **Usage**:
  - Upload progress visualization
  - Spinner for long operations
  - Transfer speed display

**Features used**:
```rust
// Progress bar for uploads
let pb = ProgressBar::new(total_bytes);
pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
    .progress_chars("#>-"));

// Spinner for operations
let spinner = ProgressBar::new_spinner();
spinner.set_message("Processing...");
```

## Dependency Size Comparison

Approximate sizes (optimized build):

- `snapto-core`: ~500 KB (includes SSH, image processing)
- `tokio`: ~300 KB
- `clap`: ~200 KB
- `colored`: ~20 KB
- `indicatif`: ~50 KB
- Other dependencies: ~100 KB

**Total CLI binary size**: ~2-3 MB (release build)

## Platform-Specific Notes

### macOS
- All dependencies work out of the box
- No additional system libraries needed

### Linux
- May need `libssl-dev` for SSH support
- May need `libx11-dev` for clipboard on X11
- May need `libwayland-dev` for clipboard on Wayland

### Windows
- Visual C++ redistributables may be needed
- Clipboard works via Windows API
- SSH support via libssh2

## Development Dependencies

While not explicitly listed, the following are inherited from workspace:

- `thiserror`: Error type derivation (used by snapto-core)
- `toml`: TOML parsing for config
- `arboard`: Clipboard access
- `ssh2`: SSH/SFTP operations
- `image`: Image format handling
- `rusqlite`: History database

## Optional Features

Currently, all features are enabled. Future versions may support:

- `minimal`: Exclude history database
- `no-progress`: Exclude progress bars
- `no-color`: Exclude colored output
- `clipboard-only`: Only clipboard operations

## Updating Dependencies

To update dependencies:

```bash
# Update all workspace dependencies
cargo update

# Update specific dependency
cargo update colored

# Check for outdated dependencies
cargo outdated
```

## Security Considerations

### Dependency Audit

Run security audits regularly:

```bash
cargo audit
```

### Key Dependencies Security

- `ssh2`: Maintained, regularly updated for security
- `tokio`: Large community, well-audited
- `clap`: Stable, widely used
- `rusqlite`: Bundled SQLite, regularly updated

### Supply Chain

All dependencies are from crates.io and can be verified:

```bash
cargo tree
cargo verify-project
```

## Build Optimization

### Release Build

```bash
# Optimized release build
cargo build --release -p snapto-cli

# Even smaller binary
cargo build --release -p snapto-cli --features minimal
strip target/release/snapto  # Remove debug symbols
```

### Compile Time

Approximate compile times:

- Clean build: ~2-3 minutes
- Incremental build: ~10-30 seconds
- Release build: ~3-5 minutes

### Binary Size Optimization

Add to workspace `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

This can reduce binary size by 30-40%.

## License Compatibility

All dependencies are MIT or Apache-2.0 licensed, compatible with SnapTo's MIT license.

## Dependency Graph

```
snapto-cli
├── snapto-core
│   ├── tokio (async runtime)
│   ├── arboard (clipboard)
│   ├── ssh2 (SFTP)
│   ├── image (image processing)
│   ├── rusqlite (history)
│   └── serde + toml (config)
├── clap (CLI framework)
├── colored (terminal colors)
├── indicatif (progress bars)
├── tracing (logging)
└── anyhow (errors)
```

## Future Dependencies

Potential additions:

- `notify`: File system watching
- `crossterm`: Advanced terminal features
- `dialoguer`: Interactive prompts
- `reqwest`: HTTP uploads
- `aws-sdk-s3`: S3 support
