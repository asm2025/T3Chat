import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'core/config/app_config.dart';
import 'core/logging/logger.dart';
import 'core/router/app_router.dart';

void main() {
  // Initialize logger
  AppLogger().init();

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

