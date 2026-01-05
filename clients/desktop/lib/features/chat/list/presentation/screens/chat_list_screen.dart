import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../../auth/providers/auth_provider.dart';
import '../../providers/chat_list_provider.dart';
import '../widgets/chat_list_item.dart';

class ChatListScreen extends ConsumerWidget {
  const ChatListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chatsAsync = ref.watch(chatListProvider);
    final selectedChatId = ref.watch(selectedChatIdProvider);

    return Row(
      children: [
        // Sidebar with chat list
        Container(
          width: 300,
          decoration: BoxDecoration(
            border: Border(
              right: BorderSide(color: Theme.of(context).dividerColor),
            ),
          ),
          child: Column(
            children: [
              // Header
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  border: Border(
                    bottom: BorderSide(color: Theme.of(context).dividerColor),
                  ),
                ),
                child: Row(
                  children: [
                    Expanded(
                      child: ElevatedButton.icon(
                        onPressed: () {
                          // TODO: Create new chat
                        },
                        icon: const Icon(Icons.add),
                        label: const Text('New Chat'),
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.logout),
                      onPressed: () async {
                        await ref.read(authStateProvider.notifier).logout();
                        if (context.mounted) {
                          context.go('/login');
                        }
                      },
                    ),
                  ],
                ),
              ),
              // Chat list
              Expanded(
                child: chatsAsync.when(
                  data: (chats) => ListView.builder(
                    itemCount: chats.length,
                    itemBuilder: (context, index) {
                      final chat = chats[index];
                      return ChatListItem(
                        chat: chat,
                        isSelected: chat.id == selectedChatId,
                        onTap: () {
                          ref.read(selectedChatIdProvider.notifier).state = chat.id;
                          context.go('/app/chat/${chat.id}');
                        },
                      );
                    },
                  ),
                  loading: () => const Center(child: CircularProgressIndicator()),
                  error: (e, _) => Center(
                    child: Text('Error: $e'),
                  ),
                ),
              ),
            ],
          ),
        ),
        // Main content area
        Expanded(
          child: selectedChatId == null
              ? const Center(
                  child: Text('Select a chat to start'),
                )
              : const Center(
                  child: Text('Chat detail will be shown here'),
                ),
        ),
      ],
    );
  }
}

