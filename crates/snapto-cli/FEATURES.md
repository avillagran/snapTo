# SnapTo CLI - Feature Documentation

## Core Features

### 1. Upload Command

Upload images from clipboard to remote servers via SSH/SFTP.

**Key Features:**
- Reads image directly from system clipboard
- Detects image format automatically (PNG, JPG, WebP)
- Shows file size before upload
- Real-time progress bar during upload
- Calculates and displays upload speed
- Automatically copies resulting URL to clipboard
- Saves upload to history database

**Output Elements:**
```
→ Reading image from clipboard...           # Step indicator
✓ Found image in clipboard (342.5 KB)       # Success with size
→ Loading configuration...                  # Next step
ℹ Using destination: production             # Info message
[████████████████████] 100% 342.5 KB/s     # Progress bar
✓ Upload completed!                         # Success
────────────────────────────────────────    # Separator
  URL: https://...                          # Key-value output
  Size: 342.5 KB
  Duration: 1.234s
  Speed: 277.4 KB/s
────────────────────────────────────────
→ Copying URL to clipboard...
✓ URL copied to clipboard!
```

### 2. Watch Mode

Continuously monitor clipboard for new images and auto-upload.

**Key Features:**
- Configurable check interval (default 500ms)
- Change detection using image hash
- Only uploads when image changes
- Upload counter to track batch operations
- Non-blocking: continues on errors
- Ctrl+C to gracefully stop

**Intelligence:**
- Detects duplicate images to avoid re-uploading
- Shows upload number for tracking
- Copies each URL to clipboard automatically
- Maintains history for each upload

**Output Elements:**
```
SnapTo Watch Mode
ℹ Watching clipboard for images...
ℹ Check interval: 500ms
ℹ Press Ctrl+C to stop
────────────────────────────────────────
ℹ Using destination: production
────────────────────────────────────────
ℹ Waiting for next image...

→ New image detected (445.2 KB)
⠋ Uploading...                             # Spinner
✓ Upload #1 completed!
  URL: https://...
  Duration: 1.345s
ℹ URL copied to clipboard
────────────────────────────────────────
ℹ Waiting for next image...
```

### 3. Configuration Management

Manage configuration files with multiple commands.

#### 3.1 Config Show
Displays current configuration in formatted output.

**Shows:**
- Default destination
- Filename template
- Default format
- All configured destinations with details
- SSH key paths
- Password storage method

**Output:**
```
SnapTo Configuration
────────────────────────────────────────
  Default Destination: production
  Filename Template: screenshot-{timestamp}.{ext}
  Default Format: png
────────────────────────────────────────
Destinations

ℹ production
  Type: sftp
  Host: images.example.com
  Port: 22
  Username: deploy
  Remote Path: /var/www/images
  Base URL: https://images.example.com
  SSH Key: ~/.ssh/id_rsa
```

#### 3.2 Config Edit
Opens configuration in default editor ($EDITOR).

**Features:**
- Auto-creates config if not exists
- Validates after editing
- Shows detailed validation errors
- Falls back to vi/notepad if $EDITOR not set

#### 3.3 Config Path
Prints configuration file path for scripting.

**Usage:**
```bash
# Open in custom editor
code $(snapto config path)

# Backup config
cp $(snapto config path) ~/backup/
```

#### 3.4 Config Init
Creates default configuration file.

**Features:**
- Checks for existing config
- Creates directory structure if needed
- Generates default TOML with examples
- Shows next steps after creation

### 4. History

View upload history from SQLite database.

**Display Modes:**

#### 4.1 Compact Mode (default)
```
#10 2024-01-15 14:41:25 screenshot.png (892.1 KB) https://...
#9  2024-01-15 14:40:01 image.png (445.2 KB) https://...
```

**Features:**
- One line per upload
- Colored output (cyan timestamp, blue URL)
- Dimmed metadata
- Quick overview

#### 4.2 Full Mode (--full)
```
ℹ Upload #3
  URL: https://...
  Filename: screenshot.png
  Size: 892.1 KB
  Destination: production
  Uploaded: 2024-01-15 14:41:25
```

**Features:**
- Detailed view
- All metadata visible
- Easy to read

**Options:**
- `-l, --limit <N>`: Number of entries (default 10)
- `-f, --full`: Full detailed view

## Visual Design

### Color Scheme

| Element | Color | Symbol |
|---------|-------|--------|
| Success | Green | ✓ |
| Error | Red | ✗ |
| Warning | Yellow | ⚠ |
| Info | Blue | ℹ |
| Step | Cyan | → |
| Separator | Dimmed | ──── |

### Progress Indicators

#### Upload Progress Bar
```
⠋ [████████████████░░░░] 342 KB/512 KB (2.3s)
```
- Spinner animation
- Visual bar with filled/unfilled sections
- Current/total size
- Estimated time remaining

#### Simple Spinner
```
⠋ Uploading...
⠙ Uploading...
⠹ Uploading...
⠸ Uploading...
```
- Rotating animation
- Used for indeterminate operations

### Text Formatting

**Headers:**
```
SnapTo Configuration
```
- Bold + Underlined

**Key-Value Pairs:**
```
  Key: Value
```
- Key is dimmed with colon
- Value is normal weight
- 2-space indent

**List Items:**
```
  • Item 1
  • Item 2
```
- Bullet character
- 2-space indent

## Error Handling

### User-Friendly Messages

**No Image in Clipboard:**
```
✗ Error: No image found in clipboard
```

**Invalid Destination:**
```
✗ Error: Destination 'xyz' not found in configuration
```

**Connection Failed:**
```
✗ Error: Failed to connect to server: Connection refused
```

**Config Not Found:**
```
✗ Error: Failed to load configuration: config file not found
```

### Error Context

Uses `anyhow` for rich error context:

```rust
.context("Failed to read image from clipboard")?
.context("Failed to load configuration")?
.context("Failed to copy URL to clipboard")?
```

Provides clear error messages with context about what operation failed.

## Utility Functions

### File Size Formatting

Converts bytes to human-readable format:

```rust
format_size(1024)           // "1.00 KB"
format_size(1048576)        // "1.00 MB"
format_size(1073741824)     // "1.00 GB"
format_size(500)            // "500 bytes"
```

### Duration Formatting

Converts milliseconds to readable format:

```rust
format_duration(1234)       // "1.234s"
format_duration(456)        // "456ms"
format_duration(5678)       // "5.678s"
```

## Command-Line Arguments

### Global Flags

- `-v, --verbose`: Enable debug logging
- `-h, --help`: Show help
- `-V, --version`: Show version

### Upload Options

- `-d, --destination <NAME>`: Override destination
- `-f, --filename <NAME>`: Custom filename

### Watch Options

- `-i, --interval <MS>`: Check interval (default: 500)
- `-d, --destination <NAME>`: Destination

### History Options

- `-l, --limit <N>`: Number of entries (default: 10)
- `-f, --full`: Show full details

## Integration Points

### 1. snapto-core Integration

Uses core library for:
- `ClipboardManager`: Read/write clipboard
- `Config`: Configuration management
- `Uploader`: Upload logic
- `HistoryStorage`: Database operations

### 2. System Integration

**Clipboard:**
- Reads images from system clipboard
- Writes URLs back to clipboard
- Cross-platform support

**Editor:**
- Uses $EDITOR environment variable
- Falls back to platform defaults

**Paths:**
- Uses `directories` crate for config paths
- Cross-platform: Linux, macOS, Windows

### 3. Logging

**Levels:**
- Default: INFO
- Verbose (-v): DEBUG
- Environment: RUST_LOG

**Output:**
```bash
# Default
snapto upload

# Verbose
snapto -v upload

# Custom log level
RUST_LOG=trace snapto upload
```

## Performance

### Optimizations

**Clipboard Monitoring:**
- Configurable check interval
- Hash-based change detection
- Minimal CPU usage in watch mode

**Upload:**
- Streaming upload (no full buffer)
- Progress updates every 100ms
- Connection pooling (via ssh2)

**Database:**
- Indexed queries
- Prepared statements
- Lazy initialization

## Security

### Credential Handling

**SSH Keys:**
- Path-based (stored in config)
- Standard permissions expected

**Passwords:**
- Optional keyring storage
- Not stored in plain text
- Prompted when needed

**Config File:**
- Standard permissions (0600 recommended)
- TOML format (easy to audit)
- No secrets in config if using keyring

## Future Enhancements

### Planned Features

- [ ] Interactive prompts for config setup
- [ ] Bulk upload from directory
- [ ] URL shortening integration
- [ ] Notification support (desktop/sound)
- [ ] Custom upload hooks
- [ ] Cloud provider support (S3, GCS)
- [ ] Compression options
- [ ] Thumbnail generation
- [ ] Expiring uploads
- [ ] Upload limits/quotas

### Nice to Have

- [ ] Shell completions (bash, zsh, fish)
- [ ] Man page generation
- [ ] Desktop file (Linux)
- [ ] GUI wrapper option
- [ ] Plugin system
- [ ] Statistics/analytics
- [ ] Upload scheduling
- [ ] Batch operations

## Testing

### Manual Testing

```bash
# Test upload
./dev.sh upload

# Test watch mode
./dev.sh watch

# Test config
./dev.sh config

# Test history
./dev.sh history
```

### Automated Testing

```bash
# Run tests
cargo test -p snapto-cli

# With coverage
cargo tarpaulin -p snapto-cli
```

## Accessibility

### Color Blind Support

- Symbols used in addition to colors
- Can be used without color (via NO_COLOR env)
- Clear text messages

### Screen Readers

- Plain text output
- Descriptive messages
- Progress indicators with text

### Terminal Compatibility

- Works in all standard terminals
- Fallback for limited color support
- Unicode symbols with ASCII fallbacks (future)
