import 'package:freezed_annotation/freezed_annotation.dart';

part 'chat_message_event.freezed.dart';

@freezed
class ChatMessageEvent with _$ChatMessageEvent {
  const factory ChatMessageEvent.chunk({required String delta}) = ChatChunk;
  const factory ChatMessageEvent.done() = ChatDone;
  const factory ChatMessageEvent.error(String message) = ChatError;
}

