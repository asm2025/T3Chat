import '../models/user.dart';

abstract class AuthRepository {
  Future<AuthConfig> getAuthConfig();
  Future<UserAndToken> login(String username, String password);
  /// Persist a token obtained via an external flow (e.g. OIDC callback) so that
  /// subsequent API calls are authorized.
  Future<void> setToken(String token);
  Future<void> logout();
  Future<User?> getCurrentUser();
}

