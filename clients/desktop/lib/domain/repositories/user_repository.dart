import '../models/user.dart';

abstract class UserRepository {
  Future<User> getProfile();
  Future<User> updateProfile({
    String? displayName,
    String? imageUrl,
  });
}

