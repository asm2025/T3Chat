import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth-context";
import { useChat } from "@/hooks/useChat";
import { useStreamingChat } from "@/hooks/useStreamingChat";
import { MessageList } from "@/components/chat/MessageList";
import { MessageInput } from "@/components/chat/MessageInput";
import { ChatPlaceholder } from "@/components/chat/ChatPlaceholder";
import { MasterLayout } from "@/components/MasterLayout";
import { useModels } from "@/hooks/useModels";
import * as api from "@/lib/serverComm";
import type { AIModel } from "@/types/model";
import type { Message } from "@/types/chat";

export function Chat() {
    const { chatId } = useParams<{ chatId?: string }>();
    const { chat, loading, error } = useChat(chatId || null);
    const { models } = useModels();
    const { user, userProfile } = useAuth();
    const navigate = useNavigate();
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);
    const [messages, setMessages] = useState<Message[]>([]);
    const [webSearchEnabled, setWebSearchEnabled] = useState(false);
    const { sendMessage, streaming } = useStreamingChat();

    // Update messages when chat loads
    useEffect(() => {
        if (chat) {
            setMessages(chat.messages || []);
        } else {
            setMessages([]);
        }
    }, [chat]);

    // Set selected model based on chat or default
    useEffect(() => {
        if (models.length === 0) {
            return;
        }

        if (chat) {
            const match = models.find((m) => m.provider === chat.model_provider && m.model_id === chat.model_id);
            const nextModel = match ?? models[0];
            if (!selectedModel || selectedModel.id !== nextModel.id) {
                setSelectedModel(nextModel);
            }
            return;
        }

        if (!selectedModel) {
            setSelectedModel(models[0]);
        }
    }, [chat, models, selectedModel]);

    const handleSendMessage = async (content: string) => {
        let currentChatId = chatId;

        // If no chat exists, create one
        if (!currentChatId && selectedModel) {
            try {
                const newChat = await api.createChat({
                    model_provider: selectedModel.provider,
                    model_id: selectedModel.model_id,
                });
                currentChatId = newChat.id;
                navigate(`/${newChat.id}`);
            } catch (err) {
                console.error("Failed to create chat:", err);
                return;
            }
        }

        if (!currentChatId || !selectedModel) return;

        try {
            // Add user message optimistically
            const userMessage: Message = {
                id: `temp-${Date.now()}`,
                chat_id: currentChatId,
                role: "user",
                content,
                sequence_number: messages.length,
                created_at: new Date().toISOString(),
            };
            setMessages((prev) => [...prev, userMessage]);

            // Create user message on server
            await api.createMessage(currentChatId, { content, role: "user" });

            // Create assistant message placeholder
            const assistantMessageId = `temp-assistant-${Date.now()}`;
            let assistantContent = "";
            const assistantMessage: Message = {
                id: assistantMessageId,
                chat_id: currentChatId,
                role: "assistant",
                content: "",
                sequence_number: messages.length + 1,
                created_at: new Date().toISOString(),
            };
            setMessages((prev) => [...prev, assistantMessage]);

            // Stream assistant response
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
                    // Update assistant message in real-time
                    setMessages((prev) => prev.map((msg) => (msg.id === assistantMessageId ? { ...msg, content: assistantContent } : msg)));
                },
                async () => {
                    try {
                        // Reload chat to get proper IDs (assistant message is already saved by the server)
                        if (currentChatId) {
                            const updatedChat = await api.getChat(currentChatId);
                            setMessages(updatedChat.messages);
                        }
                    } catch (err) {
                        console.error("Failed to reload chat:", err);
                    }
                },
            );
        } catch (err) {
            console.error("Failed to send message:", err);
            // Remove optimistic messages on error
            setMessages((prev) => prev.filter((msg) => !msg.id.startsWith("temp-")));
        }
    };

    const handlePromptClick = (prompt: string) => {
        handleSendMessage(prompt);
    };

    const displayName = userProfile?.display_name || user?.displayName || user?.email?.split("@")[0];
    const hasMessages = messages.length > 0;

    if (loading && chatId) {
        return (
            <MasterLayout>
                <div className="flex items-center justify-center h-full">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                </div>
            </MasterLayout>
        );
    }

    if (error && chatId) {
        return (
            <MasterLayout>
                <div className="flex items-center justify-center h-full">
                    <div className="text-destructive">Error: {error.message}</div>
                </div>
            </MasterLayout>
        );
    }

    return (
        <MasterLayout footer={<MessageInput onSend={handleSendMessage} disabled={streaming} models={models} selectedModel={selectedModel} onModelChange={setSelectedModel} webSearchEnabled={webSearchEnabled} onWebSearchToggle={setWebSearchEnabled} />}>
            <div className="mx-auto w-full max-w-4xl">{hasMessages ? <MessageList messages={messages} /> : <ChatPlaceholder userName={displayName} onPromptClick={handlePromptClick} />}</div>
        </MasterLayout>
    );
}
