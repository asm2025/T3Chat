import { useCallback, useEffect, useRef, useState } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { ChatWithMessages } from "@/types/chat";

export function useChat(chatId: string | null) {
    const [chat, setChat] = useState<ChatWithMessages | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const mountedRef = useRef(true);

    useEffect(() => {
        // React StrictMode in dev will mount/unmount effects twice.
        // Ensure we always reset this ref on mount so async completions can update state.
        mountedRef.current = true;
        return () => {
            mountedRef.current = false;
        };
    }, []);

    const loadChat = useCallback(async (isBackground = false) => {
        if (!chatId) {
            if (mountedRef.current) {
                setChat(null);
                setLoading(false);
                setError(null);
            }
            return;
        }

        try {
            if (mountedRef.current && !isBackground) {
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
            if (mountedRef.current && !isBackground) {
                setLoading(false);
            }
        }
    }, [chatId]);

    useEffect(() => {
        loadChat();
    }, [loadChat]);

    return { chat, loading, error, refresh: () => loadChat(true) };
}
