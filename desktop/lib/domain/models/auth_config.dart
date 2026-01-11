import 'package:json_annotation/json_annotation.dart';

part '../../../../clients/desktop/lib/domain/models/auth_config.g.dart';

@JsonSerializable()
class AuthConfig {
  @JsonKey(name: 'oidcEnabled')
  final bool oidcEnabled;

  AuthConfig({required this.oidcEnabled});

  factory AuthConfig.fromJson(Map<String, dynamic> json) =>
      _$AuthConfigFromJson(json);
  Map<String, dynamic> toJson() => _$AuthConfigToJson(this);
}
