import 'package:json_annotation/json_annotation.dart';

part '../../../../clients/desktop/lib/domain/models/message.g.dart';

@JsonSerializable()
class Message {
  final String id;
  @JsonKey(name: 'chatId')
  final String chatId;
  final String role;
  final String content;
  final Map<String, dynamic>? metadata;
  @JsonKey(name: 'parentMessageId')
  final String? parentMessageId;
  @JsonKey(name: 'sequenceNumber')
  final int sequenceNumber;
  @JsonKey(name: 'createdAt')
  final String createdAt;
  @JsonKey(name: 'tokensUsed')
  final int? tokensUsed;
  @JsonKey(name: 'modelUsed')
  final String? modelUsed;

  Message({
    required this.id,
    required this.chatId,
    required this.role,
    required this.content,
    this.metadata,
    this.parentMessageId,
    this.sequenceNumber = 0,
    required this.createdAt,
    this.tokensUsed,
    this.modelUsed,
  });

  factory Message.fromJson(Map<String, dynamic> json) =>
      _$MessageFromJson(json);
  Map<String, dynamic> toJson() => _$MessageToJson(this);
}
