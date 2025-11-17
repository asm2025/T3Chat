import { useState, useEffect } from 'react';
import { t3ChatClient } from '@/lib/t3-chat-client';
import type { AIModel } from '@/types/model';

export function useModels() {
  const [models, setModels] = useState<AIModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const loadModels = async () => {
      try {
        setLoading(true);
        const data = await t3ChatClient.listModels();
        setModels(data);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    loadModels();
  }, []);

  return { models, loading, error };
}

