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
import type { Message } from "@/types/chat";
import type { AIModel } from "@/types/model";

interface ChatViewProps {
    chatId: string | null;
}

export function ChatView({ chatId }: ChatViewProps) {
    const navigate = useNavigate();
    const { chat, loading, error, refresh } = useChat(chatId);
    const { models, loading: modelsLoading } = useModels();
    const { sendMessage, streaming } = useStreamingChat();
    const [messages, setMessages] = useState<Message[]>([]);
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);

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
            setSelectedModel(models[0]);
        }
    }, [chat, models, selectedModel]);

    const handleSendMessage = async (content: string) => {
        if (!selectedModel) {
            toast.error("Select a model before sending a message.");
            return;
        }

        let currentChatId = chatId;

        // Create new chat if one doesn't exist
        if (!currentChatId) {
            try {
                const newChat = await t3ChatClient.createChat({
                    title: content.slice(0, 30) + (content.length > 30 ? "..." : ""),
                    model_provider: selectedModel.provider,
                    model_id: selectedModel.model_id,
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
                model_provider: selectedModel.provider,
                model_id: selectedModel.model_id,
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
        <div className="flex h-full flex-col rounded-xl border border-border bg-card">
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
            <div className="flex-1 min-h-0 bg-background">
                <MessageList 
                    messages={messages} 
                    streaming={streaming} 
                    onPromptClick={handleSendMessage}
                />
            </div>
            <MessageInput
                onSend={handleSendMessage}
                disabled={streaming || !selectedModel}
                models={models}
                selectedModel={selectedModel}
                onModelSelect={setSelectedModel}
            />
        </div>
    );
}
