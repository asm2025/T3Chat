import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../../core/providers/repository_providers.dart';
import '../../../../domain/models/chat.dart';
import '../../../../domain/models/message.dart';
import '../../../../domain/repositories/chat_repository.dart';

final chatDetailProvider = FutureProvider.autoDispose.family<Chat, String>(
  (ref, chatId) async {
    final repository = ref.read(chatRepositoryProvider);
    return repository.getChat(chatId);
  },
);

final messagesProvider = FutureProvider.autoDispose.family<List<Message>, String>(
  (ref, chatId) async {
    final repository = ref.read(chatRepositoryProvider);
    return repository.getMessages(chatId);
  },
);

