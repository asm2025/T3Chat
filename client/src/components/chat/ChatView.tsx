import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { useChat } from "@/hooks/useChat";
import { useModels } from "@/hooks/useModels";
import { useStreamingChat } from "@/hooks/useStreamingChat";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import { useConfig } from "@/stores/appStore";
import type { Message, AiProvider } from "@/types/chat";
import type { AIModel } from "@/types/model";

interface ChatViewProps {
    chatId: string | null;
}

const NEW_CHAT_KEY = "__new__";

export function ChatView({ chatId }: ChatViewProps) {
    const navigate = useNavigate();
    const { chat, loading, error, refresh } = useChat(chatId);
    const { models: backendModels } = useModels();
    const { config } = useConfig();
    const { sendMessage, streaming } = useStreamingChat();
    const [messagesByChatId, setMessagesByChatId] = useState<Record<string, Message[]>>({
        [NEW_CHAT_KEY]: [],
    });
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);

    const chatKey = chatId ?? NEW_CHAT_KEY;
    // During the "/chat -> /chat/{id}" transition, React may render once with the new `chatId`
    // before our state has been migrated to that key. Fall back to the new-chat bucket to avoid
    // the placeholder flashing back in.
    const messages = messagesByChatId[chatKey] ?? (chatId ? messagesByChatId[NEW_CHAT_KEY] ?? [] : []);

    const modelSpecs = config?.modelSpecs || [];
    const useSpecs = modelSpecs.length > 0;

    const specModels: AIModel[] = useSpecs
        ? modelSpecs
              // Filter out specs with missing endpoint or model
              .filter((spec) => spec.preset?.endpoint && spec.preset?.model)
              .map((spec) => ({
                  id: spec.name,
                  provider: spec.preset.endpoint.toLowerCase() as AiProvider,
                  model_id: spec.preset.model,
                  display_name: spec.label,
                  context_window: 128000,
                  supports_streaming: true,
                  supports_images: false,
                  supports_functions: false,
                  is_active: true,
                  created_at: "",
                  updated_at: "",
              }))
        : [];

    // Merge backend model catalog + modelSpecs (specs win on label/icon/etc).
    const models = useMemo(() => {
        const merged = new Map<string, AIModel>();
        for (const m of backendModels) {
            // Remove trailing "Default" for nicer labels (e.g., "GPT-4o Default" -> "GPT-4o")
            const cleaned = { ...m, display_name: m.display_name.replace(/\s+Default$/i, "") };
            merged.set(`${cleaned.provider}:${cleaned.model_id}`, cleaned);
        }
        for (const m of specModels) {
            merged.set(`${m.provider}:${m.model_id}`, m);
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
        if (!chat) {
            return;
        }

        setMessagesByChatId((prev) => {
            const next = { ...prev };
            const nextMessages = chat.messages ?? [];

            // Store under the internal id (what the API returns) and also the current route param
            // (in case the user entered a URL using the external conversation_id).
            next[chat.id] = nextMessages;
            if (chatId) {
                next[chatId] = nextMessages;
            }
            return next;
        });
    }, [chat, chatId]);

    // When navigating to /chat (no id), reset the new-chat placeholder state.
    useEffect(() => {
        if (chatId !== null) {
            return;
        }
        setMessagesForChat(NEW_CHAT_KEY, []);
    }, [chatId, setMessagesForChat]);

    useEffect(() => {
        if (!models.length) {
            return;
        }

        if (chat) {
            const activeModel = models.find((model) => model.provider === chat.model_provider && model.model_id === chat.model_id);
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
        } else if (!selectedModel) {
            if (config?.interface?.defaultModelSpec) {
                const defaultSpec = models.find((m) => m.id === config.interface!.defaultModelSpec);
                if (defaultSpec) {
                    setSelectedModel(defaultSpec);
                    return;
                }
            }
            if (config?.interface?.defaultProvider && config?.interface?.defaultModel) {
                const defaultModel = models.find((m) => m.provider === config.interface!.defaultProvider && m.model_id === config.interface!.defaultModel);
                if (defaultModel) {
                    setSelectedModel(defaultModel);
                    return;
                }
            }
            setSelectedModel(models[0]);
        }
    }, [chat, models, selectedModel, config]);

    const handleSendMessage = async (content: string) => {
        if (!selectedModel) {
            toast.error("Select a model before sending a message.");
            return;
        }

        // Validate model has required fields before proceeding
        const provider = selectedModel.provider;
        const modelId = selectedModel.model_id;

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

        // Prefer the internal chat id once loaded (it always maps to messages.conversation_id).
        let currentChatId = chat?.id ?? chatId;

        const timestamp = new Date().toISOString();
        const nextSequence = (messages[messages.length - 1]?.sequence_number ?? 0) + 1;
        const userTempId = `temp-user-${Date.now()}`;
        const assistantTempId = `temp-assistant-${Date.now()}`;

        const optimisticUserBase: Message = {
            id: userTempId,
            chat_id: currentChatId ?? NEW_CHAT_KEY,
            role: "user",
            content,
            sequence_number: nextSequence,
            created_at: timestamp,
        };

        let assistantContent = "";
        const optimisticAssistantBase: Message = {
            id: assistantTempId,
            chat_id: currentChatId ?? NEW_CHAT_KEY,
            role: "assistant",
            content: "",
            sequence_number: nextSequence + 1,
            created_at: new Date().toISOString(),
        };

        // If we don't have a chat id yet, show the first bubbles immediately under the new-chat key
        // so the placeholder is removed instantly.
        if (!currentChatId) {
            setMessagesForChat(NEW_CHAT_KEY, (prev) => [...prev, optimisticUserBase, optimisticAssistantBase]);

            // Create new chat before streaming (stream endpoint requires a chat id)
            try {
                const normalizedProvider = provider.toLowerCase() as AiProvider;

                const newChat = await t3ChatClient.createChat({
                    title: content.slice(0, 30) + (content.length > 30 ? "..." : ""),
                    model_provider: normalizedProvider,
                    model_id: modelId,
                });
                currentChatId = newChat.id;

                // Migrate optimistic messages from the new-chat key to the real chat id.
                setMessagesByChatId((prev) => {
                    const pending = prev[NEW_CHAT_KEY] ?? [];
                    const migrated = pending.map((m) => ({ ...m, chat_id: currentChatId! }));
                    return {
                        ...prev,
                        // Keep NEW_CHAT_KEY messages around briefly so the UI can fall back during route transition.
                        // We clear it when navigating back to "/chat".
                        [NEW_CHAT_KEY]: pending,
                        [currentChatId!]: migrated,
                    };
                });

                navigate(`/chat/${currentChatId}`, { replace: true });
            } catch (err) {
                // If chat creation fails, keep the user bubble and replace assistant placeholder with an error.
                const msg = getErrorMessage(err);
                setMessagesForChat(NEW_CHAT_KEY, (prev) => prev.map((m) => (m.id === assistantTempId ? { ...m, content: `Error: ${msg}` } : m)));
                toast.error("Failed to create new chat", { description: msg });
                return;
            }
        } else {
            // Existing chat: append optimistic bubbles directly under the resolved chat id.
            const optimisticUser: Message = { ...optimisticUserBase, chat_id: currentChatId };
            const optimisticAssistant: Message = { ...optimisticAssistantBase, chat_id: currentChatId };
            setMessagesForChat(currentChatId, (prev) => [...prev, optimisticUser, optimisticAssistant]);
        }

        try {
            await sendMessage(
                {
                    chat_id: currentChatId,
                    message: content,
                    model_provider: provider.toLowerCase() as AiProvider,
                    model_id: modelId,
                    stream: true,
                },
                (chunk) => {
                    assistantContent += chunk;
                    setMessagesForChat(currentChatId!, (prev) => prev.map((message) => (message.id === assistantTempId ? { ...message, content: assistantContent } : message)));
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

    return (
        <div className="flex h-full flex-col">
            {chat && (
                <div className="border-b border-border p-4">
                    <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                        <div>
                            <p className="text-xs uppercase tracking-wide text-muted-foreground">Chat</p>
                            <h2 className="text-lg font-semibold">{chat.title}</h2>
                        </div>
                    </div>
                </div>
            )}
            <div className="flex-1 min-h-0">
                <MessageList messages={messages} streaming={streaming} onPromptClick={handleSendMessage} />
            </div>
            <MessageInput onSend={handleSendMessage} disabled={streaming || !selectedModel} models={models} selectedModel={selectedModel} onModelSelect={setSelectedModel} modelSelectEnabled={modelSelectEnabled} />
        </div>
    );
}
