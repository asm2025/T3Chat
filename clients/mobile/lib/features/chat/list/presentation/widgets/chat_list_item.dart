import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../../../domain/models/chat.dart';

class ChatListItem extends StatelessWidget {
  final Chat chat;
  final VoidCallback onTap;
  final VoidCallback onDelete;

  const ChatListItem({
    super.key,
    required this.chat,
    required this.onTap,
    required this.onDelete,
  });

  String _formatDate(String dateString) {
    try {
      final date = DateTime.parse(dateString);
      final now = DateTime.now();
      final difference = now.difference(date);

      if (difference.inDays == 0) {
        return DateFormat('HH:mm').format(date);
      } else if (difference.inDays < 7) {
        return DateFormat('EEE').format(date);
      } else {
        return DateFormat('MMM d').format(date);
      }
    } catch (e) {
      return dateString;
    }
  }

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(
        chat.title,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(
        '${chat.modelProvider}/${chat.modelId}',
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            _formatDate(chat.updatedAt),
            style: Theme.of(context).textTheme.bodySmall,
          ),
          IconButton(
            icon: const Icon(Icons.delete_outline),
            onPressed: onDelete,
          ),
        ],
      ),
      onTap: onTap,
    );
  }
}

