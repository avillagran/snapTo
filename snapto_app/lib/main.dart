import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';
import 'services/tray_service.dart';
import 'services/hotkey_service.dart';
import 'services/screenshot_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize window manager
  await windowManager.ensureInitialized();

  // Configure window to be hidden by default (menubar only app)
  WindowOptions windowOptions = const WindowOptions(
    size: Size(0, 0),
    center: false,
    backgroundColor: Colors.transparent,
    skipTaskbar: true,
    titleBarStyle: TitleBarStyle.hidden,
  );

  await windowManager.waitUntilReadyToShow(windowOptions, () async {
    await windowManager.hide();
  });

  // Initialize services
  final screenshotService = ScreenshotService();
  final trayService = TrayService(screenshotService: screenshotService);
  final hotkeyService = HotkeyService(screenshotService: screenshotService);

  // Initialize tray icon and menu
  await trayService.initialize();

  // Register global hotkeys
  await hotkeyService.initialize();

  runApp(const SnapToApp());
}

class SnapToApp extends StatelessWidget {
  const SnapToApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'SnapTo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      // Empty home widget since this is a menubar-only app
      home: const Scaffold(
        body: Center(
          child: Text('SnapTo is running in the menubar'),
        ),
      ),
      debugShowCheckedModeBanner: false,
    );
  }
}
