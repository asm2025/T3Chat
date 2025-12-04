import { useEffect, useState } from "react";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { ModelSelector } from "@/components/model/ModelSelector";
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
        if (!chatId) {
            toast.error("Select or create a chat to start messaging.");
            return;
        }

        if (!selectedModel) {
            toast.error("Select a model before sending a message.");
            return;
        }

        const timestamp = new Date().toISOString();
        const nextSequence = (messages[messages.length - 1]?.sequence_number ?? 0) + 1;
        const userTempId = `temp-user-${Date.now()}`;
        const assistantTempId = `temp-assistant-${Date.now()}`;

        const optimisticUser: Message = {
            id: userTempId,
            chat_id: chatId,
            role: "user",
            content,
            sequence_number: nextSequence,
            created_at: timestamp,
        };

        setMessages((prev) => [...prev, optimisticUser]);

        try {
            await t3ChatClient.createMessage(chatId, { content, role: "user" });
        } catch (err) {
            setMessages((prev) => prev.filter((message) => message.id !== userTempId));
            toast.error("Failed to send message", { description: getErrorMessage(err) });
            return;
        }

        let assistantContent = "";
        const optimisticAssistant: Message = {
            id: assistantTempId,
            chat_id: chatId,
            role: "assistant",
            content: "",
            sequence_number: nextSequence + 1,
            created_at: new Date().toISOString(),
        };

        setMessages((prev) => [...prev, optimisticAssistant]);

        await sendMessage(
            {
                chat_id: chatId,
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
                    await t3ChatClient.createMessage(chatId, { content: assistantContent, role: "assistant" });
                    await refresh();
                } catch (err) {
                    toast.error("Failed to save assistant response", { description: getErrorMessage(err) });
                }
            },
        );
    };

    if (!chatId) {
        return <div className="flex h-full flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card p-8 text-center text-muted-foreground">Select a chat from the list to get started.</div>;
    }

    if (loading) {
        return <div className="flex h-full items-center justify-center rounded-xl border border-border bg-card">Loading chat…</div>;
    }

    if (error) {
        return <div className="flex h-full flex-col items-center justify-center rounded-xl border border-border bg-card p-8 text-center text-red-500">Unable to load chat. {error.message}</div>;
    }

    if (!chat) {
        return <div className="flex h-full items-center justify-center rounded-xl border border-border bg-card">Chat not found.</div>;
    }

    return (
        <div className="flex h-full flex-col rounded-xl border border-border bg-card">
            <div className="border-b border-border p-4">
                <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                    <div>
                        <p className="text-xs uppercase tracking-wide text-muted-foreground">Chat</p>
                        <h2 className="text-lg font-semibold">{chat.title}</h2>
                    </div>
                    <div className="w-full lg:max-w-xs">
                        <ModelSelector models={models} selectedModel={selectedModel} onSelect={setSelectedModel} />
                        {modelsLoading && <p className="mt-1 text-xs text-muted-foreground">Loading models…</p>}
                    </div>
                </div>
            </div>
            <div className="flex-1 min-h-0 bg-background">
                <MessageList messages={messages} streaming={streaming} />
            </div>
            <MessageInput onSend={handleSendMessage} disabled={streaming || !selectedModel} />
        </div>
    );
}
