import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../providers/auth_provider.dart';

class OidcCallbackScreen extends ConsumerStatefulWidget {
  final String? token;

  const OidcCallbackScreen({super.key, this.token});

  @override
  ConsumerState<OidcCallbackScreen> createState() => _OidcCallbackScreenState();
}

class _OidcCallbackScreenState extends ConsumerState<OidcCallbackScreen> {
  @override
  void initState() {
    super.initState();
    _handleCallback();
  }

  Future<void> _handleCallback() async {
    if (widget.token != null) {
      // Store token and refresh user
      // In a real implementation, you'd extract the token from the URL
      // For now, we'll just navigate to the app
      await Future.delayed(const Duration(seconds: 1));
      if (mounted) {
        ref.read(authStateProvider.notifier).refresh();
        context.go('/app');
      }
    } else {
      // No token, go back to login
      if (mounted) {
        context.go('/login');
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(
        child: CircularProgressIndicator(),
      ),
    );
  }
}

