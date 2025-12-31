import { useEffect, useCallback } from "react";
import { useChats as useChatsStore } from "@/stores/appStore";
import type { Chat } from "@/types/chat";
import type { AiProvider } from "@/types/chat";

export function useChats(page = 1, pageSize = 20) {
    const { chats, loading, error, fetchChats } = useChatsStore();

    // Load chats on mount and when page/pageSize changes
    useEffect(() => {
        fetchChats({ page, pageSize });
    }, [page, pageSize, fetchChats]);

    // Map ChatWithTags[] to Chat[] for compatibility
    // ChatWithTags extends LibreChat Chat, but we need the simpler Chat type
    // The endpoint field contains the provider string value (cast as Endpoint, but actually any AiProvider)
    const mappedChats: Chat[] = chats.map((chat) => ({
        id: chat.id,
        userId: chat.userId,
        title: chat.title,
        modelProvider: chat.endpoint as AiProvider, // endpoint contains provider value, cast to AiProvider
        modelId: chat.model,
        createdAt: chat.createdAt,
        updatedAt: chat.updatedAt,
        deletedAt: undefined,
    }));

    // Create a stable refresh function
    const refresh = useCallback(() => {
        fetchChats({ page, pageSize });
    }, [fetchChats, page, pageSize]);

    return {
        chats: mappedChats,
        total: chats.length, // Approximate total, could be improved with pagination info
        loading,
        error,
        refresh,
    };
}
