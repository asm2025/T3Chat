import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'core/config/app_config.dart';
import 'core/logging/logger.dart';
import 'core/routing/app_router.dart';
import 'core/rust/rust_client.dart';
import 'core/window/window_manager_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize logger (all platforms)
  AppLogger().init();
  
  // Initialize window manager (desktop only)
  await WindowManagerService.initialize();
  
  // Initialize Rust client (desktop only)
  if (kIsWeb || Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
    await RustClient.initialize();
  }
  
  runApp(
    const ProviderScope(
      child: T3ChatApp(),
    ),
  );
}

class T3ChatApp extends ConsumerWidget {
  const T3ChatApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(appRouterProvider);

    return MaterialApp.router(
      title: 'T3Chat',
      debugShowCheckedModeBanner: AppConfig.isDebugMode,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      routerConfig: router,
    );
  }
}

