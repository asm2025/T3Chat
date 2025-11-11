import type { Message } from '@/types/chat';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';

export function MessageBubble({ message }: { message: Message }) {
  const isUser = message.role === 'user';

  return (
    <div className={`flex w-full gap-3 ${isUser ? 'justify-end' : 'justify-start'}`}>
      {!isUser && (
        <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
          <AvatarFallback className="text-xs font-medium">AI</AvatarFallback>
        </Avatar>
      )}
      <div className={`flex max-w-[80%] flex-col ${isUser ? 'items-end text-right' : 'items-start text-left'}`}>
        <span className="mb-1.5 text-xs uppercase tracking-[0.16em] text-muted-foreground">
          {isUser ? 'You' : 'Assistant'}
        </span>
        <div
          className={`w-full rounded-xl border px-4 py-3 text-sm leading-relaxed shadow-sm transition ${
            isUser
              ? 'border-foreground/20 bg-foreground text-background'
              : 'border-border bg-background text-foreground'
          }`}
        >
          {message.content}
        </div>
      </div>
      {isUser && (
        <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
          <AvatarFallback className="text-xs font-medium">You</AvatarFallback>
        </Avatar>
      )}
    </div>
  );
}

