import type { Message as LibreChatMessage } from '@/types/librechat';
import type { Message as T3ChatMessage } from '@/types/chat';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { FileIcon, FileTextIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

type Message = LibreChatMessage | T3ChatMessage;

function isLibreChatMessage(msg: Message): msg is LibreChatMessage {
  return 'isCreatedByUser' in msg;
}

export function MessageBubble({ message, streaming }: { message: Message; streaming?: boolean }) {
  const isUser = isLibreChatMessage(message)
    ? message.isCreatedByUser
    : message.role === 'user';

  // Get text content
  const text = isLibreChatMessage(message) ? message.text : message.content;

  // Get multimodal content if available
  const content = isLibreChatMessage(message) ? message.content : undefined;

  // Check if message has files
  const hasFiles = isLibreChatMessage(message) && message.fileIds && message.fileIds.length > 0;

  return (
    <div className={cn('flex w-full gap-3', isUser ? 'justify-end' : 'justify-start')}>
      {!isUser && (
        <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
          <AvatarFallback className="text-xs font-medium">AI</AvatarFallback>
        </Avatar>
      )}
      <div className={cn('flex max-w-[80%] flex-col', isUser ? 'items-end text-right' : 'items-start text-left')}>
        <span className="mb-1.5 text-xs uppercase tracking-[0.16em] text-muted-foreground">
          {isUser ? 'You' : 'Assistant'}
        </span>
        <div
          className={cn(
            'w-full rounded-xl border px-4 py-3 shadow-sm transition',
            isUser
              ? 'border-foreground/20 bg-foreground text-background'
              : 'border-border bg-background text-foreground'
          )}
        >
          {/* Multimodal content blocks */}
          {content && content.length > 0 ? (
            <div className="space-y-2">
              {content.map((block, index) => (
                <div key={index}>
                  {block.type === 'text' && (
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">{block.text}</p>
                  )}
                  {block.type === 'image_url' && block.image_url && (
                    <div className="relative rounded-lg overflow-hidden">
                      <img
                        src={block.image_url.url}
                        alt="Uploaded image"
                        className="max-w-full h-auto"
                      />
                    </div>
                  )}
                  {block.type === 'file' && block.file && (
                    <div className="flex items-center gap-2 rounded-md border border-border bg-background/50 p-2">
                      <FileIcon className="h-4 w-4 text-muted-foreground" />
                      <span className="text-xs truncate">{block.file.filename || 'File'}</span>
                    </div>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-2">
              {/* Simple text content */}
              <p className="text-sm leading-relaxed whitespace-pre-wrap">
                {text}
                {streaming && <span className="inline-block ml-1 w-2 h-4 bg-current animate-pulse">|</span>}
              </p>
              
              {/* File attachments */}
              {hasFiles && isLibreChatMessage(message) && message.fileIds && (
                <div className="flex flex-wrap gap-2 mt-2">
                  {message.fileIds.map((fileId, index) => (
                    <div
                      key={fileId}
                      className="flex items-center gap-2 rounded-md border border-border bg-background/50 px-2 py-1"
                    >
                      <FileTextIcon className="h-3 w-3 text-muted-foreground" />
                      <span className="text-xs">Attachment {index + 1}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Token count and model info */}
          {isLibreChatMessage(message) && (message.tokenCount || message.model) && (
            <div className="mt-2 pt-2 border-t border-current/10 flex items-center gap-2 text-xs opacity-60">
              {message.model && <span>{message.model}</span>}
              {message.tokenCount && <span>• {message.tokenCount} tokens</span>}
              {message.finishReason && <span>• {message.finishReason}</span>}
            </div>
          )}

          {/* Error indicator */}
          {isLibreChatMessage(message) && message.error && (
            <div className="mt-2 text-xs text-red-500">
              ⚠ Error generating response
            </div>
          )}
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

