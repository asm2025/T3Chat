import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
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
        const data = await api.getChat(chatId);
        setChat(data);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    loadChat();
  }, [chatId]);

  return { chat, loading, error };
}

