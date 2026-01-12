import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/error/exceptions.dart';
import '../../../core/providers/repository_providers.dart';
import '../../../domain/models/user.dart';
import '../../../domain/repositories/auth_repository.dart';

final authConfigProvider = FutureProvider<AuthConfig>((ref) async {
  final repo = ref.watch(authRepositoryProvider);
  return repo.getAuthConfig();
});

final currentUserProvider = FutureProvider<User?>((ref) async {
  final repo = ref.watch(authRepositoryProvider);
  return repo.getCurrentUser();
});

final authStateProvider = NotifierProvider<AuthStateNotifier, AsyncValue<User?>>(() {
  return AuthStateNotifier();
});

class AuthStateNotifier extends Notifier<AsyncValue<User?>> {
  AuthRepository get _authRepo => ref.read(authRepositoryProvider);

  @override
  AsyncValue<User?> build() {
    _loadUser();
    return const AsyncValue.loading();
  }

  Future<void> _loadUser() async {
    state = const AsyncValue<User?>.loading();
    try {
      final user = await _authRepo.getCurrentUser();
      state = AsyncValue<User?>.data(user);
    } catch (e, stack) {
      state = AsyncValue<User?>.error(e, stack);
    }
  }

  Future<void> login(String username, String password) async {
    state = const AsyncValue<User?>.loading();
    try {
      final userAndToken = await _authRepo.login(username, password);
      state = AsyncValue<User?>.data(userAndToken.user);
    } catch (e, stack) {
      state = AsyncValue<User?>.error(e, stack);
    }
  }

  /// Used for OIDC flows where the server redirects back with a token.
  /// We persist the token, then resolve the current user from `/api/auth/me`.
  Future<void> loginWithToken(String token) async {
    state = const AsyncValue<User?>.loading();
    try {
      await _authRepo.setToken(token);
      final user = await _authRepo.getCurrentUser();
      if (user == null) {
        throw const AuthException('Unauthorized');
      }
      state = AsyncValue<User?>.data(user);
    } catch (e, stack) {
      state = AsyncValue<User?>.error(e, stack);
    }
  }

  Future<void> logout() async {
    try {
      await _authRepo.logout();
      state = const AsyncValue<User?>.data(null);
    } catch (e, stack) {
      state = AsyncValue<User?>.error(e, stack);
    }
  }

  void refresh() {
    _loadUser();
  }
}

