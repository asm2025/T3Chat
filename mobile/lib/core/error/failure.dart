import 'exceptions.dart';

/// Result type for operations that can fail
sealed class Result<T> {
  const Result();
}

/// Success result
final class Success<T> extends Result<T> {
  final T data;
  const Success(this.data);
}

/// Failure result
final class Failure<T> extends Result<T> {
  final DomainException error;
  const Failure(this.error);
}

/// Extension methods for Result
extension ResultExtensions<T> on Result<T> {
  bool get isSuccess => this is Success<T>;
  bool get isFailure => this is Failure<T>;

  T? get dataOrNull => switch (this) {
        Success(data: final d) => d,
        Failure() => null,
      };

  DomainException? get errorOrNull => switch (this) {
        Success() => null,
        Failure(error: final e) => e,
      };

  R fold<R>(R Function(DomainException error) onError, R Function(T data) onSuccess) {
    return switch (this) {
      Success(data: final d) => onSuccess(d),
      Failure(error: final e) => onError(e),
    };
  }
}

