# SnapTo macOS App - Setup Guide

This guide will help you set up and run the SnapTo macOS menubar application.

## Prerequisites

### 1. Install Flutter

If you haven't installed Flutter yet:

```bash
# Install Flutter using Homebrew
brew install --cask flutter

# Or download from https://docs.flutter.dev/get-started/install/macos
```

### 2. Verify Flutter Installation

```bash
flutter doctor
```

Make sure you have:
- Flutter SDK installed
- Xcode installed and configured
- CocoaPods installed

### 3. Enable macOS Desktop Support

```bash
flutter config --enable-macos-desktop
```

## Setup Steps

### 1. Navigate to Project Directory

```bash
cd /Users/avillagran/Desarrollo/ClipClaude/snapto_app
```

### 2. Get Flutter Dependencies

```bash
flutter pub get
```

This will download all required packages:
- tray_manager
- screen_capturer
- hotkey_manager
- window_manager
- local_notifier
- path_provider

### 3. Build SnapTo CLI (if not already built)

```bash
cd /Users/avillagran/Desarrollo/ClipClaude
cargo build --release
```

Verify the binary exists:
```bash
ls -la target/release/snapto
```

### 4. Configure macOS Permissions (Important!)

The app requires screen recording permissions. You'll need to:

1. Run the app for the first time
2. macOS will prompt you to grant Screen Recording permission
3. Go to System Preferences > Security & Privacy > Screen Recording
4. Enable permission for SnapTo
5. Restart the app

For global hotkeys:
1. Go to System Preferences > Security & Privacy > Accessibility
2. Enable permission for SnapTo
3. Restart the app

## Running the App

### Development Mode

```bash
cd /Users/avillagran/Desarrollo/ClipClaude/snapto_app
flutter run -d macos
```

The app will:
- Launch without a dock icon (menubar only)
- Show an icon in the menubar (top-right)
- Register global hotkeys (Cmd+Shift+3, Cmd+Shift+4)

### Debug with Logs

```bash
flutter run -d macos --verbose
```

### Hot Reload

While the app is running, press:
- `r` - Hot reload
- `R` - Hot restart
- `q` - Quit

## Building for Release

### Build Release Binary

```bash
flutter build macos --release
```

### Locate the App

The built app will be at:
```
build/macos/Build/Products/Release/SnapTo.app
```

### Install to Applications

```bash
cp -r build/macos/Build/Products/Release/SnapTo.app /Applications/
```

### Run from Applications

```bash
open /Applications/SnapTo.app
```

Or double-click SnapTo.app in Finder.

## Testing the App

### 1. Check Menubar Icon

- Look for the SnapTo icon in the menubar (top-right corner)
- Click it to see the menu

### 2. Test Screenshot Capture

#### Via Menu:
1. Click the menubar icon
2. Select "Fullscreen Snap" or "Selected Area Snap"
3. For selected area, drag to select a region
4. Check notification for upload status

#### Via Hotkeys:
1. Press Cmd+Shift+3 for fullscreen
2. Press Cmd+Shift+4 for selected area
3. For selected area, drag to select a region
4. Check notification for upload status

### 3. Test TUI Launch

1. Click menubar icon
2. Select "Open TUI"
3. Terminal should open with SnapTo TUI

### 4. Check Upload

After capturing a screenshot:
1. Check notification for success/failure
2. If successful, URL should be copied to clipboard
3. Paste (Cmd+V) to verify

## Troubleshooting

### Flutter Not Found

```bash
# Add Flutter to PATH
export PATH="$PATH:`pwd`/flutter/bin"

# Or install via Homebrew
brew install --cask flutter
```

### Dependencies Failed to Download

```bash
# Clear pub cache and retry
flutter pub cache clean
flutter pub get
```

### Screenshot Capture Fails

**Error**: "Screen recording permission denied"

**Solution**:
1. System Preferences > Security & Privacy > Screen Recording
2. Enable SnapTo in the list
3. Restart the app

### Hotkeys Don't Work

**Error**: Hotkeys not triggering screenshots

**Solution**:
1. System Preferences > Security & Privacy > Accessibility
2. Enable SnapTo in the list
3. Restart the app

### Upload Fails

**Error**: "SnapTo binary not found"

**Solution**:
```bash
# Build the SnapTo CLI
cd /Users/avillagran/Desarrollo/ClipClaude
cargo build --release

# Verify it exists
ls -la target/release/snapto
```

**Error**: Upload command fails

**Solution**:
```bash
# Test the CLI manually
./target/release/snapto --version

# Try uploading a test file
./target/release/snapto upload /path/to/test.png
```

### App Doesn't Show in Menubar

**Issue**: No menubar icon appears

**Solutions**:
1. Check console logs for tray initialization errors
2. Verify LSUIElement is set to true in Info.plist
3. Try restarting the app
4. Check if tray_icon.png exists in assets/

### App Shows in Dock

**Issue**: App appears in dock instead of menubar only

**Solutions**:
1. Verify Info.plist has `LSUIElement = true`
2. Check AppDelegate.swift sets activation policy to `.accessory`
3. Rebuild the app: `flutter build macos --release`

## Development Tips

### Viewing Logs

```bash
# In another terminal while app is running
flutter logs
```

### Debugging

1. Run with verbose output:
```bash
flutter run -d macos --verbose
```

2. Use Xcode for native debugging:
```bash
open macos/Runner.xcworkspace
```

### Code Changes

After modifying Dart code:
1. Press `r` for hot reload (fast, preserves state)
2. Press `R` for hot restart (slower, resets state)

After modifying native code (Swift, Info.plist):
1. Stop the app (`q`)
2. Run again: `flutter run -d macos`

### Adding Assets

1. Add files to `assets/` directory
2. Update `pubspec.yaml` if needed
3. Run `flutter pub get`
4. Restart the app

## Configuration

### Change SnapTo CLI Path

Edit `lib/services/upload_service.dart`:

```dart
final String _snaptoPath = '/your/custom/path/to/snapto';
```

### Customize Menu Items

Edit `lib/services/tray_service.dart`:

```dart
Menu menu = Menu(
  items: [
    // Add or modify menu items here
  ],
);
```

### Customize Hotkeys

Edit `lib/services/hotkey_service.dart`:

```dart
// Change key combinations
_fullscreenHotkey = HotKey(
  key: PhysicalKeyboardKey.digit3,  // Change key
  modifiers: [HotKeyModifier.meta, HotKeyModifier.shift],  // Change modifiers
  scope: HotKeyScope.system,
);
```

## Next Steps

1. **Add Tray Icon**: Create a 22x22 PNG icon and place it at `assets/tray_icon.png`
2. **Implement Settings**: Build a settings UI for configuration
3. **Add Tests**: Write unit and integration tests
4. **Create Installer**: Package as DMG for distribution
5. **Code Signing**: Sign the app for distribution outside App Store

## Support

For issues or questions:
1. Check console logs: `flutter logs`
2. Review error messages in the terminal
3. Verify all prerequisites are installed
4. Check macOS permissions are granted

## Resources

- [Flutter macOS Documentation](https://docs.flutter.dev/platform-integration/macos/building)
- [tray_manager Package](https://pub.dev/packages/tray_manager)
- [screen_capturer Package](https://pub.dev/packages/screen_capturer)
- [hotkey_manager Package](https://pub.dev/packages/hotkey_manager)
- [window_manager Package](https://pub.dev/packages/window_manager)
