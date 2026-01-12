import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../../../../core/error/exceptions.dart';
import '../../../../auth/providers/auth_provider.dart';
import '../../providers/chat_list_provider.dart';
import '../widgets/chat_list_item.dart';

class ChatListScreen extends ConsumerWidget {
  const ChatListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chatsAsync = ref.watch(chatListProvider);
    final selectedChatId = ref.watch(selectedChatIdProvider);

    return Scaffold(
      body: SafeArea(
        child: Row(
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
                              ref.read(selectedChatIdProvider.notifier).selectChat(chat.id);
                              context.go('/app/chat/${chat.id}');
                            },
                          );
                        },
                      ),
                      loading: () => const Center(child: CircularProgressIndicator()),
                      error: (e, _) {
                        final message = e is DomainException ? e.message : e.toString();
                        final isAuth = e is AuthException ||
                            message.toLowerCase().contains('unauthorized') ||
                            message.toLowerCase().contains('ffierror.auth');

                        return Center(
                          child: Padding(
                            padding: const EdgeInsets.all(16),
                            child: Column(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Icon(
                                  isAuth ? Icons.lock_outline : Icons.error_outline,
                                  size: 32,
                                  color: Theme.of(context).colorScheme.error,
                                ),
                                const SizedBox(height: 12),
                                Text(
                                  isAuth ? 'Session expired' : 'Failed to load chats',
                                  style: Theme.of(context).textTheme.titleMedium,
                                  textAlign: TextAlign.center,
                                ),
                                const SizedBox(height: 8),
                                Text(
                                  message,
                                  style: Theme.of(context).textTheme.bodySmall,
                                  textAlign: TextAlign.center,
                                ),
                                const SizedBox(height: 16),
                                Wrap(
                                  spacing: 8,
                                  runSpacing: 8,
                                  alignment: WrapAlignment.center,
                                  children: [
                                    OutlinedButton(
                                      onPressed: () => ref.refresh(chatListProvider),
                                      child: const Text('Retry'),
                                    ),
                                    if (isAuth)
                                      FilledButton(
                                        onPressed: () async {
                                          await ref.read(authStateProvider.notifier).logout();
                                          if (context.mounted) {
                                            context.go('/login');
                                          }
                                        },
                                        child: const Text('Sign in again'),
                                      ),
                                  ],
                                ),
                              ],
                            ),
                          ),
                        );
                      },
                    ),
                  ),
                ],
              ),
            ),
            // Main content area
            Expanded(
              child: selectedChatId == null
                  ? const Center(child: Text('Select a chat to start'))
                  : const Center(child: Text('Chat detail will be shown here')),
            ),
          ],
        ),
      ),
    );
  }
}
