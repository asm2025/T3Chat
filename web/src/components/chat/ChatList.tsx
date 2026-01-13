import { useMemo, useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Plus, Search, MoreVertical } from "lucide-react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { useChats } from "@/hooks/useChats";
import { useModels } from "@/hooks/useModels";
import { useConfig } from "@/stores/appStore";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import { useStreamingContext } from "@/contexts/streaming-context";
import { RadialProgress } from "@/components/ui/radial-progress";
import type { Chat, AiProvider } from "@/types/chat";
import type { AIModel } from "@/types/model";

export function ChatList() {
    const { chats, loading, refresh } = useChats();
    const { models: backendModels } = useModels();
    const { config } = useConfig();
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const chatId = searchParams.get("chatId");
    const { streamingChatId } = useStreamingContext();
    const [query, setQuery] = useState("");
    const [deleteDialogChatId, setDeleteDialogChatId] = useState<string | null>(null);
    const [isDeleting, setIsDeleting] = useState(false);
    const [isCreatingChat, setIsCreatingChat] = useState(false);

    // Refresh chat list when chatId changes (e.g., when a new chat is created)
    useEffect(() => {
        refresh();
    }, [chatId, refresh]);

    const handleDeleteChat = async () => {
        if (!deleteDialogChatId) return;

        const isActiveChat = deleteDialogChatId === chatId;
        setIsDeleting(true);
        try {
            await t3ChatClient.deleteChat(deleteDialogChatId);
            toast.success("Chat deleted successfully");

            // If the deleted chat was the active one, navigate away first
            if (isActiveChat) {
                navigate("/");
            }

            // Then refresh the chat list
            refresh();
        } catch (err) {
            toast.error("Failed to delete chat", {
                description: getErrorMessage(err),
            });
        } finally {
            setIsDeleting(false);
            setDeleteDialogChatId(null);
        }
    };

    const filteredChats = useMemo(() => {
        if (!query.trim()) {
            return chats;
        }
        return chats.filter((chat) => chat.title.toLowerCase().includes(query.toLowerCase()));
    }, [chats, query]);

    // Format date for display
    const formatDateHeader = (dateString: string): string => {
        const date = new Date(dateString);
        const now = new Date();
        const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
        const chatDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());

        const diffTime = today.getTime() - chatDate.getTime();
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

        if (diffDays === 0) {
            return "Today";
        } else if (diffDays === 1) {
            return "Yesterday";
        } else if (diffDays < 7) {
            return `${diffDays} days ago`;
        } else {
            // Format as "Month abbreviation, day year" (e.g., "Dec 26, 2025")
            return date.toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
            });
        }
    };

    // Get date key for grouping (YYYY-MM-DD format)
    const getDateKey = (dateString: string): string => {
        const date = new Date(dateString);
        return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
    };

    // Group chats by date
    const groupedChats = useMemo(() => {
        const groups: Record<string, Chat[]> = {};

        filteredChats.forEach((chat) => {
            const dateKey = getDateKey(chat.updatedAt);
            if (!groups[dateKey]) {
                groups[dateKey] = [];
            }
            groups[dateKey].push(chat);
        });

        // Sort chats within each group by updatedAt (most recent first)
        Object.keys(groups).forEach((key) => {
            groups[key].sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime());
        });

        // Sort groups by date (most recent first)
        const sortedGroups = Object.entries(groups).sort(([keyA], [keyB]) => {
            return keyB.localeCompare(keyA);
        });

        return sortedGroups.map(([dateKey, chats]) => ({
            dateKey,
            dateLabel: formatDateHeader(chats[0].updatedAt),
            chats,
        }));
    }, [filteredChats]);

    // Get available models (similar to ChatView logic)
    const availableModels = useMemo(() => {
        const modelSpecs = config?.modelSpecs || [];
        const useSpecs = modelSpecs.length > 0;

        const specModels: AIModel[] = useSpecs
            ? modelSpecs
                  .filter((spec) => spec.preset?.endpoint && spec.preset?.model)
                  .map((spec) => ({
                      id: spec.name,
                      provider: spec.preset.endpoint.toLowerCase() as AiProvider,
                      modelId: spec.preset.model,
                      displayName: spec.label,
                      contextWindow: 128000,
                      supportsStreaming: true,
                      supportsImages: false,
                      supportsFunctions: false,
                      isActive: true,
                      createdAt: "",
                      updatedAt: "",
                  }))
            : [];

        // Merge backend models + specs
        const merged = new Map<string, AIModel>();
        for (const m of backendModels) {
            const cleaned = { ...m, displayName: m.displayName.replace(/\s+Default$/i, "") };
            merged.set(`${cleaned.provider}:${cleaned.modelId}`, cleaned);
        }
        for (const m of specModels) {
            merged.set(`${m.provider}:${m.modelId}`, m);
        }
        return Array.from(merged.values());
    }, [backendModels, config]);

    // Determine default model (similar to ChatView logic)
    const getDefaultModel = (): AIModel | null => {
        if (availableModels.length === 0) {
            return null;
        }

        // Check for defaultModelSpec first
        if (config?.interface?.defaultModelSpec) {
            const defaultSpec = availableModels.find((m) => m.id === config.interface!.defaultModelSpec);
            if (defaultSpec) {
                return defaultSpec;
            }
        }

        // Check for defaultProvider + defaultModel
        if (config?.interface?.defaultProvider && config?.interface?.defaultModel) {
            const defaultModel = availableModels.find(
                (m) => m.provider === config.interface!.defaultProvider && m.modelId === config.interface!.defaultModel
            );
            if (defaultModel) {
                return defaultModel;
            }
        }

        // Fallback to first available model
        return availableModels[0];
    };

    const handleNewChat = async () => {
        if (isCreatingChat) {
            return;
        }

        const selectedModel = getDefaultModel();
        if (!selectedModel) {
            toast.error("No models available", {
                description: "Please configure at least one model before creating a chat.",
            });
            return;
        }

        setIsCreatingChat(true);
        try {
            const normalizedProvider = selectedModel.provider.toLowerCase() as AiProvider;

            const newChat = await t3ChatClient.createChat({
                title: "New Chat",
                modelProvider: normalizedProvider,
                modelId: selectedModel.modelId,
            });

            if (!newChat?.id) {
                throw new Error("Failed to create chat: No chat ID returned");
            }

            // Refresh chat list to show the new chat
            refresh();

            // Navigate to the new chat
            navigate(`/?chatId=${newChat.id}`);
        } catch (err) {
            toast.error("Failed to create new chat", {
                description: getErrorMessage(err),
            });
        } finally {
            setIsCreatingChat(false);
        }
    };

    if (loading) {
        return <div className="p-4 text-sm text-muted-foreground">Loading chats…</div>;
    }

    return (
        <div className="flex h-full flex-col gap-2 py-2">
            <div className="px-3">
                <Button onClick={handleNewChat} className="w-full justify-start gap-2" size="default" disabled={isCreatingChat || availableModels.length === 0}>
                    <Plus className="h-4 w-4" />
                    <span>{isCreatingChat ? "Creating..." : "New Chat"}</span>
                </Button>
            </div>

            <div className="px-3">
                <div className="relative">
                    <Search className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search threads..." className="h-9 rounded-md bg-muted/50 pl-9 text-sm shadow-none focus-visible:ring-1" />
                </div>
            </div>

            <div className="flex-1 overflow-y-auto px-2">
                {groupedChats.length === 0 ? (
                    <div className="px-2 py-8 text-center text-xs text-muted-foreground">{query ? "No chats match your search." : "No chats yet."}</div>
                ) : (
                    <div className="space-y-4">
                        {groupedChats.map((group) => (
                            <div key={group.dateKey} className="space-y-1">
                                <div className="px-3 py-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wide">{group.dateLabel}</div>
                                {group.chats.map((chat) => {
                                    const active = chat.id === chatId;
                                    const isStreaming = chat.id === streamingChatId;
                                    const displayTitle = chat.title || "New Chat";
                                    return (
                                        <div key={chat.id} className="group relative flex items-center">
                                            <button
                                                onClick={() => navigate(`/?chatId=${chat.id}`)}
                                                className={`flex flex-1 items-center gap-2 px-3 py-1.5 text-left text-sm transition-colors ${active ? "text-foreground font-medium" : "text-muted-foreground hover:text-foreground"}`}>
                                                <div className="flex-1 truncate">{displayTitle}</div>
                                                {isStreaming && <RadialProgress size={16} className="text-muted-foreground" />}
                                            </button>
                                            <DropdownMenu>
                                                <DropdownMenuTrigger asChild>
                                                    <Button variant="ghost" size="icon" className="h-8 w-8 opacity-0 transition-opacity group-hover:opacity-100" onClick={(e) => e.stopPropagation()}>
                                                        <MoreVertical className="h-4 w-4" />
                                                        <span className="sr-only">Open menu</span>
                                                    </Button>
                                                </DropdownMenuTrigger>
                                                <DropdownMenuContent align="end">
                                                    <DropdownMenuItem
                                                        onClick={(e) => {
                                                            e.stopPropagation();
                                                            setDeleteDialogChatId(chat.id);
                                                        }}>
                                                        Delete...
                                                    </DropdownMenuItem>
                                                </DropdownMenuContent>
                                            </DropdownMenu>
                                        </div>
                                    );
                                })}
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <Dialog open={deleteDialogChatId !== null} onOpenChange={(open) => !open && setDeleteDialogChatId(null)}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Delete Chat</DialogTitle>
                        <DialogDescription>Are you sure you want to delete this chat? This action cannot be undone. All messages in this chat will be permanently deleted.</DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setDeleteDialogChatId(null)} disabled={isDeleting}>
                            Cancel
                        </Button>
                        <Button onClick={handleDeleteChat} disabled={isDeleting}>
                            {isDeleting ? "Deleting..." : "Delete"}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
}
