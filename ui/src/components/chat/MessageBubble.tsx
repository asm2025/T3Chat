import type { Message } from '@/types/chat';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';

export function MessageBubble({ message }: { message: Message }) {
  const isUser = message.role === 'user';

  return (
    <div className={`flex gap-3 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
      <Avatar>
        <AvatarFallback>
          {isUser ? 'U' : 'AI'}
        </AvatarFallback>
      </Avatar>
      <div className={`flex-1 ${isUser ? 'text-right' : 'text-left'}`}>
        <div className={`inline-block p-3 rounded-lg ${
          isUser ? 'bg-primary text-primary-foreground' : 'bg-muted'
        }`}>
          {message.content}
        </div>
      </div>
    </div>
  );
}

