# SnapTo macOS Menubar App

A Flutter-based macOS menubar application for the SnapTo screenshot tool.

## Features

- **Menubar-only App**: Runs in the menubar without showing a dock icon (LSUIElement = true)
- **Screenshot Capture**:
  - Fullscreen capture
  - Selected area capture
- **Global Hotkeys**:
  - `Cmd+Shift+3`: Fullscreen screenshot
  - `Cmd+Shift+4`: Selected area screenshot
- **Integration**: Uploads screenshots via SnapTo CLI
- **Notifications**: Shows upload progress and results
- **TUI Access**: Quick access to SnapTo Terminal UI

## Project Structure

```
snapto_app/
├── lib/
│   ├── main.dart                          # App entry point
│   └── services/
│       ├── tray_service.dart              # Menubar/tray management
│       ├── screenshot_service.dart        # Screenshot capture
│       ├── upload_service.dart            # SnapTo CLI integration
│       ├── hotkey_service.dart            # Global hotkey registration
│       └── notification_service.dart      # System notifications
├── macos/
│   └── Runner/
│       ├── Info.plist                     # App configuration (LSUIElement)
│       ├── DebugProfile.entitlements      # Dev permissions
│       ├── Release.entitlements           # Release permissions
│       └── Configs/
│           └── AppInfo.xcconfig           # Bundle configuration
└── pubspec.yaml                           # Dependencies

```

## Dependencies

- **tray_manager**: System tray/menubar functionality
- **screen_capturer**: Screenshot capture functionality
- **hotkey_manager**: Global keyboard shortcuts
- **window_manager**: Window control for menubar-only mode
- **local_notifier**: System notifications
- **path_provider**: File system paths
- **path**: Path manipulation

## Setup

### Prerequisites

1. Flutter SDK installed and configured for macOS
2. Xcode installed with command line tools
3. SnapTo CLI built at `../target/release/snapto`

### Installation

1. Get Flutter dependencies:
```bash
cd snapto_app
flutter pub get
```

2. Build the SnapTo CLI (if not already built):
```bash
cd ..
cargo build --release
```

3. Run the app:
```bash
cd snapto_app
flutter run -d macos
```

## Building for Release

```bash
flutter build macos --release
```

The built app will be at: `build/macos/Build/Products/Release/SnapTo.app`

## macOS Permissions

The app requires the following permissions:

1. **Screen Recording**: Required for capturing screenshots
   - Configured in Info.plist with `NSScreenCaptureDescription`
   - User will be prompted on first screenshot attempt

2. **Accessibility**: Required for global hotkeys
   - Configured in Info.plist with `NSAppleEventsUsageDescription`
   - User will be prompted on first hotkey registration

3. **Network**: Required for uploading screenshots
   - Configured in entitlements with `com.apple.security.network.client`

## Configuration

### Menubar-only Mode

The app runs as a menubar-only application (no dock icon) via:
- `LSUIElement = true` in Info.plist
- Window configured to be hidden on startup
- Skip taskbar enabled in WindowOptions

### SnapTo CLI Integration

The app calls the SnapTo CLI at:
```
/Users/avillagran/Desarrollo/ClipClaude/target/release/snapto
```

To change this path, edit `lib/services/upload_service.dart`:
```dart
final String _snaptoPath = '/path/to/snapto';
```

## Menu Items

- **Fullscreen Snap**: Capture entire screen
- **Selected Area Snap**: Capture selected region
- **Open TUI**: Launch SnapTo Terminal UI
- **Settings**: Open app settings (TODO)
- **Quit SnapTo**: Exit the application

## Global Hotkeys

- `Cmd+Shift+3`: Fullscreen screenshot
- `Cmd+Shift+4`: Selected area screenshot

Note: These override macOS default screenshot shortcuts when the app is running.

## Development

### Running in Debug Mode

```bash
flutter run -d macos
```

### Viewing Logs

```bash
flutter logs
```

### Hot Reload

Press `r` in the terminal where `flutter run` is running.

## Troubleshooting

### Screenshots not working

1. Check Screen Recording permission in System Preferences > Security & Privacy > Screen Recording
2. Ensure the app is checked in the list

### Hotkeys not working

1. Check Accessibility permission in System Preferences > Security & Privacy > Accessibility
2. Ensure the app is checked in the list
3. Restart the app after granting permissions

### Upload fails

1. Verify SnapTo CLI is built: `ls -la ../target/release/snapto`
2. Check CLI works manually: `../target/release/snapto --version`
3. Review upload service logs in console

### App doesn't appear in menubar

1. Check that LSUIElement is set to true in Info.plist
2. Look for the tray icon in the menubar (top-right area)
3. Check console logs for tray initialization errors

## TODO

- [ ] Add tray icon asset
- [ ] Implement settings UI
- [ ] Add configurable hotkeys
- [ ] Add upload history
- [ ] Add configuration file support
- [ ] Implement auto-update functionality

## License

Copyright © 2024. All rights reserved.
