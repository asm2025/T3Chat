import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/providers/repository_providers.dart';
import '../../../domain/models/auth_config.dart';
import '../../../domain/repositories/auth_repository.dart';

final authConfigProvider = FutureProvider<AuthConfig>((ref) async {
  final repository = ref.read(authRepositoryProvider);
  return repository.getAuthConfig();
});

