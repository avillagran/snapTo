import 'dart:io';
import 'package:flutter/material.dart';
import 'package:screen_capturer/screen_capturer.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import 'upload_service.dart';
import 'notification_service.dart';

class ScreenshotService {
  final ScreenCapturer _screenCapturer = ScreenCapturer.instance;
  final UploadService _uploadService = UploadService();
  final NotificationService _notificationService = NotificationService();

  ScreenshotService() {
    _notificationService.initialize();
  }

  /// Capture fullscreen screenshot
  Future<void> captureFullscreen() async {
    try {
      debugPrint('Starting fullscreen capture...');

      // Get temporary directory for saving screenshot
      final tempDir = await getTemporaryDirectory();
      final timestamp = DateTime.now().millisecondsSinceEpoch;
      final fileName = 'snapto_fullscreen_$timestamp.png';
      final filePath = path.join(tempDir.path, fileName);

      debugPrint('Saving screenshot to: $filePath');

      // Capture the screenshot
      final capturedData = await _screenCapturer.capture(
        mode: CaptureMode.screen,
        imagePath: filePath,
        copyToClipboard: false,
      );

      if (capturedData != null && capturedData.imagePath != null) {
        debugPrint('Screenshot captured successfully: ${capturedData.imagePath}');

        // Show notification
        await _notificationService.showNotification(
          title: 'Screenshot Captured',
          body: 'Uploading to SnapTo...',
        );

        // Upload using SnapTo CLI
        final url = await _uploadService.uploadScreenshot(capturedData.imagePath!);

        if (url != null) {
          debugPrint('Screenshot uploaded: $url');
          await _notificationService.showNotification(
            title: 'Upload Successful',
            body: 'URL copied to clipboard',
          );
        } else {
          debugPrint('Upload failed');
          await _notificationService.showNotification(
            title: 'Upload Failed',
            body: 'Could not upload screenshot',
          );
        }

        // Clean up temp file
        await _cleanupFile(capturedData.imagePath!);
      } else {
        debugPrint('Screenshot capture returned null');
        await _notificationService.showNotification(
          title: 'Capture Failed',
          body: 'Could not capture screenshot',
        );
      }
    } catch (e) {
      debugPrint('Error capturing fullscreen: $e');
      await _notificationService.showNotification(
        title: 'Error',
        body: 'Screenshot capture failed: $e',
      );
    }
  }

  /// Capture selected area screenshot
  Future<void> captureSelectedArea() async {
    try {
      debugPrint('Starting selected area capture...');

      // Get temporary directory for saving screenshot
      final tempDir = await getTemporaryDirectory();
      final timestamp = DateTime.now().millisecondsSinceEpoch;
      final fileName = 'snapto_selection_$timestamp.png';
      final filePath = path.join(tempDir.path, fileName);

      debugPrint('Saving screenshot to: $filePath');

      // Capture the screenshot with region selection
      final capturedData = await _screenCapturer.capture(
        mode: CaptureMode.region,
        imagePath: filePath,
        copyToClipboard: false,
      );

      if (capturedData != null && capturedData.imagePath != null) {
        debugPrint('Screenshot captured successfully: ${capturedData.imagePath}');

        // Show notification
        await _notificationService.showNotification(
          title: 'Screenshot Captured',
          body: 'Uploading to SnapTo...',
        );

        // Upload using SnapTo CLI
        final url = await _uploadService.uploadScreenshot(capturedData.imagePath!);

        if (url != null) {
          debugPrint('Screenshot uploaded: $url');
          await _notificationService.showNotification(
            title: 'Upload Successful',
            body: 'URL copied to clipboard',
          );
        } else {
          debugPrint('Upload failed');
          await _notificationService.showNotification(
            title: 'Upload Failed',
            body: 'Could not upload screenshot',
          );
        }

        // Clean up temp file
        await _cleanupFile(capturedData.imagePath!);
      } else {
        debugPrint('Screenshot capture cancelled or returned null');
        // User likely cancelled the selection, don't show error
      }
    } catch (e) {
      debugPrint('Error capturing selected area: $e');
      await _notificationService.showNotification(
        title: 'Error',
        body: 'Screenshot capture failed: $e',
      );
    }
  }

  /// Clean up temporary screenshot file
  Future<void> _cleanupFile(String filePath) async {
    try {
      final file = File(filePath);
      if (await file.exists()) {
        await file.delete();
        debugPrint('Cleaned up temp file: $filePath');
      }
    } catch (e) {
      debugPrint('Error cleaning up file: $e');
    }
  }
}
