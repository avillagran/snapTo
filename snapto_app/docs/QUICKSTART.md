# SnapTo App - Quick Start

Get up and running in 3 steps:

## 1. Install Dependencies

```bash
cd /Users/avillagran/Desarrollo/ClipClaude/snapto_app
flutter pub get
```

## 2. Build SnapTo CLI (if needed)

```bash
cd /Users/avillagran/Desarrollo/ClipClaude
cargo build --release
```

## 3. Run the App

```bash
cd /Users/avillagran/Desarrollo/ClipClaude/snapto_app
flutter run -d macos
```

## Usage

### Menubar

Click the SnapTo icon in the menubar (top-right) to access:
- Fullscreen Snap
- Selected Area Snap
- Open TUI
- Settings
- Quit

### Hotkeys

- **Cmd+Shift+3**: Capture fullscreen
- **Cmd+Shift+4**: Capture selected area

### First Run

Grant permissions when prompted:
1. **Screen Recording**: System Preferences > Security & Privacy > Screen Recording
2. **Accessibility**: System Preferences > Security & Privacy > Accessibility

## Troubleshooting

### No menubar icon?
- Check that the app is running (`flutter run -d macos`)
- Look in the top-right corner of the screen
- Restart the app

### Screenshots not working?
- Grant Screen Recording permission in System Preferences
- Restart the app after granting permission

### Hotkeys not working?
- Grant Accessibility permission in System Preferences
- Restart the app after granting permission

### Upload fails?
- Verify SnapTo CLI is built: `ls -la ../target/release/snapto`
- Test CLI manually: `../target/release/snapto --version`

## Build for Release

```bash
flutter build macos --release
cp -r build/macos/Build/Products/Release/SnapTo.app /Applications/
```

Then launch from Applications or Spotlight.

---

For detailed setup instructions, see [SETUP.md](SETUP.md)
