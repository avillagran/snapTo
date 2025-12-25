# SnapTo TUI

Terminal User Interface for SnapTo screenshot sharing tool.

## Features

### Multiple Screens

1. **Home Screen**
   - Clipboard status (shows if an image is present)
   - List of configured upload destinations
   - Last upload information

2. **History Screen**
   - Table showing all previous uploads
   - Columns: Date, Filename, Destination, Size, URL
   - Navigate with arrow keys or j/k
   - Press Enter or 'c' to copy URL to clipboard
   - Press 'd' to delete an entry

3. **Settings Screen**
   - Five configuration sections:
     - General: Local save directory, clipboard settings, default uploader
     - Naming: Filename templates and formats
     - History: Enable/disable, retention, max entries
     - Uploads: List of configured destinations
     - Security: Keychain and encryption settings
   - Navigate sections with arrow keys or h/l
   - Save changes with Ctrl+S

4. **Upload Screen**
   - Shows upload progress with progress bar
   - Displays destination information
   - Shows success/error status with URL

## Navigation

### Global Keybindings

- `Tab` / `Shift+Tab` - Switch between screens
- `Ctrl+Q` - Quit application
- `Ctrl+U` - Quick upload from any screen
- `?` - Show help (future feature)

### Screen-Specific Keybindings

**Home Screen:**
- `u` - Upload manually
- `r` - Refresh clipboard status

**History Screen:**
- `j` / `↓` - Move down
- `k` / `↑` - Move up
- `Enter` / `c` - Copy selected URL to clipboard
- `d` - Delete selected entry

**Settings Screen:**
- `h` / `←` - Previous section
- `l` / `→` - Next section
- `j` / `↓` - Move down in section
- `k` / `↑` - Move up in section
- `Ctrl+S` - Save configuration

## Building

```bash
cargo build -p snapto-tui
```

## Running

```bash
cargo run -p snapto-tui
```

Or after building:

```bash
./target/debug/snapto-tui
```

## Configuration

The TUI reads the same configuration file as the CLI tool:

- Location: `~/.snapto/config.toml`
- History database: `~/.snapto/history.db`

## Architecture

- `main.rs` - Entry point, terminal setup
- `app.rs` - Application state and event handling
- `events.rs` - Keyboard event processing
- `ui/` - User interface modules
  - `mod.rs` - Main UI layout and header/status bar
  - `home.rs` - Home screen
  - `history.rs` - History screen with table
  - `settings.rs` - Settings screen with sections
  - `upload.rs` - Upload progress screen

## Dependencies

- `ratatui` 0.28 - Terminal UI framework
- `crossterm` 0.28 - Cross-platform terminal manipulation
- `snapto-core` - Core functionality (clipboard, upload, history)

## Dark Theme

The TUI uses a dark color scheme by default:

- Cyan borders
- Yellow highlights for active items
- Green for success/enabled states
- Red for errors/disabled states
- White/Dark Gray for text
