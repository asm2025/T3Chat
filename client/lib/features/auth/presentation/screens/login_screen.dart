import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../providers/auth_provider.dart';
import '../widgets/login_form.dart';

class LoginScreen extends ConsumerWidget {
  const LoginScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authStateProvider);
    final authConfigAsync = ref.watch(authConfigProvider);

    // Navigate to app if authenticated
    ref.listen<AsyncValue>(authStateProvider, (previous, next) {
      if (next.hasValue && next.value != null) {
        context.go('/app');
      }
    });

    return Scaffold(
      body: Center(
        child: SizedBox(
          width: 400,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Text(
                'T3Chat',
                style: TextStyle(fontSize: 32, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 32),
              authConfigAsync.when(
                data: (config) => LoginForm(oidcEnabled: config.oidcEnabled),
                loading: () => const CircularProgressIndicator(),
                error: (e, _) => LoginForm(oidcEnabled: false),
              ),
              authState.when(
                data: (_) => const SizedBox.shrink(),
                loading: () => const Padding(
                  padding: EdgeInsets.only(top: 16),
                  child: CircularProgressIndicator(),
                ),
                error: (e, _) => Padding(
                  padding: const EdgeInsets.only(top: 16),
                  child: Text(
                    e.toString(),
                    style: const TextStyle(color: Colors.red),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

