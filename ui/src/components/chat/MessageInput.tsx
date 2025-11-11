import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { ArrowUp } from 'lucide-react';

export function MessageInput({
  onSend,
  disabled
}: {
  onSend: (content: string) => void;
  disabled?: boolean;
}) {
  const [content, setContent] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (content.trim() && !disabled) {
      onSend(content);
      setContent('');
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="mx-auto w-full max-w-3xl rounded-xl border border-border bg-background p-4 shadow-sm"
    >
      <Textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder="Type your message here..."
        disabled={disabled}
        rows={4}
        className="min-h-[100px] resize-none border-none bg-transparent text-sm leading-relaxed outline-none focus-visible:ring-0"
      />
      <div className="mt-3 flex flex-col gap-2 sm:flex-row sm:items-center">
        <p className="text-xs text-muted-foreground">Shift + Enter to add a new line</p>
        <div className="flex w-full justify-end">
          <Button
            type="submit"
            disabled={disabled || !content.trim()}
            className="inline-flex items-center gap-2 rounded-full bg-foreground text-background shadow-sm hover:bg-foreground/90"
          >
            Send
            <ArrowUp className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </form>
  );
}

