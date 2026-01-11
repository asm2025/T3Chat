import 'package:dio/dio.dart';
import '../../error/exceptions.dart';
import '../../logging/logger.dart';

class ErrorInterceptor extends Interceptor {
  final AppLogger _logger = AppLogger();

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    DomainException exception;

    switch (err.type) {
      case DioExceptionType.connectionTimeout:
      case DioExceptionType.sendTimeout:
      case DioExceptionType.receiveTimeout:
        exception = NetworkException('Connection timeout', err);
        break;
      case DioExceptionType.badResponse:
        final statusCode = err.response?.statusCode;
        if (statusCode == 401) {
          exception = AuthException('Unauthorized', err);
        } else if (statusCode == 403) {
          exception = AuthException('Forbidden', err);
        } else if (statusCode == 404) {
          exception = NotFoundException('Resource not found', err);
        } else if (statusCode != null && statusCode >= 500) {
          exception = ServerException('Server error', statusCode, err);
        } else {
          exception = ServerException(
            err.response?.data?['message'] ?? 'Request failed',
            statusCode,
            err,
          );
        }
        break;
      case DioExceptionType.cancel:
        exception = NetworkException('Request cancelled', err);
        break;
      case DioExceptionType.connectionError:
        exception = NetworkException('No internet connection', err);
        break;
      default:
        exception = UnknownException('Unknown error: ${err.message}', err);
    }

    _logger.error('API Error: ${exception.message}', err);

    handler.reject(
      DioException(
        requestOptions: err.requestOptions,
        error: exception,
        type: err.type,
        response: err.response,
      ),
    );
  }
}

