import { useState, useCallback } from 'react';
import * as api from '@/lib/serverComm';
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

      for await (const chunk of api.streamChat(request)) {
        onChunk(chunk);
      }

      onComplete();
    } catch (err) {
      setError(err as Error);
    } finally {
      setStreaming(false);
    }
  }, []);

  return { sendMessage, streaming, error };
}

