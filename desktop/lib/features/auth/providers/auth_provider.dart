import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/providers/repository_providers.dart';
import '../../../domain/models/user.dart';
import '../../../domain/repositories/auth_repository.dart';

final authStateProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return AuthNotifier(ref.read(authRepositoryProvider));
});

class AuthState {
  final User? user;
  final bool isLoading;
  final String? error;

  AuthState({
    this.user,
    this.isLoading = false,
    this.error,
  });

  bool get isAuthenticated => user != null;
}

class AuthNotifier extends StateNotifier<AuthState> {
  final AuthRepository _repository;

  AuthNotifier(this._repository) : super(AuthState()) {
    _checkAuth();
  }

  Future<void> _checkAuth() async {
    state = AuthState(isLoading: true);
    try {
      final user = await _repository.getCurrentUser();
      state = AuthState(user: user);
    } catch (e) {
      state = AuthState();
    }
  }

  Future<void> login(String username, String password) async {
    state = AuthState(isLoading: true);
    try {
      final result = await _repository.login(username, password);
      state = AuthState(user: result.user);
    } catch (e) {
      state = AuthState(error: e.toString());
    }
  }

  Future<void> logout() async {
    try {
      await _repository.logout();
      state = AuthState();
    } catch (e) {
      state = AuthState(error: e.toString());
    }
  }
}

