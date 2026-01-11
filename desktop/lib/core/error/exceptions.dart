class AppException implements Exception {
  final String message;
  AppException(this.message);
}

class NetworkException extends AppException {
  NetworkException(super.message);
}

class AuthException extends AppException {
  AuthException(super.message);
}

class NotFoundException extends AppException {
  NotFoundException(super.message);
}

class ServerException extends AppException {
  ServerException(super.message);
}

