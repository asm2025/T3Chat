import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'core/config/app_config.dart';
import 'core/routing/app_router.dart';
import 'core/rust/rust_client.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize Rust client
  await RustClient.initialize();
  
  runApp(
    const ProviderScope(
      child: T3ChatDesktopApp(),
    ),
  );
}

class T3ChatDesktopApp extends StatelessWidget {
  const T3ChatDesktopApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'T3Chat Desktop',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      routerConfig: appRouter,
    );
  }
}

