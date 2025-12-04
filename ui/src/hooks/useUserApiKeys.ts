import { useCallback, useEffect, useState } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { CreateUserApiKeyRequest, UserApiKey } from "@/types/api";

export function useUserApiKeys() {
    const [keys, setKeys] = useState<UserApiKey[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    const loadKeys = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const data = await t3ChatClient.listUserApiKeys();
            setKeys(data);
        } catch (err) {
            setError(err as Error);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        loadKeys();
    }, [loadKeys]);

    const createKey = useCallback(async (data: CreateUserApiKeyRequest) => {
        const newKey = await t3ChatClient.createUserApiKey(data);
        setKeys((prev) => [...prev, newKey]);
        return newKey;
    }, []);

    const deleteKey = useCallback(async (id: string) => {
        await t3ChatClient.deleteUserApiKey(id);
        setKeys((prev) => prev.filter((key) => key.id !== id));
    }, []);

    return { keys, loading, error, createKey, deleteKey, refresh: loadKeys };
}

