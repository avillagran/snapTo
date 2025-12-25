import 'package:flutter/material.dart';
import 'package:local_notifier/local_notifier.dart';

class NotificationService {
  bool _isInitialized = false;

  Future<void> initialize() async {
    if (_isInitialized) return;

    try {
      await localNotifier.setup(
        appName: 'SnapTo',
        shortcutPolicy: ShortcutPolicy.requireCreate,
      );
      _isInitialized = true;
      debugPrint('Notification service initialized');
    } catch (e) {
      debugPrint('Error initializing notification service: $e');
    }
  }

  Future<void> showNotification({
    required String title,
    required String body,
    String? subtitle,
  }) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      final notification = LocalNotification(
        title: title,
        body: body,
        subtitle: subtitle,
      );

      await notification.show();
      debugPrint('Notification shown: $title - $body');
    } catch (e) {
      debugPrint('Error showing notification: $e');
    }
  }

  Future<void> showUploadProgress({
    required String fileName,
  }) async {
    await showNotification(
      title: 'Uploading Screenshot',
      body: 'Uploading $fileName...',
    );
  }

  Future<void> showUploadSuccess({
    required String url,
  }) async {
    await showNotification(
      title: 'Upload Successful',
      body: 'URL copied to clipboard',
      subtitle: url,
    );
  }

  Future<void> showUploadError({
    required String error,
  }) async {
    await showNotification(
      title: 'Upload Failed',
      body: error,
    );
  }
}
