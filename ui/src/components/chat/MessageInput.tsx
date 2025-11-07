import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';

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
    <form onSubmit={handleSubmit} className="border-t p-4">
      <div className="flex gap-2">
        <Textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Type your message..."
          disabled={disabled}
          rows={3}
          className="flex-1"
        />
        <Button type="submit" disabled={disabled || !content.trim()}>
          Send
        </Button>
      </div>
    </form>
  );
}

