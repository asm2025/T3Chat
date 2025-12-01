import React, { useState, useImperativeHandle, forwardRef } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { ArrowUp, X } from 'lucide-react';
import { cn } from '@/lib/utils';

interface LibreChatMessageInputProps {
  onSend: (content: string) => void;
  disabled?: boolean;
  onCancel?: () => void;
}

export interface MessageInputRef {
  setContent: (content: string) => void;
  clearContent: () => void;
}

export const LibreChatMessageInput = forwardRef<MessageInputRef, LibreChatMessageInputProps>(
  ({ onSend, disabled, onCancel }, ref) => {
    const [content, setContent] = useState('');

    // Expose methods via ref
    useImperativeHandle(ref, () => ({
      setContent: (newContent: string) => {
        setContent(newContent);
      },
      clearContent: () => {
        setContent('');
      },
    }));

    const handleSubmit = (e: React.FormEvent) => {
      e.preventDefault();
      if (content.trim() && !disabled) {
        onSend(content);
        setContent('');
      }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        if (content.trim() && !disabled) {
          onSend(content);
          setContent('');
        }
      }
    };

    return (
      <div className="border-reflect pointer-events-none min-w-0 overflow-hidden rounded-t-[20px] bg-chat-input-background p-2 pb-0 backdrop-blur-lg">
        <form
          onSubmit={handleSubmit}
          className={cn(
            'text-secondary-foreground pb-safe-offset-3 dark:bg-secondary/4.5 dark:outline-chat-background/40',
            'pointer-events-auto relative flex w-full min-w-0 flex-col items-stretch gap-2',
            'rounded-t-xl border border-b-0 border-white/70 bg-chat-input-background px-3 pt-3',
            'outline-8 outline-solid max-sm:pb-6 sm:max-w-3xl dark:border-[hsl(0,100%,83%)]/4',
            'shadow-[rgba(0,0,0,0.1)_0px_80px_50px_0px,rgba(0,0,0,0.07)_0px_50px_30px_0px,rgba(0,0,0,0.06)_0px_30px_15px_0px,rgba(0,0,0,0.04)_0px_15px_8px,rgba(0,0,0,0.04)_0px_6px_4px,rgba(0,0,0,0.02)_0px_2px_2px]'
          )}
          id="chat-input-form"
        >
          <div className="flex min-w-0 grow flex-row items-start">
            <Textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type your message here..."
              disabled={disabled}
              id="chat-input"
              aria-label="Message input"
              aria-describedby="chat-input-description"
              autoComplete="off"
              className="text-foreground placeholder:text-secondary-foreground/60 w-full min-w-0 resize-none bg-transparent text-base leading-6 outline-none disabled:opacity-50 border-0 p-0 focus-visible:ring-0 shadow-none"
              style={{ height: '48px' }}
            />
            <div id="chat-input-description" className="sr-only">
              Press Enter to send, Shift + Enter for new line
            </div>
          </div>
          
          <div className="mt-2 -mb-px flex w-full min-w-0 flex-row-reverse justify-between">
            {/* Send/Cancel Button */}
            <div className="-mt-0.5 -mr-0.5 flex shrink-0 items-center justify-center gap-2" aria-label="Message actions">
              {disabled && onCancel ? (
                <Button
                  type="button"
                  onClick={onCancel}
                  className={cn(
                    'size-9 relative rounded-lg p-2 font-semibold shadow-sm',
                    'bg-destructive hover:bg-destructive/90',
                    'text-destructive-foreground border-reflect button-reflect'
                  )}
                  aria-label="Cancel generation"
                >
                  <X className="size-5" />
                </Button>
              ) : (
                <Button
                  type="submit"
                  disabled={disabled || !content.trim()}
                  className={cn(
                    'size-9 relative rounded-lg p-2 font-semibold shadow-sm',
                    'bg-foreground hover:bg-foreground/90 active:bg-foreground',
                    'disabled:hover:bg-foreground/50 disabled:active:bg-foreground/50',
                    'text-background border-reflect button-reflect'
                  )}
                  aria-label={content.trim() ? 'Send message' : 'Message requires text'}
                >
                  <ArrowUp className="size-5" />
                </Button>
              )}
            </div>

            <div className="flex min-w-0 flex-1 items-center gap-2 pr-2">
              <div className="text-xs text-muted-foreground">
                {disabled ? 'Generating...' : 'Press Enter to send'}
              </div>
            </div>
          </div>
        </form>
      </div>
    );
  }
);

LibreChatMessageInput.displayName = 'LibreChatMessageInput';

