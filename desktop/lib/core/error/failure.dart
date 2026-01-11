import 'package:freezed_annotation/freezed_annotation.dart';

part '../../../../clients/desktop/lib/core/error/failure.freezed.dart';

@freezed
class Failure with _$Failure {
  const factory Failure.network(String message) = NetworkFailure;
  const factory Failure.auth(String message) = AuthFailure;
  const factory Failure.notFound(String message) = NotFoundFailure;
  const factory Failure.server(String message) = ServerFailure;
  const factory Failure.unknown(String message) = UnknownFailure;
}
