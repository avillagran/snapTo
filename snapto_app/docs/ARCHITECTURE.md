# SnapTo macOS App - Architecture

This document describes the architecture and design of the SnapTo macOS menubar application.

## Overview

SnapTo is a macOS menubar application built with Flutter that provides quick screenshot capture and upload functionality. It integrates with the SnapTo CLI tool for uploading screenshots and runs as a menubar-only app without showing a dock icon.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        macOS System                          │
│  ┌────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│  │  Menubar   │  │   Screen     │  │  Global Hotkeys   │   │
│  │   Tray     │  │  Recording   │  │  (Accessibility)  │   │
│  └────────────┘  └──────────────┘  └───────────────────┘   │
└───────┬──────────────────┬──────────────────┬───────────────┘
        │                  │                  │
        │                  │                  │
┌───────▼──────────────────▼──────────────────▼───────────────┐
│                   Flutter Application                        │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                      main.dart                         │ │
│  │  - App initialization                                  │ │
│  │  - Window configuration (hidden, no dock)              │ │
│  │  - Service initialization                              │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                    Services Layer                      │ │
│  │                                                        │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │ │
│  │  │TrayService   │  │Screenshot    │  │Hotkey      │  │ │
│  │  │              │  │Service       │  │Service     │  │ │
│  │  │-Menubar mgmt │  │              │  │            │  │ │
│  │  │-Menu items   │  │-Fullscreen   │  │-Cmd+Shift+3│  │ │
│  │  │-Click handler│  │-Selection    │  │-Cmd+Shift+4│  │ │
│  │  └──────────────┘  └──────────────┘  └────────────┘  │ │
│  │                                                        │ │
│  │  ┌──────────────┐  ┌──────────────┐                  │ │
│  │  │Upload        │  │Notification  │                  │ │
│  │  │Service       │  │Service       │                  │ │
│  │  │              │  │              │                  │ │
│  │  │-CLI exec     │  │-Notifications│                  │ │
│  │  │-URL extract  │  │-Upload status│                  │ │
│  │  │-Clipboard    │  │              │                  │ │
│  │  └──────────────┘  └──────────────┘                  │ │
│  └────────────────────────────────────────────────────────┘ │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Process.run()
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                    SnapTo CLI (Rust)                         │
│  - Screenshot upload                                         │
│  - URL generation                                            │
│  - TUI interface                                             │
└──────────────────────────────────────────────────────────────┘
```

## Components

### 1. Main Application (main.dart)

**Responsibility**: Application lifecycle and initialization

**Key Features**:
- Initialize Flutter bindings
- Configure window manager for menubar-only mode
- Initialize all services
- Set up hidden window with zero size
- Skip taskbar and hide on startup

**Important Settings**:
```dart
WindowOptions(
  size: Size(0, 0),           // Hidden window
  skipTaskbar: true,           // No dock icon
  titleBarStyle: TitleBarStyle.hidden,
)
```

### 2. Tray Service (tray_service.dart)

**Responsibility**: Menubar icon and menu management

**Key Features**:
- Create and manage menubar icon
- Define menu structure
- Handle menu item clicks
- Show/hide app window for settings

**Menu Structure**:
```
SnapTo
├── Fullscreen Snap (Cmd+Shift+3)
├── Selected Area Snap (Cmd+Shift+4)
├── ─────────────────
├── Open TUI
├── Settings
├── ─────────────────
└── Quit SnapTo
```

**Dependencies**:
- `tray_manager`: System tray integration
- `window_manager`: Window control for settings

### 3. Screenshot Service (screenshot_service.dart)

**Responsibility**: Capture screenshots

**Key Features**:
- Fullscreen capture
- Region selection capture
- Temporary file management
- Integration with upload service
- Error handling

**Flow**:
1. Capture screenshot to temp file
2. Show "capturing" notification
3. Call upload service
4. Show result notification
5. Clean up temp file

**Dependencies**:
- `screen_capturer`: Screenshot capture
- `path_provider`: Temp directory access

### 4. Upload Service (upload_service.dart)

**Responsibility**: Upload screenshots via SnapTo CLI

**Key Features**:
- Execute SnapTo CLI
- Parse CLI output for URL
- Copy URL to clipboard
- Validate CLI availability

**CLI Integration**:
```dart
Process.run(_snaptoPath, ['upload', imagePath])
```

**URL Extraction**:
- Parse stdout for HTTP/HTTPS URLs
- Extract last URL from output
- Copy to clipboard automatically

**Dependencies**:
- Dart's `dart:io` for process execution
- `flutter/services` for clipboard

### 5. Hotkey Service (hotkey_service.dart)

**Responsibility**: Register and handle global hotkeys

**Key Features**:
- Register Cmd+Shift+3 (fullscreen)
- Register Cmd+Shift+4 (selection)
- System-wide hotkey scope
- Cleanup on app exit

**Hotkey Definitions**:
```dart
HotKey(
  key: PhysicalKeyboardKey.digit3,
  modifiers: [HotKeyModifier.meta, HotKeyModifier.shift],
  scope: HotKeyScope.system,
)
```

**Dependencies**:
- `hotkey_manager`: Global hotkey registration

### 6. Notification Service (notification_service.dart)

**Responsibility**: Show system notifications

**Key Features**:
- Initialize notification system
- Show upload progress
- Show success/error messages
- Include URLs in notifications

**Notification Types**:
- Screenshot captured
- Uploading...
- Upload successful (with URL)
- Upload failed (with error)

**Dependencies**:
- `local_notifier`: macOS notifications

## Data Flow

### Screenshot Capture Flow

```
User Action (Menu/Hotkey)
    │
    ▼
Screenshot Service
    │
    ├─ Capture to temp file
    │  (screen_capturer)
    │
    ├─ Show "Capturing" notification
    │  (notification_service)
    │
    ├─ Upload screenshot
    │  (upload_service)
    │     │
    │     ├─ Execute CLI: snapto upload <file>
    │     ├─ Parse output for URL
    │     └─ Copy URL to clipboard
    │
    ├─ Show result notification
    │  (notification_service)
    │
    └─ Clean up temp file
```

### Menu Click Flow

```
User Clicks Menubar Icon
    │
    ▼
Tray Service (onTrayIconMouseDown)
    │
    ├─ Show context menu
    │
    └─ Wait for selection
        │
        ▼
    onTrayMenuItemClick
        │
        ├─ Fullscreen Snap → Screenshot Service
        ├─ Selected Area → Screenshot Service
        ├─ Open TUI → Launch Terminal
        ├─ Settings → Show window
        └─ Quit → exit(0)
```

### Hotkey Flow

```
User Presses Cmd+Shift+3/4
    │
    ▼
Hotkey Service (keyDownHandler)
    │
    └─ Screenshot Service
        │
        └─ (same as screenshot flow above)
```

## macOS Integration

### LSUIElement Configuration

**File**: `macos/Runner/Info.plist`

```xml
<key>LSUIElement</key>
<true/>
```

This tells macOS to run the app without a dock icon.

### Activation Policy

**File**: `macos/Runner/AppDelegate.swift`

```swift
NSApp.setActivationPolicy(.accessory)
```

Ensures the app runs as an accessory (menubar only).

### Permissions

**Screen Recording**:
- Required for screenshot capture
- Prompts user on first use
- Configured via `NSScreenCaptureDescription` in Info.plist

**Accessibility**:
- Required for global hotkeys
- Prompts user on first use
- Configured via `NSAppleEventsUsageDescription` in Info.plist

### Entitlements

**Development** (`DebugProfile.entitlements`):
- App Sandbox enabled
- Network client access
- File read/write access
- Temporary file access
- Apple Events for Terminal
- Disabled library validation (for development)

**Release** (`Release.entitlements`):
- Same as debug but without development exceptions
- More restricted for security

## Dependencies

### Flutter Packages

| Package | Version | Purpose |
|---------|---------|---------|
| tray_manager | ^0.2.0 | System tray/menubar |
| screen_capturer | ^0.2.0 | Screenshot capture |
| hotkey_manager | ^0.2.0 | Global hotkeys |
| window_manager | ^0.3.0 | Window control |
| local_notifier | ^0.1.5 | Notifications |
| path_provider | ^2.1.0 | System paths |
| path | ^1.8.3 | Path manipulation |

### External Dependencies

- **SnapTo CLI** (`../target/release/snapto`): Rust binary for upload
- **Terminal.app**: For launching TUI
- **macOS System APIs**: For permissions and integrations

## Security Considerations

### Sandboxing

The app runs in macOS App Sandbox with limited permissions:
- Network access (for uploads)
- Temporary file access (for screenshots)
- User-selected file access (for manual file selection)

### Permissions

All required permissions are explicitly declared:
- Screen Recording: For capturing screenshots
- Accessibility: For global hotkeys
- Apple Events: For launching Terminal

### Data Handling

- Screenshots are stored temporarily
- Files are deleted after upload
- URLs are copied to clipboard
- No persistent storage of images

## Performance Considerations

### Startup

- Window is hidden immediately
- Services initialize asynchronously
- Minimal UI (menubar only)

### Screenshot Capture

- Uses native macOS screen capture
- Temporary file I/O
- Asynchronous upload

### Memory

- Cleanup temp files after upload
- No image caching
- Minimal UI state

## Error Handling

### Screenshot Errors

- Permission denied → Notify user to grant access
- Capture failed → Show error notification
- No selection → Silent cancel (expected behavior)

### Upload Errors

- CLI not found → Check binary path
- Upload failed → Show error with details
- Network error → Retry or manual retry via menu

### Hotkey Errors

- Registration failed → Log error, continue without hotkeys
- Permission denied → Notify user to grant access

## Future Enhancements

### Settings UI

- Configure upload destination
- Customize hotkeys
- Set screenshot format
- Configure notifications

### Upload History

- Track uploaded screenshots
- View URL history
- Re-copy URLs
- Delete from server

### Advanced Features

- OCR text extraction
- Annotation tools
- Multiple upload destinations
- Automatic clipboard monitoring

## Development Workflow

### Running

```bash
flutter run -d macos
```

### Building

```bash
flutter build macos --release
```

### Testing

```bash
flutter test
```

### Debugging

- Use `flutter logs` for runtime logs
- Use `debugPrint()` in code
- Check Console.app for native logs

## Deployment

### Distribution

1. Build release binary
2. Code sign the app
3. Create DMG installer
4. Distribute via website or App Store

### Auto-Updates

- Consider using Sparkle framework
- Check for updates on launch
- Download and install updates

## Maintenance

### Updating Dependencies

```bash
flutter pub upgrade
```

### Flutter SDK Updates

```bash
flutter upgrade
```

### macOS Version Compatibility

- Minimum: macOS 10.14 (Mojave)
- Recommended: macOS 11.0+ (Big Sur)
- Test on multiple OS versions

---

Last Updated: 2024-12-25
