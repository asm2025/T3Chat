import { useState, useCallback, useRef } from 'react';
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from '@/lib/toast';
import { getErrorMessage } from '@/lib/utils';
import type { ChatCompletionRequest, StreamChunk } from '@/types/librechat';

interface StreamingState {
  streaming: boolean;
  currentMessageId: string | null;
  error: Error | null;
}

export function useLibreChatStreaming() {
  const [state, setState] = useState<StreamingState>({
    streaming: false,
    currentMessageId: null,
    error: null,
  });
  const abortControllerRef = useRef<AbortController | null>(null);

  const sendMessage = useCallback(async (
    request: ChatCompletionRequest,
    onChunk: (chunk: StreamChunk) => void,
    onComplete: (messageId?: string) => void,
    onError?: (error: Error) => void
  ) => {
    try {
      setState({
        streaming: true,
        currentMessageId: request.conversationId || null,
        error: null,
      });

      // Create abort controller for cancellation
      abortControllerRef.current = new AbortController();

      // Start streaming
      const response = await t3ChatClient.chat.streamMessage(request);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error('No response body');
      }

      const decoder = new TextDecoder();
      let buffer = '';

      try {
        while (true) {
          const { done, value } = await reader.read();
          
          if (done) {
            setState((prev) => ({ ...prev, streaming: false }));
            onComplete();
            break;
          }

          // Decode the chunk and add to buffer
          buffer += decoder.decode(value, { stream: true });
          
          // Process complete lines
          const lines = buffer.split('\n');
          buffer = lines.pop() || ''; // Keep incomplete line in buffer

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const data = line.slice(6).trim();
              
              if (data === '[DONE]') {
                setState((prev) => ({ ...prev, streaming: false }));
                onComplete();
                return;
              }

              try {
                const chunk: StreamChunk = JSON.parse(data);
                onChunk(chunk);
              } catch (err) {
                console.error('Error parsing chunk:', err, data);
              }
            }
          }
        }
      } finally {
        reader.releaseLock();
      }

    } catch (err) {
      const error = err instanceof Error ? err : new Error(getErrorMessage(err));
      setState({
        streaming: false,
        currentMessageId: null,
        error,
      });
      
      if (onError) {
        onError(error);
      } else {
        toast.error('Failed to send message', {
          description: error.message,
        });
      }
    }
  }, []);

  const cancelStreaming = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
  }, []);

  return {
    ...state,
    sendMessage,
    cancelStreaming,
  };
}

