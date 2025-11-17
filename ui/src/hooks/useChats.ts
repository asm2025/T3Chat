import { useState, useEffect } from 'react';
import { t3ChatClient } from '@/lib/t3-chat-client';
import type { Chat } from '@/types/chat';

export function useChats() {
  const [chats, setChats] = useState<Chat[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const loadChats = async () => {
    try {
      setLoading(true);
      const result = await t3ChatClient.listChats();
      setChats(result.data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadChats();
  }, []);

  return { chats, loading, error, refresh: loadChats };
}

