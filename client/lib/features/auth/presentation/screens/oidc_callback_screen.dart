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
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final token = widget.token;
      if (token == null || token.isEmpty) {
        context.go('/login');
        return;
      }

      await ref.read(authStateProvider.notifier).loginWithToken(token);

      // GoRouter redirect will handle navigation once authState becomes non-null.
      if (mounted) {
        final authState = ref.read(authStateProvider);
        if (authState.hasError) {
          context.go('/login');
        }
      }
    });
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

