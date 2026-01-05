import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/providers/api_providers.dart';
import '../../../domain/models/chat.dart';
import '../../../domain/repositories/chat_repository.dart';

final chatListProvider = FutureProvider.autoDispose<List<Chat>>((ref) async {
  final repo = ref.watch(chatRepositoryProvider);
  return repo.listChats();
});

final chatListRefreshProvider = StateProvider<int>((ref) => 0);

final chatListWithRefreshProvider = FutureProvider.autoDispose<List<Chat>>((ref) {
  ref.watch(chatListRefreshProvider);
  final repo = ref.watch(chatRepositoryProvider);
  return repo.listChats();
});

