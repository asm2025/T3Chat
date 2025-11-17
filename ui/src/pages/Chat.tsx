import { useEffect, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useAuth } from "@/stores/appStore";
import { useStreamingChat } from "@/hooks/useStreamingChat";
import { MessageList } from "@/components/chat/MessageList";
import { MessageInput, type MessageInputRef } from "@/components/chat/MessageInput";
import { ChatPlaceholder } from "@/components/chat/ChatPlaceholder";
import { MasterLayout } from "@/components/MasterLayout";
import { useModels, useChat as useChatStore, useAppStore } from "@/stores/appStore";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import type { Message } from "@/types/chat";

export function Chat() {
    const { chatId } = useParams<{ chatId?: string }>();
    const { models, fetchModels } = useModels();
    const { user, userProfile } = useAuth();
    const navigate = useNavigate();
    const { sendMessage, streaming } = useStreamingChat();
    const messageInputRef = useRef<MessageInputRef>(null);
    
    // Zustand store
    const {
        currentChat,
        messages,
        selectedModel,
        webSearchEnabled,
        loading,
        error,
        setCurrentChatId,
        setCurrentChat,
        setSelectedModel,
        setWebSearchEnabled,
        addMessage,
        updateMessage,
        setMessages,
        clearChat,
        fetchChat,
    } = useChatStore();

    // Fetch models on mount
    useEffect(() => {
        if (models.length === 0) {
            fetchModels();
        }
    }, [models.length, fetchModels]);

    // Fetch chat when chatId changes
    useEffect(() => {
        if (chatId) {
            setCurrentChatId(chatId);
            fetchChat(chatId);
        } else {
            clearChat();
        }
    }, [chatId, fetchChat, setCurrentChatId, clearChat]);

    // Show toast when error occurs
    useEffect(() => {
        if (error && chatId) {
            toast.error("Failed to load chat", {
                description: error.message,
            });
        }
    }, [error, chatId]);

    // Set selected model based on chat or default
    useEffect(() => {
        if (models.length === 0) {
            return;
        }

        if (currentChat) {
            const match = models.find((m) => m.provider === currentChat.model_provider && m.model_id === currentChat.model_id);
            const nextModel = match ?? models[0];
            if (!selectedModel || selectedModel.id !== nextModel.id) {
                setSelectedModel(nextModel);
            }
            return;
        }

        if (!selectedModel) {
            setSelectedModel(models[0]);
        }
    }, [currentChat, models, selectedModel, setSelectedModel]);

    const handleSendMessage = async (content: string) => {
        let currentChatId = chatId;

        // If no chat exists, create one
        if (!currentChatId && selectedModel) {
            try {
                const newChat = await t3ChatClient.createChat({
                    model_provider: selectedModel.provider,
                    model_id: selectedModel.model_id,
                });
                currentChatId = newChat.id;
                navigate(`/${newChat.id}`);
            } catch (err) {
                const errorMessage = getErrorMessage(err);
                toast.error("Failed to create chat", {
                    description: errorMessage,
                });
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
            addMessage(userMessage);

            // Create user message on server
            await t3ChatClient.createMessage(currentChatId, { content, role: "user" });

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
            addMessage(assistantMessage);

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
                    updateMessage(assistantMessageId, { content: assistantContent });
                },
                async () => {
                    try {
                        // Reload chat to get proper IDs (assistant message is already saved by the server)
                        if (currentChatId) {
                            const updatedChat = await t3ChatClient.getChat(currentChatId);
                            setMessages(updatedChat.messages);
                        }
                    } catch (err) {
                        const errorMessage = getErrorMessage(err);
                        toast.error("Failed to reload chat", {
                            description: errorMessage,
                        });
                        console.error("Failed to reload chat:", err);
                    }
                },
            );
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            toast.error("Failed to send message", {
                description: errorMessage,
            });
            console.error("Failed to send message:", err);
            // Remove optimistic messages on error
            const currentMessages = useAppStore.getState().messages;
            setMessages(currentMessages.filter((msg) => !msg.id.startsWith("temp-")));
        }
    };

    const handlePromptClick = (prompt: string) => {
        // Clear the textarea and set the prompt text
        messageInputRef.current?.clearContent();
        messageInputRef.current?.setContent(prompt);
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

    // Error is now handled via toast, but we still show a fallback UI
    if (error && chatId) {
        return (
            <MasterLayout>
                <div className="flex items-center justify-center h-full">
                    <div className="text-muted-foreground">Unable to load chat. Please try again.</div>
                </div>
            </MasterLayout>
        );
    }

    return (
        <MasterLayout footer={<MessageInput ref={messageInputRef} onSend={handleSendMessage} disabled={streaming} models={models} selectedModel={selectedModel} onModelChange={setSelectedModel} webSearchEnabled={webSearchEnabled} onWebSearchToggle={setWebSearchEnabled} />}>
            <div className="mx-auto w-full max-w-4xl">{hasMessages ? <MessageList messages={messages} /> : <ChatPlaceholder userName={displayName} onPromptClick={handlePromptClick} />}</div>
        </MasterLayout>
    );
}
