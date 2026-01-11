import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import '../../logging/logger.dart';

class AuthInterceptor extends Interceptor {
  final FlutterSecureStorage _storage;
  final AppLogger _logger = AppLogger();

  AuthInterceptor(this._storage);

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) async {
    try {
      final token = await _storage.read(key: 'access_token');
      if (token != null) {
        options.headers['Authorization'] = 'Bearer $token';
      }
    } catch (e) {
      _logger.warning('Failed to read token from storage', e);
    }
    handler.next(options);
  }
}

