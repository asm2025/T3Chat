import { useEffect, useState } from "react";
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

export function ChatView({ chatId }: ChatViewProps) {
    const navigate = useNavigate();
    const { chat, loading, error, refresh } = useChat(chatId);
    const { models: backendModels } = useModels();
    const { config } = useConfig();
    const { sendMessage, streaming } = useStreamingChat();
    const [messages, setMessages] = useState<Message[]>([]);
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);

    const modelSpecs = config?.modelSpecs || [];
    const useSpecs = modelSpecs.length > 0;

    const effectiveModels: AIModel[] = useSpecs
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
        : backendModels;

    const models = effectiveModels;
    const modelSelectEnabled = config?.interface?.modelSelect ?? true;

    useEffect(() => {
        setMessages(chat?.messages ?? []);
    }, [chat]);

    useEffect(() => {
        if (!models.length) {
            return;
        }

        if (chat) {
            const activeModel = models.find((model) => model.provider === chat.model_provider && model.model_id === chat.model_id);
            setSelectedModel(activeModel ?? models[0]);
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

        let currentChatId = chatId;

        // Create new chat if one doesn't exist
        if (!currentChatId) {
            try {
                const normalizedProvider = provider.toLowerCase() as AiProvider;
                
                const newChat = await t3ChatClient.createChat({
                    title: content.slice(0, 30) + (content.length > 30 ? "..." : ""),
                    model_provider: normalizedProvider,
                    model_id: modelId,
                });
                currentChatId = newChat.id;
                navigate(`/chat/${currentChatId}`, { replace: true });
            } catch (err) {
                toast.error("Failed to create new chat", { description: getErrorMessage(err) });
                return;
            }
        }

        const timestamp = new Date().toISOString();
        const nextSequence = (messages[messages.length - 1]?.sequence_number ?? 0) + 1;
        const userTempId = `temp-user-${Date.now()}`;
        const assistantTempId = `temp-assistant-${Date.now()}`;

        const optimisticUser: Message = {
            id: userTempId,
            chat_id: currentChatId,
            role: "user",
            content,
            sequence_number: nextSequence,
            created_at: timestamp,
        };

        setMessages((prev) => [...prev, optimisticUser]);

        try {
            console.log("Creating message", currentChatId, content);
            await t3ChatClient.createMessage(currentChatId, { content, role: "user" });
        } catch (err) {
            setMessages((prev) => prev.filter((message) => message.id !== userTempId));
            toast.error("Failed to send message", { description: getErrorMessage(err) });
            return;
        }

        let assistantContent = "";
        const optimisticAssistant: Message = {
            id: assistantTempId,
            chat_id: currentChatId,
            role: "assistant",
            content: "",
            sequence_number: nextSequence + 1,
            created_at: new Date().toISOString(),
        };

        setMessages((prev) => [...prev, optimisticAssistant]);

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
                setMessages((prev) => prev.map((message) => (message.id === assistantTempId ? { ...message, content: assistantContent } : message)));
            },
            async () => {
                try {
                    await t3ChatClient.createMessage(currentChatId, { content: assistantContent, role: "assistant" });
                    // Only refresh if we didn't just create the chat (since we navigated)
                    if (chatId === currentChatId) {
                        await refresh();
                    }
                } catch (err) {
                    toast.error("Failed to save assistant response", { description: getErrorMessage(err) });
                }
            },
        );
    };

    if (loading) {
        return <div className="flex h-full items-center justify-center rounded-xl border border-border bg-card">Loading chat…</div>;
    }

    if (error) {
        return <div className="flex h-full flex-col items-center justify-center rounded-xl border border-border bg-card p-8 text-center text-red-500">Unable to load chat. {error.message}</div>;
    }

    if (chatId && !chat) {
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
