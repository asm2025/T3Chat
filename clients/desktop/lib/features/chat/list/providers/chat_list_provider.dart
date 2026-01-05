import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../../core/providers/repository_providers.dart';
import '../../../../domain/models/chat.dart';
import '../../../../domain/repositories/chat_repository.dart';

final chatListProvider = FutureProvider.autoDispose<List<Chat>>((ref) async {
  final repository = ref.read(chatRepositoryProvider);
  final result = await repository.listChats();
  return result;
});

final selectedChatIdProvider = StateProvider<String?>((ref) => null);

