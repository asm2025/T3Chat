import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../../../../../clients/mobile/lib/features/domain/models/message.dart';
import '../../providers/chat_detail_provider.dart';
import '../widgets/message_bubble.dart';
import '../widgets/message_input.dart';
import '../widgets/streaming_message.dart';

class ChatDetailScreen extends ConsumerStatefulWidget {
  final String chatId;

  const ChatDetailScreen({super.key, required this.chatId});

  @override
  ConsumerState<ChatDetailScreen> createState() => _ChatDetailScreenState();
}

class _ChatDetailScreenState extends ConsumerState<ChatDetailScreen> {
  final ScrollController _scrollController = ScrollController();
  String? _streamingContent;
  bool _isStreaming = false;

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _handleSendMessage(String content) async {
    final chatAsync = ref.read(chatDetailProvider(widget.chatId));
    final chat = await chatAsync.future;

    setState(() {
      _isStreaming = true;
      _streamingContent = '';
    });

    final params = ChatStreamParams(
      chatId: widget.chatId,
      content: content,
      modelProvider: chat.modelProvider,
      modelId: chat.modelId,
    );

    ref.read(chatStreamParamsProvider.notifier).state = params;

    // Watch the stream provider and handle events
    ref.listen<AsyncValue<ChatMessageEvent>>(messageStreamProvider(params), (
      previous,
      next,
    ) {
      next.whenData((event) {
        event.when(
          chunk: (delta) {
            setState(() {
              _streamingContent = (_streamingContent ?? '') + delta;
            });
            _scrollToBottom();
          },
          done: () {
            setState(() {
              _isStreaming = false;
              _streamingContent = null;
            });
            // Refresh messages
            ref.invalidate(messagesProvider(widget.chatId));
          },
          error: (error) {
            setState(() {
              _isStreaming = false;
              _streamingContent = null;
            });
            if (mounted) {
              ScaffoldMessenger.of(
                context,
              ).showSnackBar(SnackBar(content: Text('Error: $error')));
            }
          },
        );
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final chatAsync = ref.watch(chatDetailProvider(widget.chatId));
    final messagesAsync = ref.watch(messagesProvider(widget.chatId));

    return Scaffold(
      appBar: AppBar(
        title: chatAsync.when(
          data: (chat) => Text(chat.title),
          loading: () => const Text('Chat'),
          error: (_, __) => const Text('Chat'),
        ),
      ),
      body: Column(
        children: [
          Expanded(
            child: messagesAsync.when(
              data: (messages) {
                return ListView.builder(
                  controller: _scrollController,
                  padding: const EdgeInsets.all(16),
                  itemCount: messages.length + (_isStreaming ? 1 : 0),
                  itemBuilder: (context, index) {
                    if (index < messages.length) {
                      return MessageBubble(message: messages[index]);
                    } else {
                      return StreamingMessage(content: _streamingContent ?? '');
                    }
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (error, stack) =>
                  Center(child: Text('Error loading messages: $error')),
            ),
          ),
          MessageInput(onSend: _handleSendMessage, enabled: !_isStreaming),
        ],
      ),
    );
  }
}
