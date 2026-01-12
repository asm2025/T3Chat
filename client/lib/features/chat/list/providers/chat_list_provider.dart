import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../../core/providers/repository_providers.dart';
import '../../../../domain/models/chat.dart';

final chatListProvider = FutureProvider.autoDispose<List<Chat>>((ref) async {
  final repository = ref.read(chatRepositoryProvider);
  final result = await repository.listChats();
  return result;
});

final selectedChatIdProvider = NotifierProvider<SelectedChatIdNotifier, String?>(() {
  return SelectedChatIdNotifier();
});

class SelectedChatIdNotifier extends Notifier<String?> {
  @override
  String? build() => null;

  void selectChat(String? chatId) {
    state = chatId;
  }
}

