/// Base exception class for all domain exceptions
abstract class DomainException implements Exception {
  final String message;
  final Object? originalError;

  const DomainException(this.message, [this.originalError]);

  @override
  String toString() => message;
}

/// Authentication-related exceptions
class AuthException extends DomainException {
  const AuthException(super.message, [super.originalError]);
}

/// Network-related exceptions
class NetworkException extends DomainException {
  const NetworkException(super.message, [super.originalError]);
}

/// Server error exceptions
class ServerException extends DomainException {
  final int? statusCode;
  const ServerException(super.message, this.statusCode, [super.originalError]);
}

/// Not found exceptions
class NotFoundException extends DomainException {
  const NotFoundException(super.message, [super.originalError]);
}

/// Validation exceptions
class ValidationException extends DomainException {
  const ValidationException(super.message, [super.originalError]);
}

/// Unknown exceptions
class UnknownException extends DomainException {
  const UnknownException(super.message, [super.originalError]);
}

