import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class UploadService {
  static final UploadService _instance = UploadService._internal();
  factory UploadService() => _instance;
  UploadService._internal();

  String? _snaptoBinaryPath;
  String? _snaptoTuiBinaryPath;
  bool _initialized = false;

  /// Initialize the service and locate bundled binaries
  Future<void> initialize() async {
    if (_initialized) return;

    _snaptoBinaryPath = await _findBinary('snapto');
    _snaptoTuiBinaryPath = await _findBinary('snapto-tui');

    debugPrint('SnapTo binary: $_snaptoBinaryPath');
    debugPrint('SnapTo TUI binary: $_snaptoTuiBinaryPath');

    _initialized = true;
  }

  /// Find binary in app bundle or development location (platform-aware)
  Future<String?> _findBinary(String name) async {
    final execPath = Platform.resolvedExecutable;
    final binaryName = _getPlatformBinaryName(name);

    if (Platform.isMacOS) {
      // macOS: Check app bundle first
      // Executable: /path/to/App.app/Contents/MacOS/<executable>
      // Resources:  /path/to/App.app/Contents/Resources/
      final macosDir = File(execPath).parent.path; // .../Contents/MacOS
      final contentsDir = Directory(macosDir).parent.path; // .../Contents
      final resourcesPath = '$contentsDir/Resources/$binaryName';
      debugPrint('Checking macOS bundle path: $resourcesPath');
      if (await File(resourcesPath).exists()) {
        return resourcesPath;
      }
    } else if (Platform.isWindows) {
      // Windows: Check next to executable
      final exeDir = File(execPath).parent.path;
      final bundlePath = '$exeDir/$binaryName';
      debugPrint('Checking Windows path: $bundlePath');
      if (await File(bundlePath).exists()) {
        return bundlePath;
      }
    } else if (Platform.isLinux) {
      // Linux: Check next to executable or in /usr/local/bin
      final exeDir = File(execPath).parent.path;
      final paths = [
        '$exeDir/$binaryName',
        '/usr/local/bin/$binaryName',
      ];
      for (final path in paths) {
        if (await File(path).exists()) {
          return path;
        }
      }
    }

    // Development paths - relative to project root
    final devPaths = _getDevPaths(binaryName);

    for (final path in devPaths) {
      final file = File(path);
      if (await file.exists()) {
        debugPrint('Found binary at dev path: $path');
        return file.absolute.path;
      }
    }

    debugPrint('Binary not found: $name');
    return null;
  }

  /// Get platform-specific binary name
  String _getPlatformBinaryName(String name) {
    if (Platform.isWindows) {
      return '$name.exe';
    }
    return name;
  }

  /// Get development paths based on platform
  List<String> _getDevPaths(String binaryName) {
    final home = Platform.environment['HOME'] ?? Platform.environment['USERPROFILE'] ?? '';

    if (Platform.isWindows) {
      return [
        '$home/Desarrollo/ClipClaude/target/release/$binaryName',
        '$home/Desarrollo/ClipClaude/target/debug/$binaryName',
        // Current directory
        'target/release/$binaryName',
        'target/debug/$binaryName',
      ];
    } else {
      // macOS and Linux
      return [
        '/Users/avillagran/Desarrollo/ClipClaude/target/release/$binaryName',
        '/Users/avillagran/Desarrollo/ClipClaude/target/debug/$binaryName',
        '$home/Desarrollo/ClipClaude/target/release/$binaryName',
        '$home/Desarrollo/ClipClaude/target/debug/$binaryName',
      ];
    }
  }

  /// Get the path to the snapto CLI binary
  String? get snaptoBinaryPath => _snaptoBinaryPath;

  /// Get the path to the snapto-tui binary
  String? get snaptoTuiBinaryPath => _snaptoTuiBinaryPath;

  /// Check if SnapTo CLI is available
  Future<bool> isSnapToAvailable() async {
    await initialize();
    return _snaptoBinaryPath != null;
  }

  /// Upload a screenshot file using the snapto CLI
  Future<String?> uploadScreenshot(String imagePath) async {
    await initialize();

    if (_snaptoBinaryPath == null) {
      debugPrint('SnapTo binary not found');
      return null;
    }

    try {
      debugPrint('Uploading screenshot: $imagePath');
      debugPrint('Using binary: $_snaptoBinaryPath');

      final result = await Process.run(
        _snaptoBinaryPath!,
        ['upload', imagePath],
        environment: Platform.environment,
      );

      if (result.exitCode == 0) {
        final output = result.stdout.toString().trim();
        debugPrint('SnapTo upload output: $output');

        // Extract URL from output
        final url = _extractUrl(output);

        if (url != null && url.isNotEmpty) {
          // Copy URL to clipboard
          await Clipboard.setData(ClipboardData(text: url));
          debugPrint('URL copied to clipboard: $url');
          return url;
        } else {
          debugPrint('No URL found in output');
          return null;
        }
      } else {
        debugPrint('SnapTo upload failed with exit code: ${result.exitCode}');
        debugPrint('Stdout: ${result.stdout}');
        debugPrint('Stderr: ${result.stderr}');
        return null;
      }
    } catch (e) {
      debugPrint('Error uploading screenshot: $e');
      return null;
    }
  }

  /// Upload from clipboard (using snapto's clipboard functionality)
  Future<String?> uploadFromClipboard({String? destination}) async {
    await initialize();

    if (_snaptoBinaryPath == null) {
      debugPrint('SnapTo binary not found');
      return null;
    }

    try {
      final args = ['upload'];
      if (destination != null) {
        args.addAll(['-d', destination]);
      }

      debugPrint('Uploading from clipboard with: $_snaptoBinaryPath ${args.join(' ')}');

      final result = await Process.run(
        _snaptoBinaryPath!,
        args,
        environment: Platform.environment,
      );

      if (result.exitCode == 0) {
        final output = result.stdout.toString();
        final url = _extractUrl(output);

        if (url != null) {
          await Clipboard.setData(ClipboardData(text: url));
          debugPrint('URL copied to clipboard: $url');
        }

        return url;
      } else {
        debugPrint('Upload from clipboard failed: ${result.stderr}');
        return null;
      }
    } catch (e) {
      debugPrint('Error uploading from clipboard: $e');
      return null;
    }
  }

  /// Open the TUI in a new Terminal window (platform-aware)
  Future<bool> openTui() async {
    await initialize();

    if (_snaptoTuiBinaryPath == null) {
      debugPrint('SnapTo TUI binary not found');
      return false;
    }

    try {
      if (Platform.isMacOS) {
        // macOS: Use AppleScript to open Terminal
        final escapedPath = _snaptoTuiBinaryPath!.replaceAll('"', '\\"');
        final script = '''
tell application "Terminal"
  activate
  do script "$escapedPath"
end tell
''';
        debugPrint('Opening TUI with AppleScript');
        final result = await Process.run('osascript', ['-e', script]);
        return result.exitCode == 0;

      } else if (Platform.isWindows) {
        // Windows: Open in new cmd window
        final result = await Process.run(
          'cmd',
          ['/c', 'start', 'cmd', '/k', _snaptoTuiBinaryPath!],
        );
        return result.exitCode == 0;

      } else if (Platform.isLinux) {
        // Linux: Try common terminal emulators
        final terminals = [
          ['gnome-terminal', '--', _snaptoTuiBinaryPath!],
          ['konsole', '-e', _snaptoTuiBinaryPath!],
          ['xfce4-terminal', '-e', _snaptoTuiBinaryPath!],
          ['xterm', '-e', _snaptoTuiBinaryPath!],
        ];

        for (final termCmd in terminals) {
          try {
            final result = await Process.run(termCmd[0], termCmd.sublist(1));
            if (result.exitCode == 0) {
              return true;
            }
          } catch (_) {
            // Terminal not found, try next
          }
        }
        return false;
      }

      return false;
    } catch (e) {
      debugPrint('Error opening TUI: $e');
      return false;
    }
  }

  /// Open settings - opens the config file or TUI
  Future<bool> openSettings() async {
    final configPath = '${Platform.environment['HOME']}/.config/snapto/config.toml';
    final configFile = File(configPath);

    if (await configFile.exists()) {
      try {
        // Open config file in default text editor
        final result = await Process.run('open', ['-e', configPath]);
        return result.exitCode == 0;
      } catch (e) {
        debugPrint('Failed to open settings file: $e');
        return openTui(); // Fallback to TUI
      }
    } else {
      // Config doesn't exist, open TUI to create it
      return openTui();
    }
  }

  /// Get SnapTo version
  Future<String?> getSnapToVersion() async {
    await initialize();

    if (_snaptoBinaryPath == null) {
      return null;
    }

    try {
      final result = await Process.run(
        _snaptoBinaryPath!,
        ['--version'],
      );

      if (result.exitCode == 0) {
        return result.stdout.toString().trim();
      }
      return null;
    } catch (e) {
      debugPrint('Error getting SnapTo version: $e');
      return null;
    }
  }

  /// Extract URL from snapto output
  String? _extractUrl(String output) {
    // Look for URLs (http/https)
    final urlPattern = RegExp(r'https?://[^\s\]]+');
    final match = urlPattern.firstMatch(output);
    if (match != null) {
      return match.group(0);
    }

    // Look for specific output patterns
    final lines = output.split('\n');
    for (final line in lines) {
      // Check for "URL: ..." or "Uploaded to: ..." patterns
      if (line.contains('URL:') || line.contains('Uploaded to:')) {
        final parts = line.split(RegExp(r':\s*'));
        if (parts.length > 1) {
          return parts.last.trim();
        }
      }
    }

    return null;
  }
}
