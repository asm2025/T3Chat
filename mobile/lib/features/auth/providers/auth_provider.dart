import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/providers/api_providers.dart';
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

final authStateProvider = StateNotifierProvider<AuthStateNotifier, AsyncValue<User?>>((ref) {
  return AuthStateNotifier(ref);
});

class AuthStateNotifier extends StateNotifier<AsyncValue<User?>> {
  final Ref _ref;
  final AuthRepository _authRepo;

  AuthStateNotifier(this._ref)
      : _authRepo = _ref.read(authRepositoryProvider),
        super(const AsyncValue.loading()) {
    _loadUser();
  }

  Future<void> _loadUser() async {
    state = const AsyncValue.loading();
    try {
      final user = await _authRepo.getCurrentUser();
      state = AsyncValue.data(user);
    } catch (e, stack) {
      state = AsyncValue.error(e, stack);
    }
  }

  Future<void> login(String username, String password) async {
    state = const AsyncValue.loading();
    try {
      final userAndToken = await _authRepo.login(username, password);
      state = AsyncValue.data(userAndToken.user);
    } catch (e, stack) {
      state = AsyncValue.error(e, stack);
      rethrow;
    }
  }

  Future<void> logout() async {
    try {
      await _authRepo.logout();
      state = const AsyncValue.data(null);
    } catch (e, stack) {
      state = AsyncValue.error(e, stack);
    }
  }

  void refresh() {
    _loadUser();
  }
}

