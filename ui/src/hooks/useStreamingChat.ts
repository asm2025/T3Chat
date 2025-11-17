import { useState, useCallback } from 'react';
import { t3ChatClient } from '@/lib/t3-chat-client';
import { toast } from '@/lib/toast';
import type { ChatRequest } from '@/types/api';

export function useStreamingChat() {
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const sendMessage = useCallback(async (
    request: ChatRequest,
    onChunk: (chunk: string) => void,
    onComplete: () => void
  ) => {
    try {
      setStreaming(true);
      setError(null);

      for await (const chunk of t3ChatClient.streamChat(request)) {
        onChunk(chunk);
      }

      onComplete();
    } catch (err) {
      const error = err as Error;
      setError(error);
      toast.error("Failed to send message", {
        description: error.message,
      });
    } finally {
      setStreaming(false);
    }
  }, []);

  return { sendMessage, streaming, error };
}

