import { useCallback, useEffect, useState } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { Conversation } from "@/types/conversation";

export function useConversations(page = 1, pageSize = 20) {
    const [chats, setChats] = useState<Conversation[]>([]);
    const [total, setTotal] = useState(0);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    const loadChats = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await t3ChatClient.listChats(page, pageSize);
            setChats(result.data);
            setTotal(result.total);
        } catch (err) {
            setError(err as Error);
        } finally {
            setLoading(false);
        }
    }, [page, pageSize]);

    useEffect(() => {
        loadChats();
    }, [loadChats]);

    return { chats, total, loading, error, refresh: loadChats };
}
