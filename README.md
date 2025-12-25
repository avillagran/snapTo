# SnapTo

**The fastest way to share screenshots with AI assistants and remote teams.**

Cross-platform screenshot tool that captures, uploads to your server via SFTP/SSH, and copies the URL to your clipboard in seconds. Built for developers who work with AI coding assistants like **Claude Code**, **Gemini CLI**, **GitHub Copilot**, **Cursor**, **Grok**, and others that need to see images on remote servers.

## Why SnapTo?

When working with AI assistants on remote servers or in CLI environments, sharing screenshots is painful:

- **Problem**: AI assistants can't see images on your local machine
- **Problem**: Uploading to Imgur/other services is slow and adds friction
- **Problem**: Enterprise environments block public image hosts

**SnapTo solves this**: One hotkey captures your screen, uploads to YOUR server, and copies a direct URL. Paste it into Claude Code, Gemini, or any AI chat and they can see your screenshot instantly.

### Perfect for:

- **AI-Assisted Development**: Share error screenshots, UI bugs, or code output with Claude Code, Cursor, Windsurf, or Gemini CLI
- **Remote Pair Programming**: Quick visual sharing without screen sharing overhead
- **Bug Reports**: Capture and share with a single keystroke
- **Documentation**: Fast screenshot-to-URL workflow
- **Self-Hosted**: Your images, your server, your privacy

## Features

- **One-Click Capture**: `Cmd+Shift+3` (fullscreen) or `Cmd+Shift+4` (selection)
- **Instant Upload**: SFTP/SSH to your own server
- **URL to Clipboard**: Ready to paste immediately
- **Three Interfaces**: Menubar app, Terminal UI, or CLI
- **Secure Credentials**: OS keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Cross-Platform**: macOS, Windows, Linux support
- **Self-Hosted**: No third-party services, complete privacy
- **Customizable Naming**: Templates like `{date}_{random:6}.png`

## Quick Demo

```bash
# Capture screenshot and get URL in one command
$ snapto upload
Screenshot captured
Uploading to: my-server:/var/www/images/
URL copied: https://images.example.com/2024-01-15_a7f3x2.png

# Now paste the URL into Claude Code, Gemini, or any AI assistant!
```

## Installation

### From Source (Rust 1.70+)

```bash
git clone https://github.com/yourusername/snapto.git
cd snapto
cargo install --path crates/snapto-cli
```

### Build All (CLI + TUI + GUI)

```bash
./scripts/build-all.sh
```

## Configuration

Create `~/.config/snapto/config.toml`:

```toml
[general]
default_destination = "my-server"

[naming]
template = "{date}_{random:6}"
date_format = "%Y-%m-%d"

[destinations.my-server]
type = "sftp"
host = "images.example.com"
port = 22
username = "deploy"
use_key_auth = true
key_path = "~/.ssh/id_rsa"
remote_path = "/var/www/images/"
url_template = "https://images.example.com/{filename}"
```

## Usage

### CLI

```bash
snapto upload              # Upload from clipboard
snapto upload screenshot.png  # Upload specific file
snapto watch               # Auto-upload clipboard images
snapto config show         # Show configuration
snapto history             # View upload history
```

### TUI (Terminal UI)

```bash
snapto-tui
```

Interactive terminal interface with:
- Quick upload from home screen
- Browse and re-upload history
- Configure destinations and credentials
- Vim-style keybindings

### GUI (Menubar App)

Run the Flutter app for a native menubar experience:
- System tray icon with quick actions
- Global hotkeys (`Cmd+Shift+3`, `Cmd+Shift+4`)
- Native notifications
- Bundled CLI for seamless integration

## Use Cases

### Working with Claude Code

```bash
# You're debugging with Claude Code and need to show an error
$ snapto upload
URL: https://images.example.com/error_2024-01-15.png

# Paste URL in Claude Code chat:
> "Here's the error I'm seeing: https://images.example.com/error_2024-01-15.png"
# Claude can now see and analyze your screenshot!
```

### Remote Development

When SSH'd into a server, use the TUI to configure uploads and share screenshots with team members or AI assistants.

### CI/CD Screenshots

```bash
# In your test script
screenshot-tool capture output.png
snapto upload output.png
# URL is now in clipboard for your PR comment
```

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│  Flutter GUI │ Rust TUI │ Rust CLI                            │
│  (menubar)   │(ratatui) │ (clap)                              │
└───────────────────────────────────────────────────────────────┘
                           │
┌───────────────────────────────────────────────────────────────┐
│                    snapto-core (Rust)                         │
│  Clipboard │ Screenshot │ SFTP Upload │ History │ Keychain    │
└───────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+3` | Capture fullscreen |
| `Cmd+Shift+4` | Capture selected area |

## Project Structure

```
snapto/
├── crates/
│   ├── snapto-core/     # Shared Rust library
│   ├── snapto-cli/      # Command-line interface
│   └── snapto-tui/      # Terminal user interface
├── snapto_app/          # Flutter menubar app
└── scripts/             # Build scripts
```

## Security

- **No cloud dependency**: Your images stay on YOUR server
- **Secure credentials**: OS keychain with AES-256-GCM fallback
- **SSH key auth**: No passwords stored in config files
- **Private by default**: No analytics, no telemetry

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE)

## Related Projects

- [ShareX](https://getsharex.com/) - Windows screenshot tool (inspiration)
- [Skitch](https://evernote.com/products/skitch) - macOS screenshot tool (inspiration)
- [Flameshot](https://flameshot.org/) - Linux screenshot tool

---

**Built for developers who talk to AI.** Share your screen, not your privacy.

Keywords: screenshot tool, image upload, SFTP upload, SSH upload, Claude Code, Gemini CLI, AI assistant, screen capture, clipboard upload, self-hosted, developer tools, remote work, pair programming
