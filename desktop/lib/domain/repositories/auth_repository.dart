import '../models/auth_config.dart';
import '../models/user.dart';

abstract class AuthRepository {
  Future<AuthConfig> getAuthConfig();
  Future<UserAndToken> login(String username, String password);
  Future<void> logout();
  Future<User?> getCurrentUser();
}

