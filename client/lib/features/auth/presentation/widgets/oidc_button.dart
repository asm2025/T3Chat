import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';
import '../../../../core/config/app_config.dart';

class OidcButton extends StatelessWidget {
  const OidcButton({super.key});

  Future<void> _handleOidcLogin() async {
    final oidcUrl = Uri.parse('${AppConfig.apiBaseUrl}/api/auth/login');
    if (await canLaunchUrl(oidcUrl)) {
      await launchUrl(oidcUrl, mode: LaunchMode.externalApplication);
    }
  }

  @override
  Widget build(BuildContext context) {
    return OutlinedButton.icon(
      onPressed: _handleOidcLogin,
      icon: const Icon(Icons.login),
      label: const Text('Login with OIDC'),
      style: OutlinedButton.styleFrom(
        padding: const EdgeInsets.symmetric(vertical: 16),
      ),
    );
  }
}
