import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:hotkey_manager/hotkey_manager.dart';
import 'screenshot_service.dart';

class HotkeyService {
  final ScreenshotService screenshotService;

  HotKey? _fullscreenHotkey;
  HotKey? _selectedAreaHotkey;

  HotkeyService({required this.screenshotService});

  Future<void> initialize() async {
    try {
      // Register Cmd+Shift+3 for fullscreen capture
      await _registerFullscreenHotkey();

      // Register Cmd+Shift+4 for selected area capture
      await _registerSelectedAreaHotkey();

      debugPrint('Global hotkeys registered successfully');
    } catch (e) {
      debugPrint('Error registering hotkeys: $e');
    }
  }

  Future<void> _registerFullscreenHotkey() async {
    _fullscreenHotkey = HotKey(
      key: PhysicalKeyboardKey.digit3,
      modifiers: [HotKeyModifier.meta, HotKeyModifier.shift],
      scope: HotKeyScope.system,
    );

    await hotKeyManager.register(
      _fullscreenHotkey!,
      keyDownHandler: (hotKey) {
        debugPrint('Fullscreen hotkey pressed: Cmd+Shift+3');
        screenshotService.captureFullscreen();
      },
    );
  }

  Future<void> _registerSelectedAreaHotkey() async {
    _selectedAreaHotkey = HotKey(
      key: PhysicalKeyboardKey.digit4,
      modifiers: [HotKeyModifier.meta, HotKeyModifier.shift],
      scope: HotKeyScope.system,
    );

    await hotKeyManager.register(
      _selectedAreaHotkey!,
      keyDownHandler: (hotKey) {
        debugPrint('Selected area hotkey pressed: Cmd+Shift+4');
        screenshotService.captureSelectedArea();
      },
    );
  }

  Future<void> dispose() async {
    try {
      if (_fullscreenHotkey != null) {
        await hotKeyManager.unregister(_fullscreenHotkey!);
      }
      if (_selectedAreaHotkey != null) {
        await hotKeyManager.unregister(_selectedAreaHotkey!);
      }
      debugPrint('Hotkeys unregistered');
    } catch (e) {
      debugPrint('Error unregistering hotkeys: $e');
    }
  }
}
