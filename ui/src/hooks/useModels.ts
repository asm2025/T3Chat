import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
import type { AIModel } from '@/types/model';

export function useModels() {
  const [models, setModels] = useState<AIModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const loadModels = async () => {
      try {
        setLoading(true);
        const data = await api.listModels();
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

