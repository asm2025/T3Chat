import 'package:json_annotation/json_annotation.dart';

part 'message.g.dart';

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
    required this.sequenceNumber,
    required this.createdAt,
    this.tokensUsed,
    this.modelUsed,
  });

  factory Message.fromJson(Map<String, dynamic> json) => _$MessageFromJson(json);
  Map<String, dynamic> toJson() => _$MessageToJson(this);
}

/// Event types for streaming chat messages
sealed class ChatMessageEvent {
  const ChatMessageEvent();

  T when<T>({
    required T Function(String delta) chunk,
    required T Function() done,
    required T Function(String error) error,
  }) {
    return switch (this) {
      ChatMessageChunk(delta: final d) => chunk(d),
      ChatMessageDone() => done(),
      ChatMessageError(error: final e) => error(e),
    };
  }
}

final class ChatMessageChunk extends ChatMessageEvent {
  final String delta;
  const ChatMessageChunk({required this.delta});
}

final class ChatMessageDone extends ChatMessageEvent {
  const ChatMessageDone();
}

final class ChatMessageError extends ChatMessageEvent {
  final String error;
  const ChatMessageError({required this.error});
}

