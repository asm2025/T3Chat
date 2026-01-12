import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';

class WindowManagerService {
  static const double minWidth = 800.0;
  static const double minHeight = 600.0;
  static const String _keyWindowWidth = 'window_width';
  static const String _keyWindowHeight = 'window_height';
  static const String _keyWindowX = 'window_x';
  static const String _keyWindowY = 'window_y';

  /// Initialize window manager for desktop platforms
  static Future<void> initialize() async {
    // Only initialize on desktop platforms
    if (kIsWeb || !(Platform.isWindows || Platform.isLinux || Platform.isMacOS)) {
      return;
    }

    await windowManager.ensureInitialized();

    // Set window title
    await windowManager.setTitle('T3.Chat');

    // Set minimum window size
    await windowManager.setMinimumSize(const Size(minWidth, minHeight));

    // Load saved window state
    final prefs = await SharedPreferences.getInstance();
    final savedWidth = prefs.getDouble(_keyWindowWidth);
    final savedHeight = prefs.getDouble(_keyWindowHeight);
    final savedX = prefs.getDouble(_keyWindowX);
    final savedY = prefs.getDouble(_keyWindowY);

    Size windowSize = const Size(1280, 720);
    Offset? windowPosition;

    if (savedWidth != null && savedHeight != null) {
      windowSize = Size(savedWidth, savedHeight);
    }

    if (savedX != null && savedY != null) {
      windowPosition = Offset(savedX, savedY);
    }

    // Validate size and position
    windowSize = _validateSize(windowSize);
    if (windowPosition != null) {
      windowPosition = await _validatePosition(windowPosition, windowSize);
    }

    // Configure window options
    WindowOptions windowOptions = WindowOptions(
      size: windowSize,
      center: windowPosition == null,
      backgroundColor: Colors.transparent,
      skipTaskbar: false,
      titleBarStyle: TitleBarStyle.normal,
    );

    await windowManager.waitUntilReadyToShow(windowOptions, () async {
      // Set position after window is ready if we have a saved position
      if (windowPosition != null) {
        await windowManager.setPosition(windowPosition);
      }
      await windowManager.show();
      await windowManager.focus();
    });

    // Listen to window events to save position and size
    windowManager.addListener(_WindowListener());
  }

  /// Validate that the position is within visible desktop bounds
  static Future<Offset> _validatePosition(Offset position, Size windowSize) async {
    try {
      // Get current window size to infer screen bounds
      // We'll use a reasonable default screen size and validate against it
      // In a multi-monitor setup, Windows will handle positioning
      const defaultScreenWidth = 1920.0;
      const defaultScreenHeight = 1080.0;
      
      // Ensure window fits within reasonable screen bounds
      final maxX = defaultScreenWidth - windowSize.width;
      final maxY = defaultScreenHeight - windowSize.height;
      
      // Clamp position to ensure window is at least partially visible
      // Allow some negative offset for multi-monitor setups
      final validatedX = position.dx.clamp(-windowSize.width + 100, maxX > 0 ? maxX : defaultScreenWidth);
      final validatedY = position.dy.clamp(-windowSize.height + 100, maxY > 0 ? maxY : defaultScreenHeight);
      
      return Offset(validatedX, validatedY);
    } catch (e) {
      // If validation fails, return the original position
      return position;
    }
  }

  /// Validate that the size meets minimum requirements
  static Size _validateSize(Size size) {
    return Size(
      size.width < minWidth ? minWidth : size.width,
      size.height < minHeight ? minHeight : size.height,
    );
  }

  /// Save window state to preferences
  static Future<void> _saveWindowState() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final position = await windowManager.getPosition();
      final size = await windowManager.getSize();

      await prefs.setDouble(_keyWindowWidth, size.width);
      await prefs.setDouble(_keyWindowHeight, size.height);
      await prefs.setDouble(_keyWindowX, position.dx);
      await prefs.setDouble(_keyWindowY, position.dy);
    } catch (e) {
      // Ignore errors during save
    }
  }
}

/// Window listener to save position and size on changes
class _WindowListener extends WindowListener {
  @override
  void onWindowMoved() async {
    await WindowManagerService._saveWindowState();
  }

  @override
  void onWindowResized() async {
    await WindowManagerService._saveWindowState();
  }

  @override
  void onWindowClose() async {
    await WindowManagerService._saveWindowState();
    // Allow default close behavior
  }
}
