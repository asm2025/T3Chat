import 'package:flutter/material.dart';
import '../../../../domain/models/chat.dart';

class ChatListItem extends StatelessWidget {
  final Chat chat;
  final bool isSelected;
  final VoidCallback onTap;

  const ChatListItem({
    super.key,
    required this.chat,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      selected: isSelected,
      title: Text(
        chat.title,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(
        '${chat.modelProvider}/${chat.modelId}',
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: TextStyle(
          fontSize: 12,
          color: Theme.of(context).textTheme.bodySmall?.color,
        ),
      ),
      onTap: onTap,
    );
  }
}

