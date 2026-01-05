import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/repositories/auth_repository_rust_impl.dart';
import '../../data/repositories/chat_repository_rust_impl.dart';
import '../../data/repositories/user_repository_rust_impl.dart';
import '../../domain/repositories/auth_repository.dart';
import '../../domain/repositories/chat_repository.dart';
import '../../domain/repositories/user_repository.dart';

final authRepositoryProvider = Provider<AuthRepository>((ref) {
  return AuthRepositoryRustImpl();
});

final chatRepositoryProvider = Provider<ChatRepository>((ref) {
  return ChatRepositoryRustImpl();
});

final userRepositoryProvider = Provider<UserRepository>((ref) {
  return UserRepositoryRustImpl();
});

