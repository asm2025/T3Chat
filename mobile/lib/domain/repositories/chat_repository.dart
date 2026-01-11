import '../models/chat.dart';
import '../models/message.dart';

abstract class ChatRepository {
  Future<List<Chat>> listChats({int? page, int? pageSize});
  Future<Chat> createChat({
    String? title,
    required String modelProvider,
    required String modelId,
  });
  Future<Chat> getChat(String chatId);
  Future<Chat> updateChat(String chatId, {String? title});
  Future<void> deleteChat(String chatId);
  Future<List<Message>> getMessages(String chatId);
  Future<Message> sendMessage(
    String chatId,
    String content, {
    required String modelProvider,
    required String modelId,
  });
  Stream<ChatMessageEvent> streamMessage(
    String chatId,
    String content, {
    required String modelProvider,
    required String modelId,
  });
}

