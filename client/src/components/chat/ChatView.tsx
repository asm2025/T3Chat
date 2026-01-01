import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Trash2 } from "lucide-react";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { useChat } from "@/hooks/useChat";
import { useModels } from "@/hooks/useModels";
import { useStreamingChat } from "@/hooks/useStreamingChat";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import { useConfig } from "@/stores/appStore";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { useChats } from "@/hooks/useChats";
import { useStreamingContext } from "@/contexts/streaming-context";
import type { Message, AiProvider } from "@/types/chat";
import type { AIModel } from "@/types/model";

interface ChatViewProps {
    chatId?: string | null;
}

const NEW_CONVERSATION_KEY = "__new__";

export function ChatView({ chatId: chatIdProp }: ChatViewProps = {}) {
    const [searchParams, setSearchParams] = useSearchParams();
    // Use prop if provided, otherwise get from search params
    const chatId = chatIdProp !== undefined ? chatIdProp : searchParams.get("chatId");
    const { chat, loading, error, refresh } = useChat(chatId);
    const { models: backendModels } = useModels();
    const { config } = useConfig();
    const { sendMessage, streaming } = useStreamingChat();
    const { refresh: refreshChats } = useChats();
    const { setStreamingChatId } = useStreamingContext();
    const [messagesByChatId, setMessagesByChatId] = useState<Record<string, Message[]>>({
        [NEW_CONVERSATION_KEY]: [],
    });
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);
    // Track input content to control placeholder visibility
    const [inputContent, setInputContent] = useState("");
    const [showDeleteDialog, setShowDeleteDialog] = useState(false);
    const [isDeleting, setIsDeleting] = useState(false);

    const chatKey = chatId ?? NEW_CONVERSATION_KEY;
    // During the transition from no chatId to chatId, React may render once with the new `chatId`
    // before our state has been migrated to that key. Fall back to the new-chat bucket to avoid
    // the placeholder flashing back in.
    const messages = messagesByChatId[chatKey] ?? (chatId ? messagesByChatId[NEW_CONVERSATION_KEY] ?? [] : []);

    const modelSpecs = config?.modelSpecs || [];
    const useSpecs = modelSpecs.length > 0;

    const specModels: AIModel[] = useSpecs
        ? modelSpecs
              // Filter out specs with missing endpoint or model
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

    // Merge backend model catalog + modelSpecs (specs win on label/icon/etc).
    const models = useMemo(() => {
        const merged = new Map<string, AIModel>();
        for (const m of backendModels) {
            // Remove trailing "Default" for nicer labels (e.g., "GPT-4o Default" -> "GPT-4o")
            const cleaned = { ...m, displayName: m.displayName.replace(/\s+Default$/i, "") };
            merged.set(`${cleaned.provider}:${cleaned.modelId}`, cleaned);
        }
        for (const m of specModels) {
            merged.set(`${m.provider}:${m.modelId}`, m);
        }
        return Array.from(merged.values());
    }, [backendModels, specModels]);
    const modelSelectEnabled = config?.interface?.modelSelect ?? true;

    const setMessagesForChat = useCallback((key: string, updater: Message[] | ((prev: Message[]) => Message[])) => {
        setMessagesByChatId((prev) => {
            const prevForChat = prev[key] ?? [];
            const nextForChat = typeof updater === "function" ? (updater as (p: Message[]) => Message[])(prevForChat) : updater;
            return { ...prev, [key]: nextForChat };
        });
    }, []);

    // When a chat is loaded from the backend, sync its messages into the cache.
    // Important: do NOT clear messages when `chat` is temporarily null during loading,
    // otherwise the placeholder reappears after the first optimistic bubble.
    useEffect(() => {
        if (!chat || !chat.id) {
            return;
        }

        setMessagesByChatId((prev) => {
            const next = { ...prev };
            const nextMessages = chat.messages ?? [];

            // Store under the internal id (what the API returns) and also the current route param
            // (in case the user entered a URL using the external chat_id).
            next[chat.id] = nextMessages;
            if (chatId) {
                next[chatId] = nextMessages;
            }
            return next;
        });
    }, [chat, chatId]);

    // When navigating to home with no chatId, reset the new-chat state.
    useEffect(() => {
        if (chatId !== null) {
            // Clear input when navigating to an existing chat
            setInputContent("");
            return;
        }
        // Reset messages and input for new chat
        setMessagesForChat(NEW_CONVERSATION_KEY, []);
        setInputContent("");
    }, [chatId, setMessagesForChat]);

    useEffect(() => {
        if (!models.length) {
            return;
        }

        if (chat) {
            const activeModel = models.find((model) => model.provider === chat.modelProvider && model.modelId === chat.modelId);
            if (activeModel) {
                if (selectedModel?.id !== activeModel.id) {
                    setSelectedModel(activeModel);
                }
            } else {
                const isSelectedValid = selectedModel && models.some((m) => m.id === selectedModel.id);
                if (!isSelectedValid) {
                    setSelectedModel(models[0]);
                }
            }
        } else {
            // For new chats (no chatId), always try to use the configured default
            // This overrides any persisted selectedModel to ensure the admin's default is respected
            if (config?.interface?.defaultModelSpec) {
                const defaultSpec = models.find((m) => m.id === config.interface!.defaultModelSpec);
                if (defaultSpec) {
                    if (selectedModel?.id !== defaultSpec.id) {
                        setSelectedModel(defaultSpec);
                    }
                    return;
                }
            }
            if (config?.interface?.defaultProvider && config?.interface?.defaultModel) {
                const defaultModel = models.find((m) => m.provider === config.interface!.defaultProvider && m.modelId === config.interface!.defaultModel);
                if (defaultModel) {
                    if (selectedModel?.id !== defaultModel.id) {
                        setSelectedModel(defaultModel);
                    }
                    return;
                }
            }
            // Fallback to first model only if no default is configured
            if (!selectedModel) {
                setSelectedModel(models[0]);
            }
        }
    }, [chat, models, selectedModel, config]);

    // Update streaming context when streaming state or chatId changes
    useEffect(() => {
        if (streaming && chatId) {
            setStreamingChatId(chatId);
        } else {
            setStreamingChatId(null);
        }
    }, [streaming, chatId, setStreamingChatId]);

    const handleSendMessage = async (content: string) => {
        if (!selectedModel) {
            toast.error("Select a model before sending a message.");
            return;
        }

        // Validate model has required fields before proceeding
        const provider = selectedModel.provider;
        const modelId = selectedModel.modelId;

        if (!provider || typeof provider !== "string" || provider.trim() === "") {
            toast.error("Invalid model configuration", { description: "Model provider is missing or invalid. Please select a different model." });
            console.error("Invalid model provider:", { selectedModel, provider });
            return;
        }

        if (!modelId || typeof modelId !== "string" || modelId.trim() === "") {
            toast.error("Invalid model configuration", { description: "Model ID is missing or invalid. Please select a different model." });
            console.error("Invalid model_id:", { selectedModel, modelId });
            return;
        }

        // Prefer the internal chat id once loaded (it always maps to messages.chat_id).
        let currentChatId = chat?.id ?? chatId;

        const timestamp = new Date().toISOString();
        // sequenceNumber is always 0 from backend, so we don't need to calculate it
        const userTempId = `temp-user-${Date.now()}`;
        const assistantTempId = `temp-assistant-${Date.now()}`;

        const optimisticUserBase: Message = {
            id: userTempId,
            chatId: currentChatId ?? NEW_CONVERSATION_KEY,
            role: "user",
            content,
            sequenceNumber: 0, // Backend always returns 0
            createdAt: timestamp,
        };

        const optimisticAssistantBase: Message = {
            id: assistantTempId,
            chatId: currentChatId ?? NEW_CONVERSATION_KEY,
            role: "assistant",
            content: "",
            sequenceNumber: 0, // Backend always returns 0
            createdAt: new Date().toISOString(),
        };

        // If we don't have a chat id yet, show the first bubbles immediately under the new-chat key
        // so the placeholder is removed instantly.
        if (!currentChatId) {
            setInputContent(""); // Clear input when sending message
            setMessagesForChat(NEW_CONVERSATION_KEY, (prev) => [...prev, optimisticUserBase, optimisticAssistantBase]);

            // Create new chat before streaming (stream endpoint requires a chat id)
            try {
                const normalizedProvider = provider.toLowerCase() as AiProvider;

                const newChat = await t3ChatClient.createChat({
                    title: "New Chat",
                    modelProvider: normalizedProvider,
                    modelId,
                });

                if (!newChat?.id) {
                    throw new Error("Failed to create chat: No chat ID returned");
                }

                currentChatId = newChat.id;

                // Refresh chat list immediately so the new chat appears in sidebar
                refreshChats();

                // Update chat title after creation with actual content (non-blocking)
                const chatTitle = content.slice(0, 30) + (content.length > 30 ? "..." : "");
                t3ChatClient
                    .updateChat(currentChatId, { title: chatTitle })
                    .then(() => {
                        refreshChats();
                    })
                    .catch((err) => {
                        // Non-critical error, continue
                        console.error("Failed to update chat title:", err);
                    });

                // Migrate optimistic messages from the new-chat key to the real chat id.
                setMessagesByChatId((prev) => {
                    const pending = prev[NEW_CONVERSATION_KEY] ?? [];
                    const migrated = pending.map((m) => ({ ...m, chatId: currentChatId! }));
                    return {
                        ...prev,
                        // Keep NEW_CHAT_KEY messages around briefly so the UI can fall back during route transition.
                        // We clear it when navigating back to home without a chatId.
                        [NEW_CONVERSATION_KEY]: pending,
                        [currentChatId!]: migrated,
                    };
                });

                setSearchParams({ chatId: currentChatId }, { replace: true });
            } catch (err) {
                // If chat creation fails, keep the user bubble and replace assistant placeholder with an error.
                const msg = getErrorMessage(err);
                setMessagesForChat(NEW_CONVERSATION_KEY, (prev) => prev.map((m) => (m.id === assistantTempId ? { ...m, content: `Error: ${msg}` } : m)));
                toast.error("Failed to create new chat", { description: msg });
                return;
            }
        } else {
            // Existing chat: append optimistic bubbles directly under the resolved chat id.
            // Do this IMMEDIATELY so user sees their message right away
            const optimisticUser: Message = { ...optimisticUserBase, chatId: currentChatId };
            const optimisticAssistant: Message = { ...optimisticAssistantBase, chatId: currentChatId };
            setMessagesForChat(currentChatId, (prev) => [...prev, optimisticUser, optimisticAssistant]);
        }

        // Ensure we have a valid chat ID before sending
        if (!currentChatId) {
            toast.error("Failed to send message", { description: "Chat ID is missing" });
            return;
        }

        // Clear input when sending message
        setInputContent("");

        try {
            await sendMessage(
                {
                    chatId: currentChatId,
                    message: content,
                    modelProvider: provider.toLowerCase() as AiProvider,
                    modelId,
                    stream: true,
                },
                (chunk) => {
                    // Use functional update to ensure each chunk is appended immediately
                    setMessagesForChat(currentChatId!, (prev) => {
                        const assistantMsg = prev.find((m) => m.id === assistantTempId);
                        const currentContent = assistantMsg?.content || "";
                        return prev.map((message) => (message.id === assistantTempId ? { ...message, content: currentContent + chunk } : message));
                    });
                },
                async () => {
                    try {
                        // `/v1/chat/stream` persists user + assistant messages on the server.
                        // Refresh to reconcile optimistic state with DB.
                        if (chatId === currentChatId) {
                            await refresh();
                        } else {
                            // New chat navigation: `refresh` in this closure may be tied to the old chatId (null),
                            // so fetch directly and sync messages into the cache.
                            const latest = await t3ChatClient.getChat(currentChatId!);
                            setMessagesByChatId((prev) => ({
                                ...prev,
                                [currentChatId!]: latest.messages ?? prev[currentChatId!] ?? [],
                            }));
                        }
                    } catch (err) {
                        toast.error("Failed to save assistant response", { description: getErrorMessage(err) });
                    }
                },
            );
        } catch (err) {
            // Replace the empty assistant placeholder with a visible error so the UI doesn't look "stuck".
            const msg = getErrorMessage(err);
            setMessagesForChat(currentChatId!, (prev) => prev.map((m) => (m.id === assistantTempId ? { ...m, content: `Error: ${msg}` } : m)));
        }
    };

    // Only show the full-page loading state when we don't already have optimistic (or cached) messages to render.
    if (loading && messages.length === 0) {
        return <div className="flex h-full items-center justify-center rounded-xl border border-border bg-card">Loading chat…</div>;
    }

    if (error) {
        return <div className="flex h-full flex-col items-center justify-center rounded-xl border border-border bg-card p-8 text-center text-red-500">Unable to load chat. {error.message}</div>;
    }

    if (!loading && chatId && !chat) {
        return <div className="flex h-full items-center justify-center rounded-xl border border-border bg-card">Chat not found.</div>;
    }

    const handleDeleteChat = async () => {
        if (!chatId || !chat) return;

        setIsDeleting(true);
        try {
            await t3ChatClient.deleteChat(chatId);
            toast.success("Chat deleted successfully");
            // Navigate away from the deleted chat first, then refresh the chat list
            setSearchParams({}, { replace: true });
            refreshChats();
        } catch (err) {
            toast.error("Failed to delete chat", {
                description: getErrorMessage(err),
            });
        } finally {
            setIsDeleting(false);
            setShowDeleteDialog(false);
        }
    };

    // Only show chat title if we have a chat and it matches the current chatId
    const showChatTitle = chat && chat.id === chatId;

    return (
        <div className="flex h-full flex-col">
            {showChatTitle && (
                <div className="border-b border-border p-4">
                    <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                        <div>
                            <p className="text-xs uppercase tracking-wide text-muted-foreground">Chat</p>
                            <h2 className="text-lg font-semibold">{chat.title}</h2>
                        </div>
                        <Button variant="ghost" size="icon" onClick={() => setShowDeleteDialog(true)} className="text-muted-foreground hover:text-destructive">
                            <Trash2 className="h-4 w-4" />
                            <span className="sr-only">Delete chat</span>
                        </Button>
                    </div>
                </div>
            )}

            <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Delete Chat</DialogTitle>
                        <DialogDescription>Are you sure you want to delete this chat? This action cannot be undone. All messages in this chat will be permanently deleted.</DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowDeleteDialog(false)} disabled={isDeleting}>
                            Cancel
                        </Button>
                        <Button variant="destructive" onClick={handleDeleteChat} disabled={isDeleting}>
                            {isDeleting ? "Deleting..." : "Delete"}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
            <div className="flex-1 min-h-0">
                <MessageList
                    messages={messages}
                    streaming={streaming}
                    showPlaceholder={chatId === null && !streaming && messages.length === 0 && inputContent.trim().length === 0}
                    onPromptClick={(prompt) => {
                        // Set the prompt text in the input instead of sending
                        setInputContent(prompt);
                    }}
                />
            </div>
            <MessageInput
                onSend={handleSendMessage}
                disabled={streaming || !selectedModel}
                models={models}
                selectedModel={selectedModel}
                onModelSelect={setSelectedModel}
                modelSelectEnabled={modelSelectEnabled}
                value={inputContent}
                onValueChange={(value) => {
                    setInputContent(value);
                }}
                onContentChange={(_hasContent) => {
                    // This callback is kept for backward compatibility but not used for placeholder control
                }}
            />
        </div>
    );
}
