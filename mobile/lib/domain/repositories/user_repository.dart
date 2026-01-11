import '../models/user.dart';

abstract class UserRepository {
  Future<User> getProfile();
  Future<User> updateProfile({
    String? name,
    String? username,
    String? avatarUrl,
  });
}

