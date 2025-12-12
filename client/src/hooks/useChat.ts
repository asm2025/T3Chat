import { useCallback, useEffect, useRef, useState } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { ChatWithMessages } from "@/types/chat";

export function useChat(chatId: string | null) {
    const [chat, setChat] = useState<ChatWithMessages | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const mountedRef = useRef(true);

    useEffect(() => {
        return () => {
            mountedRef.current = false;
        };
    }, []);

    const loadChat = useCallback(async () => {
        if (!chatId) {
            if (mountedRef.current) {
                setChat(null);
                setLoading(false);
                setError(null);
            }
            return;
        }

        try {
            if (mountedRef.current) {
                setLoading(true);
                setError(null);
            }
            const data = await t3ChatClient.getChat(chatId);
            if (mountedRef.current) {
                setChat(data);
            }
        } catch (err) {
            if (mountedRef.current) {
                setError(err as Error);
            }
        } finally {
            if (mountedRef.current) {
                setLoading(false);
            }
        }
    }, [chatId]);

    useEffect(() => {
        loadChat();
    }, [loadChat]);

    return { chat, loading, error, refresh: loadChat };
}

