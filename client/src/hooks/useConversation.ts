import { useCallback, useEffect, useRef, useState } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { ConversationWithMessages } from "@/types/conversation";

export function useConversation(conversationId: string | null) {
    const [conversation, setConversation] = useState<ConversationWithMessages | null>(null);
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

    const loadConversation = useCallback(async () => {
        if (!conversationId) {
            if (mountedRef.current) {
                setConversation(null);
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
            const data = await t3ChatClient.getChat(conversationId);
            if (mountedRef.current) {
                setConversation(data);
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
    }, [conversationId]);

    useEffect(() => {
        loadConversation();
    }, [loadConversation]);

    return { conversation, loading, error, refresh: loadConversation };
}
