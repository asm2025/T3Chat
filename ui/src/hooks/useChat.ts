import { useState, useEffect } from 'react';
import { t3ChatClient } from '@/lib/t3-chat-client';
import { toast } from '@/lib/toast';
import type { ChatWithMessages } from '@/types/chat';

export function useChat(chatId: string | null) {
  const [chat, setChat] = useState<ChatWithMessages | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!chatId) {
      setChat(null);
      setLoading(false);
      return;
    }

    const loadChat = async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await t3ChatClient.getChat(chatId);
        setChat(data);
      } catch (err) {
        const error = err as Error;
        setError(error);
        toast.error("Failed to load chat", {
          description: error.message,
        });
      } finally {
        setLoading(false);
      }
    };

    loadChat();
  }, [chatId]);

  return { chat, loading, error };
}

