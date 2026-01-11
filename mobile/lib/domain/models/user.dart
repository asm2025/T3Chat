import 'package:json_annotation/json_annotation.dart';

part '../../../../clients/mobile/lib/domain/models/user.g.dart';

@JsonSerializable()
class User {
  final String id;
  final String email;
  @JsonKey(name: 'emailVerified')
  final bool? emailVerified;
  final String? name;
  final String? username;
  @JsonKey(name: 'avatarUrl')
  final String? avatarUrl;
  final List<String> roles;

  User({
    required this.id,
    required this.email,
    this.emailVerified,
    this.name,
    this.username,
    this.avatarUrl,
    required this.roles,
  });

  factory User.fromJson(Map<String, dynamic> json) => _$UserFromJson(json);
  Map<String, dynamic> toJson() => _$UserToJson(this);
}

@JsonSerializable()
class UserAndToken {
  final String token;
  @JsonKey(name: 'expiresAt')
  final int expiresAt;
  final User user;

  UserAndToken({
    required this.token,
    required this.expiresAt,
    required this.user,
  });

  factory UserAndToken.fromJson(Map<String, dynamic> json) =>
      _$UserAndTokenFromJson(json);
  Map<String, dynamic> toJson() => _$UserAndTokenToJson(this);
}

@JsonSerializable()
class AuthConfig {
  @JsonKey(name: 'oidcEnabled')
  final bool oidcEnabled;

  AuthConfig({required this.oidcEnabled});

  factory AuthConfig.fromJson(Map<String, dynamic> json) =>
      _$AuthConfigFromJson(json);
  Map<String, dynamic> toJson() => _$AuthConfigToJson(this);
}
