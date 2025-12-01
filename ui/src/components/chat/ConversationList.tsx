// ============================================================================
// ConversationList Component - Phase 1B
// ============================================================================
// Sidebar list of conversations

import { useEffect, useState } from "react";
import { useLibreChatConversations, useLibreChatCurrentConversation } from "@/stores/appStore";
import { ConversationItem } from "./ConversationItem";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { PlusIcon, SearchIcon } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { librechatClient } from "@/lib/librechat-client";
import { toast } from "sonner";
import type { Endpoint } from "@/types/librechat";

// ============================================================================
// Props
// ============================================================================

export interface ConversationListProps {
    onNewChat?: () => void;
    className?: string;
}

// ============================================================================
// Component
// ============================================================================

export function ConversationList({ onNewChat, className }: ConversationListProps) {
    const { conversations, loading, error, fetchConversations, addConversation, removeConversation } = useLibreChatConversations();
    const { currentConversation, setCurrentConversation } = useLibreChatCurrentConversation();
    const [searchQuery, setSearchQuery] = useState("");
    const [creating, setCreating] = useState(false);

    useEffect(() => {
        fetchConversations({ isArchived: false });
    }, [fetchConversations]);

    const handleNewChat = async () => {
        setCreating(true);
        try {
            const newConversation = await librechatClient.conversations.create({
                title: "New Chat",
                endpoint: "openai" as Endpoint,
                model: "gpt-4-turbo",
            });
            addConversation(newConversation);
            setCurrentConversation(newConversation);
            onNewChat?.();
            toast.success("New conversation created");
        } catch (error) {
            console.error("Error creating conversation:", error);
            toast.error("Failed to create conversation");
        } finally {
            setCreating(false);
        }
    };

    const handleSelectConversation = async (conversationId: string) => {
        try {
            const conversation = await librechatClient.conversations.get(conversationId);
            // TODO: Load and set messages in store
            // const messages = await librechatClient.messages.list(conversationId);
            setCurrentConversation({ ...conversation });
        } catch (error) {
            console.error("Error loading conversation:", error);
            toast.error("Failed to load conversation");
        }
    };

    const handleDeleteConversation = async (conversationId: string) => {
        try {
            await librechatClient.conversations.delete(conversationId);
            removeConversation(conversationId);
            if (currentConversation?.id === conversationId) {
                setCurrentConversation(null);
            }
            toast.success("Conversation deleted");
        } catch (error) {
            console.error("Error deleting conversation:", error);
            toast.error("Failed to delete conversation");
        }
    };

    const handleArchiveConversation = async (conversationId: string) => {
        try {
            await librechatClient.conversations.archive(conversationId, true);
            removeConversation(conversationId);
            if (currentConversation?.id === conversationId) {
                setCurrentConversation(null);
            }
            toast.success("Conversation archived");
        } catch (error) {
            console.error("Error archiving conversation:", error);
            toast.error("Failed to archive conversation");
        }
    };

    // Filter conversations based on search
    const filteredConversations = conversations.filter((conv) =>
        conv.title.toLowerCase().includes(searchQuery.toLowerCase()),
    );

    if (loading && conversations.length === 0) {
        return (
            <div className="space-y-2 p-4">
                <Skeleton className="h-10 w-full" />
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
            </div>
        );
    }

    return (
        <div className={`flex h-full flex-col ${className}`}>
            {/* Header with New Chat button */}
            <div className="border-b p-4 space-y-2">
                <Button onClick={handleNewChat} className="w-full" disabled={creating}>
                    <PlusIcon className="mr-2 h-4 w-4" />
                    {creating ? "Creating..." : "New Chat"}
                </Button>

                {/* Search */}
                <div className="relative">
                    <SearchIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                        placeholder="Search conversations..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="pl-9"
                    />
                </div>
            </div>

            {/* Error State */}
            {error && (
                <div className="p-4 text-center text-sm text-destructive">
                    <p>Failed to load conversations</p>
                    <p className="text-xs text-muted-foreground">{error.message}</p>
                </div>
            )}

            {/* Conversations List */}
            <div className="flex-1 overflow-y-auto">
                {filteredConversations.length === 0 ? (
                    <div className="p-8 text-center text-sm text-muted-foreground">
                        {searchQuery ? (
                            <>
                                <p>No conversations found</p>
                                <p className="mt-1 text-xs">Try a different search term</p>
                            </>
                        ) : (
                            <>
                                <p>No conversations yet</p>
                                <p className="mt-1 text-xs">Click "New Chat" to get started</p>
                            </>
                        )}
                    </div>
                ) : (
                    <div className="space-y-1 p-2">
                        {filteredConversations.map((conversation) => (
                            <ConversationItem
                                key={conversation.id}
                                conversation={conversation}
                                isActive={currentConversation?.id === conversation.id}
                                onClick={() => handleSelectConversation(conversation.id)}
                                onDelete={() => handleDeleteConversation(conversation.id)}
                                onArchive={() => handleArchiveConversation(conversation.id)}
                            />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

