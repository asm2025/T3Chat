import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/providers/api_providers.dart';
import '../../../domain/models/chat.dart';
import '../../../domain/models/message.dart';
import '../../../domain/repositories/chat_repository.dart';

final chatDetailProvider = FutureProvider.autoDispose.family<Chat, String>((ref, chatId) async {
  final repo = ref.watch(chatRepositoryProvider);
  return repo.getChat(chatId);
});

final messagesProvider = FutureProvider.autoDispose.family<List<Message>, String>((ref, chatId) async {
  final repo = ref.watch(chatRepositoryProvider);
  return repo.getMessages(chatId);
});

final chatStreamParamsProvider = StateProvider<ChatStreamParams?>((ref) => null);

class ChatStreamParams {
  final String chatId;
  final String content;
  final String modelProvider;
  final String modelId;

  ChatStreamParams({
    required this.chatId,
    required this.content,
    required this.modelProvider,
    required this.modelId,
  });
}

final messageStreamProvider = StreamProvider.autoDispose.family<ChatMessageEvent, ChatStreamParams>((ref, params) async* {
  final repo = ref.watch(chatRepositoryProvider);
  yield* repo.streamMessage(
    params.chatId,
    params.content,
    modelProvider: params.modelProvider,
    modelId: params.modelId,
  );
});

