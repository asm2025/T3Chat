import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/repositories/auth_repository_impl.dart';
import '../../data/repositories/auth_repository_rust_impl.dart';
import '../../data/repositories/chat_repository_impl.dart';
import '../../data/repositories/chat_repository_rust_impl.dart';
import '../../data/repositories/models_repository_impl.dart';
import '../../data/repositories/user_repository_impl.dart';
import '../../data/repositories/user_repository_rust_impl.dart';
import '../../domain/repositories/auth_repository.dart';
import '../../domain/repositories/chat_repository.dart';
import '../../domain/repositories/models_repository.dart';
import '../../domain/repositories/user_repository.dart';
import '../network/api_client.dart';

// Platform detection helper
bool get _isDesktopPlatform {
  if (kIsWeb) return true;
  return Platform.isWindows || Platform.isLinux || Platform.isMacOS;
}

// API Client provider (mobile only)
final apiClientProvider = Provider<ApiClient?>((ref) {
  if (_isDesktopPlatform) {
    return null; // Desktop uses Rust FFI
  }
  return ApiClient();
});

// Repository providers with platform detection
final authRepositoryProvider = Provider<AuthRepository>((ref) {
  if (_isDesktopPlatform) {
    return AuthRepositoryRustImpl();
  } else {
    final apiClient = ref.watch(apiClientProvider);
    if (apiClient == null) {
      throw StateError('ApiClient is null on mobile platform');
    }
    return AuthRepositoryImpl(apiClient);
  }
});

final chatRepositoryProvider = Provider<ChatRepository>((ref) {
  if (_isDesktopPlatform) {
    return ChatRepositoryRustImpl();
  } else {
    final apiClient = ref.watch(apiClientProvider);
    if (apiClient == null) {
      throw StateError('ApiClient is null on mobile platform');
    }
    return ChatRepositoryImpl(apiClient);
  }
});

final userRepositoryProvider = Provider<UserRepository>((ref) {
  if (_isDesktopPlatform) {
    return UserRepositoryRustImpl();
  } else {
    final apiClient = ref.watch(apiClientProvider);
    if (apiClient == null) {
      throw StateError('ApiClient is null on mobile platform');
    }
    return UserRepositoryImpl(apiClient);
  }
});

final modelsRepositoryProvider = Provider<ModelsRepository>((ref) {
  if (_isDesktopPlatform) {
    // TODO: Implement Rust models repository if needed
    throw UnimplementedError('Models repository not yet implemented for Rust');
  } else {
    final apiClient = ref.watch(apiClientProvider);
    if (apiClient == null) {
      throw StateError('ApiClient is null on mobile platform');
    }
    return ModelsRepositoryImpl(apiClient);
  }
});

