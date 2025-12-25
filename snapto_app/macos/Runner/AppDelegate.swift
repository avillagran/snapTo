import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    // Keep the app running even when the window is closed
    // This is important for menubar-only apps
    return false
  }

  override func applicationDidFinishLaunching(_ notification: Notification) {
    // Hide the app from dock if LSUIElement is not working
    NSApp.setActivationPolicy(.accessory)
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}
