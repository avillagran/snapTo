import 'dart:io';
import 'package:flutter/material.dart';
import 'package:tray_manager/tray_manager.dart';
import 'package:window_manager/window_manager.dart';
import 'screenshot_service.dart';
import 'upload_service.dart';

class TrayService extends TrayListener {
  final ScreenshotService screenshotService;
  final UploadService uploadService = UploadService();

  TrayService({required this.screenshotService});

  Future<void> initialize() async {
    // Initialize upload service to find bundled binaries
    await uploadService.initialize();

    // Add this service as listener
    trayManager.addListener(this);

    // Set tray icon
    await _setTrayIcon();

    // Set tray menu
    await _setTrayMenu();

    // Set tooltip
    await trayManager.setToolTip('SnapTo Screenshot Tool');
  }

  Future<void> _setTrayIcon() async {
    // On macOS, we can use a system symbol or skip if no icon
    // The tray will show with default appearance
    if (Platform.isMacOS) {
      // Use NSImage system symbol - camera icon
      try {
        await trayManager.setIcon(
          'assets/tray_icon.png',
          isTemplate: true,
        );
      } catch (e) {
        debugPrint('Failed to load tray icon: $e');
        // On macOS, tray still works without icon (shows app name)
      }
    }
  }

  Future<void> _setTrayMenu() async {
    Menu menu = Menu(
      items: [
        MenuItem(
          key: 'fullscreen_snap',
          label: 'Fullscreen Snap',
          toolTip: 'Capture entire screen (Cmd+Shift+3)',
        ),
        MenuItem(
          key: 'selected_area_snap',
          label: 'Selected Area Snap',
          toolTip: 'Capture selected area (Cmd+Shift+4)',
        ),
        MenuItem.separator(),
        MenuItem(
          key: 'open_tui',
          label: 'Open TUI',
          toolTip: 'Open SnapTo Terminal UI',
        ),
        MenuItem(
          key: 'settings',
          label: 'Settings',
          toolTip: 'Configure SnapTo',
        ),
        MenuItem.separator(),
        MenuItem(
          key: 'quit',
          label: 'Quit SnapTo',
        ),
      ],
    );

    await trayManager.setContextMenu(menu);
  }

  @override
  void onTrayIconMouseDown() {
    // Show menu on click
    trayManager.popUpContextMenu();
  }

  @override
  void onTrayIconRightMouseDown() {
    // Show menu on right click
    trayManager.popUpContextMenu();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) {
    switch (menuItem.key) {
      case 'fullscreen_snap':
        _handleFullscreenSnap();
        break;
      case 'selected_area_snap':
        _handleSelectedAreaSnap();
        break;
      case 'open_tui':
        _handleOpenTUI();
        break;
      case 'settings':
        _handleSettings();
        break;
      case 'quit':
        _handleQuit();
        break;
    }
  }

  void _handleFullscreenSnap() async {
    debugPrint('Fullscreen snap triggered from tray menu');
    await screenshotService.captureFullscreen();
  }

  void _handleSelectedAreaSnap() async {
    debugPrint('Selected area snap triggered from tray menu');
    await screenshotService.captureSelectedArea();
  }

  void _handleOpenTUI() async {
    debugPrint('Opening TUI...');

    final success = await uploadService.openTui();
    if (!success) {
      debugPrint('Failed to open TUI - binary not found or error occurred');
      // TODO: Show notification to user
    }
  }

  void _handleSettings() async {
    debugPrint('Opening settings...');

    // Open the settings config file or TUI
    final success = await uploadService.openSettings();
    if (!success) {
      debugPrint('Failed to open settings');
      // Fallback: Show the Flutter app window
      await windowManager.show();
      await windowManager.focus();
      await windowManager.setSize(const Size(600, 400));
      await windowManager.center();
    }
  }

  void _handleQuit() {
    debugPrint('Quitting SnapTo...');
    exit(0);
  }

  void dispose() {
    trayManager.removeListener(this);
  }
}
