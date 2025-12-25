# SnapTo CLI Examples

## Setup

### First Time Setup

```bash
# 1. Initialize configuration
$ snapto config init
✓ Configuration initialized at: /Users/user/.config/snapto/config.toml

Next steps:
  • Edit the configuration: snapto config edit
  • Add your SSH/SFTP server details
  • Set your default destination
  • Test upload: snapto upload

# 2. Edit configuration
$ snapto config edit
ℹ Opening /Users/user/.config/snapto/config.toml in vim...
→ Validating configuration...
✓ Configuration is valid!

# 3. Verify configuration
$ snapto config show

SnapTo Configuration
────────────────────────────────────────────────────────────────────────────────
  Default Destination: production
  Filename Template: screenshot-{timestamp}.{ext}
  Default Format: png
────────────────────────────────────────────────────────────────────────────────

Destinations

ℹ production
  Type: sftp
  Host: images.example.com
  Port: 22
  Username: deploy
  Remote Path: /var/www/images
  Base URL: https://images.example.com
  SSH Key: ~/.ssh/id_rsa
────────────────────────────────────────────────────────────────────────────────
```

## Basic Upload

### Single Upload

```bash
# Copy an image to clipboard (e.g., take a screenshot)
# Then upload it

$ snapto upload
→ Reading image from clipboard...
✓ Found image in clipboard (342.5 KB)
→ Loading configuration...
ℹ Using destination: production
⠋ Uploading image...
✓ Upload completed!
────────────────────────────────────────────────────────────────────────────────
  URL: https://images.example.com/screenshot-20240115-143022.png
  Size: 342.5 KB
  Duration: 1.234s
  Speed: 277.4 KB/s
  Filename: screenshot-20240115-143022.png
────────────────────────────────────────────────────────────────────────────────
→ Copying URL to clipboard...
✓ URL copied to clipboard!
```

### Upload to Specific Destination

```bash
$ snapto upload -d staging
→ Reading image from clipboard...
✓ Found image in clipboard (128.3 KB)
→ Loading configuration...
ℹ Using destination: staging
⠋ Uploading image...
✓ Upload completed!
────────────────────────────────────────────────────────────────────────────────
  URL: https://staging.example.com/screenshot-20240115-143522.png
  Size: 128.3 KB
  Duration: 0.856s
  Speed: 149.9 KB/s
────────────────────────────────────────────────────────────────────────────────
→ Copying URL to clipboard...
✓ URL copied to clipboard!
```

### Upload with Custom Filename

```bash
$ snapto upload -f "bug-report-login-screen.png"
→ Reading image from clipboard...
✓ Found image in clipboard (256.8 KB)
→ Loading configuration...
ℹ Using destination: production
⠋ Uploading image...
✓ Upload completed!
────────────────────────────────────────────────────────────────────────────────
  URL: https://images.example.com/bug-report-login-screen.png
  Size: 256.8 KB
  Duration: 1.102s
  Speed: 233.0 KB/s
  Filename: bug-report-login-screen.png
────────────────────────────────────────────────────────────────────────────────
→ Copying URL to clipboard...
✓ URL copied to clipboard!
```

## Watch Mode

### Basic Watch

```bash
$ snapto watch

SnapTo Watch Mode
ℹ Watching clipboard for images...
ℹ Check interval: 500ms
ℹ Press Ctrl+C to stop
────────────────────────────────────────────────────────────────────────────────
ℹ Using destination: production
────────────────────────────────────────────────────────────────────────────────
ℹ Waiting for next image...

→ New image detected (445.2 KB)
⠋ Uploading...
✓ Upload #1 completed!
  URL: https://images.example.com/screenshot-20240115-144001.png
  Duration: 1.345s
ℹ URL copied to clipboard
────────────────────────────────────────────────────────────────────────────────
ℹ Waiting for next image...

→ New image detected (892.1 KB)
⠋ Uploading...
✓ Upload #2 completed!
  URL: https://images.example.com/screenshot-20240115-144125.png
  Duration: 2.156s
ℹ URL copied to clipboard
────────────────────────────────────────────────────────────────────────────────
ℹ Waiting for next image...

^C
```

### Watch with Custom Interval

```bash
$ snapto watch -i 1000

SnapTo Watch Mode
ℹ Watching clipboard for images...
ℹ Check interval: 1000ms
ℹ Press Ctrl+C to stop
────────────────────────────────────────────────────────────────────────────────
ℹ Using destination: production
────────────────────────────────────────────────────────────────────────────────
ℹ Waiting for next image...
```

### Watch with Specific Destination

```bash
$ snapto watch -d staging -i 250

SnapTo Watch Mode
ℹ Watching clipboard for images...
ℹ Check interval: 250ms
ℹ Press Ctrl+C to stop
────────────────────────────────────────────────────────────────────────────────
ℹ Using destination: staging
────────────────────────────────────────────────────────────────────────────────
ℹ Waiting for next image...
```

## History

### View Recent Uploads

```bash
$ snapto history

Upload History
ℹ Showing last 10 upload(s)
────────────────────────────────────────────────────────────────────────────────
#10 2024-01-15 14:41:25 screenshot-20240115-144125.png (892.1 KB) https://images.example.com/screenshot-20240115-144125.png
#9 2024-01-15 14:40:01 screenshot-20240115-144001.png (445.2 KB) https://images.example.com/screenshot-20240115-144001.png
#8 2024-01-15 14:35:22 screenshot-20240115-143522.png (128.3 KB) https://staging.example.com/screenshot-20240115-143522.png
#7 2024-01-15 14:30:22 screenshot-20240115-143022.png (342.5 KB) https://images.example.com/screenshot-20240115-143022.png
#6 2024-01-15 14:25:18 bug-report-login-screen.png (256.8 KB) https://images.example.com/bug-report-login-screen.png
#5 2024-01-15 14:20:45 screenshot-20240115-142045.png (512.4 KB) https://images.example.com/screenshot-20240115-142045.png
#4 2024-01-15 14:15:33 screenshot-20240115-141533.png (678.9 KB) https://images.example.com/screenshot-20240115-141533.png
#3 2024-01-15 14:10:12 screenshot-20240115-141012.png (234.1 KB) https://images.example.com/screenshot-20240115-141012.png
#2 2024-01-15 14:05:58 screenshot-20240115-140558.png (445.7 KB) https://images.example.com/screenshot-20240115-140558.png
#1 2024-01-15 14:00:00 screenshot-20240115-140000.png (389.2 KB) https://images.example.com/screenshot-20240115-140000.png

ℹ Use --full flag for detailed view
────────────────────────────────────────────────────────────────────────────────
```

### View Full Details

```bash
$ snapto history --full -l 3

Upload History
ℹ Showing last 3 upload(s)
────────────────────────────────────────────────────────────────────────────────

ℹ Upload #3
  URL: https://images.example.com/screenshot-20240115-144125.png
  Filename: screenshot-20240115-144125.png
  Size: 892.1 KB
  Destination: production
  Uploaded: 2024-01-15 14:41:25

ℹ Upload #2
  URL: https://images.example.com/screenshot-20240115-144001.png
  Filename: screenshot-20240115-144001.png
  Size: 445.2 KB
  Destination: production
  Uploaded: 2024-01-15 14:40:01

ℹ Upload #1
  URL: https://staging.example.com/screenshot-20240115-143522.png
  Filename: screenshot-20240115-143522.png
  Size: 128.3 KB
  Destination: staging
  Uploaded: 2024-01-15 14:35:22
────────────────────────────────────────────────────────────────────────────────
```

### View More Entries

```bash
$ snapto history -l 50
# Shows last 50 uploads
```

## Error Handling

### No Image in Clipboard

```bash
$ snapto upload
→ Reading image from clipboard...
✗ Error: No image found in clipboard
```

### Invalid Destination

```bash
$ snapto upload -d nonexistent
→ Reading image from clipboard...
✓ Found image in clipboard (234.5 KB)
→ Loading configuration...
✗ Error: Destination 'nonexistent' not found in configuration
```

### Connection Error

```bash
$ snapto upload
→ Reading image from clipboard...
✓ Found image in clipboard (234.5 KB)
→ Loading configuration...
ℹ Using destination: production
⠋ Uploading image...
✗ Error: Failed to connect to server: Connection refused
```

### Configuration Error

```bash
$ snapto upload
✗ Error: Failed to load configuration: config file not found

# Fix by initializing
$ snapto config init
✓ Configuration initialized at: /Users/user/.config/snapto/config.toml
```

## Advanced Usage

### Verbose Mode

```bash
$ snapto -v upload
DEBUG snapto::clipboard - Initializing clipboard manager
DEBUG snapto::clipboard - Reading image from clipboard
DEBUG snapto::clipboard - Found image: PNG format, 1920x1080, 342500 bytes
→ Reading image from clipboard...
✓ Found image in clipboard (342.5 KB)
DEBUG snapto::config - Loading config from: /Users/user/.config/snapto/config.toml
DEBUG snapto::config - Config loaded successfully
→ Loading configuration...
ℹ Using destination: production
DEBUG snapto::uploader - Connecting to images.example.com:22
DEBUG snapto::uploader - Connected, authenticating with key
DEBUG snapto::uploader - Authenticated successfully
DEBUG snapto::uploader - Opening SFTP session
DEBUG snapto::uploader - Uploading to /var/www/images/screenshot-20240115-143022.png
⠋ Uploading image...
DEBUG snapto::uploader - Upload complete, 342500 bytes transferred
✓ Upload completed!
────────────────────────────────────────────────────────────────────────────────
  URL: https://images.example.com/screenshot-20240115-143022.png
  Size: 342.5 KB
  Duration: 1.234s
  Speed: 277.4 KB/s
  Filename: screenshot-20240115-143022.png
────────────────────────────────────────────────────────────────────────────────
DEBUG snapto::clipboard - Setting clipboard text
→ Copying URL to clipboard...
✓ URL copied to clipboard!
DEBUG snapto::storage - Saving upload to history database
```

### Scripting Integration

```bash
#!/bin/bash
# auto-upload.sh - Automatically upload and post to Slack

# Upload image
URL=$(snapto upload 2>&1 | grep "URL:" | awk '{print $2}')

if [ $? -eq 0 ]; then
    echo "Uploaded: $URL"

    # Post to Slack
    curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
        -H 'Content-Type: application/json' \
        -d "{\"text\": \"New screenshot: $URL\"}"
else
    echo "Upload failed"
    exit 1
fi
```

### Workflow Integration

```bash
# Take screenshot (macOS)
screencapture -i -c

# Upload and get URL
snapto upload

# URL is now in clipboard, paste into issue tracker, chat, etc.
```

## Tips & Tricks

### Quick Screenshot Upload

Create an alias for your shell:

```bash
# .bashrc or .zshrc
alias sup='screencapture -i -c && snapto upload'
```

Now you can:
```bash
$ sup
# Takes screenshot, uploads it, copies URL to clipboard
```

### Multiple Destinations Workflow

```bash
# Test on staging first
$ snapto upload -d staging
✓ Upload completed!
  URL: https://staging.example.com/screenshot.png

# If looks good, upload to production
$ snapto upload -d production
✓ Upload completed!
  URL: https://images.example.com/screenshot.png
```

### Watch Mode for Demos

```bash
# Start watch mode before demo
$ snapto watch -i 1000

# During demo, just take screenshots
# URLs are automatically copied to clipboard
# Paste them directly into chat/docs
```

### Review Recent Uploads

```bash
# Quick review of today's uploads
$ snapto history -l 50 | grep $(date +%Y-%m-%d)

# Copy a previous URL
$ snapto history --full -l 20
# Find the URL you want, copy it manually
```
