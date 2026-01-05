import 'package:json_annotation/json_annotation.dart';
import 'message.dart';

part 'chat.g.dart';

@JsonSerializable()
class Chat {
  final String id;
  @JsonKey(name: 'userId')
  final String userId;
  final String title;
  @JsonKey(name: 'modelProvider')
  final String modelProvider;
  @JsonKey(name: 'modelId')
  final String modelId;
  @JsonKey(name: 'createdAt')
  final String createdAt;
  @JsonKey(name: 'updatedAt')
  final String updatedAt;

  Chat({
    required this.id,
    required this.userId,
    required this.title,
    required this.modelProvider,
    required this.modelId,
    required this.createdAt,
    required this.updatedAt,
  });

  factory Chat.fromJson(Map<String, dynamic> json) => _$ChatFromJson(json);
  Map<String, dynamic> toJson() => _$ChatToJson(this);
}

@JsonSerializable()
class ChatListResponse {
  final List<Chat> data;
  final int total;

  ChatListResponse({
    required this.data,
    required this.total,
  });

  factory ChatListResponse.fromJson(Map<String, dynamic> json) => _$ChatListResponseFromJson(json);
  Map<String, dynamic> toJson() => _$ChatListResponseToJson(this);
}

@JsonSerializable()
class ChatWithMessagesResponse {
  final Chat chat;
  final List<Message> messages;

  ChatWithMessagesResponse({
    required this.chat,
    required this.messages,
  });

  factory ChatWithMessagesResponse.fromJson(Map<String, dynamic> json) => _$ChatWithMessagesResponseFromJson(json);
  Map<String, dynamic> toJson() => _$ChatWithMessagesResponseToJson(this);
}

