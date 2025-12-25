# SnapTo CLI

Command-line interface for SnapTo - Upload images from clipboard to remote servers.

## Installation

```bash
cargo install --path .
```

Or from the workspace root:

```bash
cargo build --release -p snapto-cli
```

The binary will be available at `target/release/snapto`.

## Quick Start

1. Initialize configuration:
```bash
snapto config init
```

2. Edit configuration to add your server:
```bash
snapto config edit
```

3. Upload an image from clipboard:
```bash
snapto upload
```

## Commands

### `snapto upload`

Upload an image from the clipboard to a remote server.

**Options:**
- `-d, --destination <NAME>` - Override the default destination
- `-f, --filename <NAME>` - Specify a custom filename
- `-v, --verbose` - Enable verbose output

**Example:**
```bash
# Upload to default destination
snapto upload

# Upload to specific destination
snapto upload -d production

# Upload with custom filename
snapto upload -f "screenshot.png"
```

**Workflow:**
1. Reads image from clipboard
2. Uploads to configured SSH/SFTP server
3. Copies resulting URL to clipboard
4. Saves upload to history

---

### `snapto watch`

Continuously watch the clipboard for new images and automatically upload them.

**Options:**
- `-i, --interval <MS>` - Check interval in milliseconds (default: 500)
- `-d, --destination <NAME>` - Destination to upload to
- `-v, --verbose` - Enable verbose output

**Example:**
```bash
# Start watching with default settings
snapto watch

# Watch with custom interval
snapto watch -i 1000

# Watch and upload to specific destination
snapto watch -d staging
```

**Features:**
- Detects new images automatically
- Only uploads when clipboard content changes
- Shows upload progress and statistics
- Copies URL to clipboard after each upload
- Press Ctrl+C to stop watching

---

### `snapto config`

Manage configuration settings.

#### Subcommands

**`snapto config show`** - Display current configuration
```bash
snapto config show
```

**`snapto config edit`** - Edit configuration in $EDITOR
```bash
snapto config edit
```

**`snapto config path`** - Show configuration file path
```bash
snapto config path
```

**`snapto config init`** - Initialize default configuration
```bash
snapto config init
```

---

### `snapto history`

Show upload history.

**Options:**
- `-l, --limit <N>` - Number of entries to show (default: 10)
- `-f, --full` - Show full details for each entry
- `-v, --verbose` - Enable verbose output

**Example:**
```bash
# Show last 10 uploads
snapto history

# Show last 20 uploads
snapto history -l 20

# Show full details
snapto history --full
```

## Configuration

Configuration is stored in TOML format at:
- Linux: `~/.config/snapto/config.toml`
- macOS: `~/Library/Application Support/snapto/config.toml`
- Windows: `%APPDATA%\snapto\config.toml`

### Example Configuration

```toml
default_destination = "production"
filename_template = "screenshot-{timestamp}.{ext}"
default_format = "png"

[destinations.production]
type = "sftp"
host = "example.com"
port = 22
username = "user"
remote_path = "/var/www/uploads"
base_url = "https://example.com/uploads"
ssh_key_path = "~/.ssh/id_rsa"
use_keyring = false

[destinations.staging]
type = "sftp"
host = "staging.example.com"
port = 22
username = "user"
remote_path = "/var/www/uploads"
base_url = "https://staging.example.com/uploads"
ssh_key_path = "~/.ssh/id_rsa"
use_keyring = true
```

### Configuration Fields

- `default_destination` - Default server to upload to
- `filename_template` - Template for generated filenames
  - `{timestamp}` - Current timestamp
  - `{date}` - Current date
  - `{time}` - Current time
  - `{ext}` - File extension
- `default_format` - Default image format (png, jpg, webp)

### Destination Fields

- `type` - Connection type (sftp, ssh)
- `host` - Server hostname
- `port` - Server port (default: 22)
- `username` - SSH username
- `remote_path` - Remote directory path
- `base_url` - Base URL for uploaded files
- `ssh_key_path` - Path to SSH private key (optional)
- `use_keyring` - Store password in system keyring (optional)

## Output

SnapTo CLI provides colorful, informative output:

- **✓** Green checkmarks for success
- **✗** Red crosses for errors
- **⚠** Yellow warnings for non-critical issues
- **ℹ** Blue info messages
- **→** Cyan arrows for steps in progress

Progress bars show:
- Upload progress with percentage
- File size and transfer rate
- Estimated time remaining

## History

Upload history is stored in a SQLite database at:
- Linux: `~/.local/share/snapto/history.db`
- macOS: `~/Library/Application Support/snapto/history.db`
- Windows: `%APPDATA%\snapto\history.db`

History includes:
- Upload timestamp
- File URL
- Filename
- File size
- Destination used

## Environment Variables

- `EDITOR` - Editor to use for `snapto config edit` (default: vi/notepad)
- `SNAPTO_CONFIG` - Override config file path
- `RUST_LOG` - Control logging level (e.g., `RUST_LOG=debug snapto upload`)

## Exit Codes

- `0` - Success
- `1` - Error occurred

## Examples

### Basic Workflow

```bash
# Take a screenshot (OS-specific)
# The screenshot is now in your clipboard

# Upload it
snapto upload

# The URL is now in your clipboard, ready to paste
```

### Watch Mode Workflow

```bash
# Start watch mode
snapto watch

# Now just take screenshots or copy images
# They'll be automatically uploaded
# URLs will be copied to clipboard
```

### Multiple Destinations

```bash
# Upload to staging first
snapto upload -d staging

# After testing, upload to production
snapto upload -d production
```

## Troubleshooting

**No image found in clipboard:**
- Ensure you've copied an image (not text or file path)
- Try copying the image again

**Connection failed:**
- Check your SSH credentials
- Verify server is accessible
- Check firewall settings

**Configuration not found:**
- Run `snapto config init` to create default config
- Check config path with `snapto config path`

**Permission denied:**
- Verify SSH key permissions (should be 600)
- Check remote directory permissions

## Development

```bash
# Run from source
cargo run -p snapto-cli -- upload

# Run with verbose logging
cargo run -p snapto-cli -- -v upload

# Build release version
cargo build --release -p snapto-cli
```

## License

MIT
