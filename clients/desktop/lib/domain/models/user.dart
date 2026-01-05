import 'package:json_annotation/json_annotation.dart';

part 'user.g.dart';

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
  final User user;
  final String token;
  @JsonKey(name: 'expiresAt')
  final int expiresAt;

  UserAndToken({
    required this.user,
    required this.token,
    required this.expiresAt,
  });

  factory UserAndToken.fromJson(Map<String, dynamic> json) =>
      _$UserAndTokenFromJson(json);
  Map<String, dynamic> toJson() => _$UserAndTokenToJson(this);
}

