import 'package:dio/dio.dart';
import '../../config/app_config.dart';
import '../../logging/logger.dart';

class LoggingInterceptor extends Interceptor {
  final AppLogger _logger = AppLogger();

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (AppConfig.isDebugMode) {
      _logger.debug(
        'REQUEST[${options.method}] => PATH: ${options.path}',
      );
    }
    handler.next(options);
  }

  @override
  void onResponse(Response<dynamic> response, ResponseInterceptorHandler handler) {
    if (AppConfig.isDebugMode) {
      _logger.debug(
        'RESPONSE[${response.statusCode}] => PATH: ${response.requestOptions.path}',
      );
    }
    handler.next(response);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    if (AppConfig.isDebugMode) {
      _logger.error(
        'ERROR[${err.response?.statusCode}] => PATH: ${err.requestOptions.path}',
        err,
      );
    }
    handler.next(err);
  }
}
