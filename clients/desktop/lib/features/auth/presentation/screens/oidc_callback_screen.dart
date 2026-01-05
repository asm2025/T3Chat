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
    if (widget.token != null) {
      // Token is extracted from URL, now we need to store it
      // The Rust client should handle this automatically when we call getCurrentUser
      // For now, just navigate to app
      WidgetsBinding.instance.addPostFrameCallback((_) {
        context.go('/app');
      });
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

