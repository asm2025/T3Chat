import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/chat_detail_provider.dart';
import '../widgets/message_bubble.dart';
import '../widgets/message_input.dart';

class ChatDetailScreen extends ConsumerWidget {
  final String chatId;

  const ChatDetailScreen({super.key, required this.chatId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chatAsync = ref.watch(chatDetailProvider(chatId));
    final messagesAsync = ref.watch(messagesProvider(chatId));

    return Scaffold(
      appBar: AppBar(
        title: chatAsync.when(
          data: (chat) => Text(chat.title),
          loading: () => const Text('Loading...'),
          error: (_, __) => const Text('Error'),
        ),
      ),
      body: Column(
        children: [
          Expanded(
            child: messagesAsync.when(
              data: (messages) => ListView.builder(
                padding: const EdgeInsets.all(16),
                itemCount: messages.length,
                itemBuilder: (context, index) {
                  final message = messages[index];
                  return MessageBubble(message: message);
                },
              ),
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(child: Text('Error: $e')),
            ),
          ),
          MessageInput(chatId: chatId),
        ],
      ),
    );
  }
}

