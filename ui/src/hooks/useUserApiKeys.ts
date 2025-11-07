import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
import type { UserApiKey, CreateUserApiKeyRequest } from '@/types/api';

export function useUserApiKeys() {
  const [keys, setKeys] = useState<UserApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const loadKeys = async () => {
    try {
      setLoading(true);
      const data = await api.listUserApiKeys();
      setKeys(data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadKeys();
  }, []);

  const createKey = async (data: CreateUserApiKeyRequest) => {
    const newKey = await api.createUserApiKey(data);
    setKeys([...keys, newKey]);
    return newKey;
  };

  const deleteKey = async (id: string) => {
    await api.deleteUserApiKey(id);
    setKeys(keys.filter(k => k.id !== id));
  };

  return { keys, loading, error, createKey, deleteKey, refresh: loadKeys };
}

