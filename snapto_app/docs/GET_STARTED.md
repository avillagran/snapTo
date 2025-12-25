# Get Started with SnapTo macOS App

Welcome! This guide will get you running the SnapTo menubar app in minutes.

## What You Have

A complete Flutter macOS menubar application with:
- 30 source and configuration files
- Full menubar integration (no dock icon)
- Screenshot capture (fullscreen & region)
- Global hotkeys (Cmd+Shift+3, Cmd+Shift+4)
- SnapTo CLI integration for uploads
- System notifications
- Complete documentation

## Prerequisites Check

Before starting, verify you have:

### 1. Flutter SDK

```bash
flutter --version
```

If not installed:
```bash
brew install --cask flutter
# OR download from https://flutter.dev
```

### 2. Xcode

```bash
xcode-select --version
```

If not installed:
- Download from Mac App Store
- Run: `xcode-select --install`

### 3. SnapTo CLI (Rust)

The SnapTo CLI should be at:
```
/Users/avillagran/Desarrollo/ClipClaude/target/release/snapto
```

Build it if needed:
```bash
cd /Users/avillagran/Desarrollo/ClipClaude
cargo build --release
```

## Quick Start (3 Steps)

### Step 1: Install Dependencies

```bash
cd /Users/avillagran/Desarrollo/ClipClaude/snapto_app
flutter pub get
```

This downloads all required Flutter packages.

### Step 2: Run the App

```bash
./run.sh
```

Or manually:
```bash
flutter run -d macos
```

### Step 3: Grant Permissions

When the app runs, macOS will prompt for permissions:

1. **Screen Recording**
   - Click "Open System Preferences"
   - Enable "SnapTo" in Screen Recording
   - Close System Preferences

2. **Accessibility** (for hotkeys)
   - System Preferences > Security & Privacy > Accessibility
   - Click lock to make changes
   - Add and enable "SnapTo"

3. **Restart the App**
   - Press `q` in terminal to quit
   - Run `./run.sh` again

## Using the App

### Finding the Menubar Icon

Look in the **top-right corner** of your screen. You should see the SnapTo icon in the menubar (next to WiFi, battery, etc.).

### Menu Options

Click the menubar icon to see:
- **Fullscreen Snap**: Capture entire screen
- **Selected Area Snap**: Select region to capture
- **Open TUI**: Launch SnapTo Terminal UI
- **Settings**: Open settings (TODO)
- **Quit SnapTo**: Exit the app

### Using Hotkeys

- **Cmd+Shift+3**: Fullscreen screenshot
- **Cmd+Shift+4**: Region selection screenshot

### Capture Flow

1. Trigger capture (menu or hotkey)
2. For region: drag to select area
3. Notification: "Uploading..."
4. Notification: "URL copied to clipboard"
5. Paste (Cmd+V) to use the URL

## What Happens After Capture

1. Screenshot saved to temp file
2. Uploaded via SnapTo CLI
3. URL extracted from CLI output
4. URL copied to clipboard
5. Notification shown with result
6. Temp file deleted

## Development Mode

While running `flutter run -d macos`, you can:

- **Hot Reload**: Press `r` (preserves state)
- **Hot Restart**: Press `R` (full restart)
- **View Logs**: Press `l`
- **Quit**: Press `q`

## Building for Production

### Build Release Binary

```bash
./build.sh
```

Or manually:
```bash
flutter build macos --release
```

### Install to Applications

```bash
cp -r build/macos/Build/Products/Release/SnapTo.app /Applications/
```

### Run from Applications

```bash
open /Applications/SnapTo.app
```

Or find "SnapTo" in Spotlight and launch.

## Common Issues

### Issue: "Flutter not found"

**Solution**:
```bash
brew install --cask flutter
flutter config --enable-macos-desktop
```

### Issue: "No menubar icon appears"

**Solutions**:
1. Look carefully in top-right corner
2. Check console for errors
3. Verify app is running (check Activity Monitor)
4. Try restarting the app

### Issue: "Screenshot fails" or "Permission denied"

**Solution**:
1. System Preferences > Security & Privacy
2. Go to "Screen Recording" tab
3. Find and enable "SnapTo"
4. Restart the app

### Issue: "Hotkeys don't work"

**Solution**:
1. System Preferences > Security & Privacy
2. Go to "Accessibility" tab
3. Find and enable "SnapTo"
4. Restart the app

### Issue: "Upload fails"

**Solutions**:

1. Verify CLI exists:
   ```bash
   ls -la /Users/avillagran/Desarrollo/ClipClaude/target/release/snapto
   ```

2. Build CLI if missing:
   ```bash
   cd /Users/avillagran/Desarrollo/ClipClaude
   cargo build --release
   ```

3. Test CLI manually:
   ```bash
   ./target/release/snapto --version
   ```

### Issue: App shows in Dock

**Solution**:
- This shouldn't happen with LSUIElement = true
- If it does, rebuild: `flutter clean && flutter build macos --release`

## Project Structure

```
snapto_app/
├── lib/
│   ├── main.dart                    # App entry point
│   └── services/                    # Service layer
│       ├── tray_service.dart        # Menubar
│       ├── screenshot_service.dart  # Capture
│       ├── upload_service.dart      # CLI integration
│       ├── hotkey_service.dart      # Shortcuts
│       └── notification_service.dart # Notifications
├── macos/
│   └── Runner/                      # macOS configuration
│       ├── Info.plist               # LSUIElement, permissions
│       ├── *.entitlements           # Sandbox, capabilities
│       └── *.swift                  # App delegate
└── Documentation and scripts...
```

## Documentation

The project includes comprehensive documentation:

- **README.md**: Overview and features
- **SETUP.md**: Detailed setup instructions
- **QUICKSTART.md**: 3-step quick start
- **ARCHITECTURE.md**: System architecture and design
- **PROJECT_SUMMARY.md**: Complete project summary
- **FILES.txt**: Complete file list
- **GET_STARTED.md**: This file

## Configuration

### Change SnapTo CLI Path

Edit `lib/services/upload_service.dart`:

```dart
final String _snaptoPath = '/your/custom/path/to/snapto';
```

### Customize Menu Items

Edit `lib/services/tray_service.dart` and modify the `Menu` items.

### Customize Hotkeys

Edit `lib/services/hotkey_service.dart` and change the `HotKey` definitions.

## Next Steps

After getting the app running:

1. **Test all features**:
   - Fullscreen capture
   - Region selection
   - Hotkeys
   - Upload
   - TUI launcher

2. **Add tray icon**:
   - Create 22x22 PNG icon
   - Save as `assets/tray_icon.png`
   - Restart app

3. **Customize**:
   - Adjust menu items
   - Configure hotkeys
   - Set upload preferences

4. **Build for release**:
   - Run `./build.sh`
   - Test release build
   - Install to Applications

5. **Implement settings UI**:
   - Add preferences window
   - Save configuration
   - Allow runtime changes

## Getting Help

If you encounter issues:

1. **Check logs**:
   ```bash
   flutter logs
   ```

2. **Review documentation**:
   - See SETUP.md for detailed troubleshooting
   - Check ARCHITECTURE.md for technical details

3. **Verify prerequisites**:
   - Flutter installed and working
   - SnapTo CLI built
   - Permissions granted

4. **Clean and rebuild**:
   ```bash
   flutter clean
   flutter pub get
   flutter run -d macos
   ```

## Success Checklist

- [ ] Flutter SDK installed
- [ ] Dependencies downloaded (`flutter pub get`)
- [ ] SnapTo CLI built
- [ ] App running (`./run.sh`)
- [ ] Menubar icon visible
- [ ] Screen Recording permission granted
- [ ] Accessibility permission granted
- [ ] Screenshot capture works
- [ ] Upload successful
- [ ] URL copied to clipboard
- [ ] Hotkeys working
- [ ] Notifications showing

When all items are checked, you're ready to use SnapTo!

## Resources

- [Flutter Documentation](https://docs.flutter.dev)
- [Flutter macOS](https://docs.flutter.dev/platform-integration/macos/building)
- [tray_manager Package](https://pub.dev/packages/tray_manager)
- [screen_capturer Package](https://pub.dev/packages/screen_capturer)
- [hotkey_manager Package](https://pub.dev/packages/hotkey_manager)

## Support

For issues specific to this implementation:
1. Check console output
2. Review service logs in `debugPrint()` statements
3. Verify configuration files
4. Test SnapTo CLI independently

---

**You're all set!** Run `./run.sh` and start capturing screenshots.

Happy snapping! 📸
