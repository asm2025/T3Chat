import { useEffect, useState, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useLibreChatStreaming } from "@/hooks/useLibreChatStreaming";
import { MessageList } from "./MessageList";
import { MessageInput, type MessageInputRef } from "./MessageInput";
import { FileUpload } from "@/components/Files/FileUpload";
import { useLibreChatCurrentConversation, useLibreChatConversations } from "@/stores/appStore";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import { librechatClient } from "@/lib/librechat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import type { Endpoint, EndpointOption, Message } from "@/types/librechat";
import type { AIModel } from "@/types/model";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { useAuth } from "@/lib/use-auth";
import { createDefaultEndpointOptions } from "@/constants/endpoint-options";

interface ChatViewProps {
    conversationId: string | null;
}

export function ChatView({ conversationId }: ChatViewProps) {
    const navigate = useNavigate();
    const { user } = useAuth();
    const messageInputRef = useRef<MessageInputRef>(null);
    const placeholderUserName = user?.name || user?.email || undefined;
    const { currentConversation, messages, loading, error, loadConversation, addMessage, updateMessage, removeMessage, clearCurrentConversation, endpointOptions, setEndpointOptions } = useLibreChatCurrentConversation();
    const { createConversation } = useLibreChatConversations();
    const { sendMessage, streaming } = useLibreChatStreaming();

    const [uploadedFiles, setUploadedFiles] = useState<File[]>([]);
    const [showFileUpload, setShowFileUpload] = useState(false);
    const [availableModels, setAvailableModels] = useState<AIModel[]>([]);
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);
    const [creatingConversation, setCreatingConversation] = useState(false);
    const defaultEndpointOptionsRef = useRef<EndpointOption>(createDefaultEndpointOptions());
    const activeEndpointOptions = endpointOptions ?? defaultEndpointOptionsRef.current;
    const resolveEndpointOptions = (): EndpointOption => {
        const resolvedEndpoint = (selectedModel?.provider as Endpoint) || activeEndpointOptions.endpoint;
        const resolvedModel = selectedModel?.model_id || activeEndpointOptions.model;
        return {
            ...activeEndpointOptions,
            endpoint: resolvedEndpoint,
            model: resolvedModel,
        };
    };

    useEffect(() => {
        if (conversationId) {
            loadConversation(conversationId);
        } else {
            clearCurrentConversation();
        }
    }, [conversationId, loadConversation, clearCurrentConversation]);

    useEffect(() => {
        let mounted = true;
        t3ChatClient
            .listModels()
            .then((models) => {
                if (mounted) setAvailableModels(models);
            })
            .catch((err) => {
                console.error("Failed to load models:", err);
                if (mounted) setAvailableModels([]);
            });
        return () => {
            mounted = false;
        };
    }, []);

    useEffect(() => {
        if (!endpointOptions) {
            setEndpointOptions(createDefaultEndpointOptions());
        }
    }, [endpointOptions, setEndpointOptions]);

    useEffect(() => {
        if (currentConversation) {
            setEndpointOptions({
                endpoint: currentConversation.endpoint,
                model: currentConversation.model || "gpt-4-turbo",
                modelLabel: currentConversation.modelLabel,
                parameters: currentConversation.modelParameters || {},
                featureFlags: currentConversation.featureFlags,
                systemMessage: currentConversation.systemMessage,
            });

            const model = availableModels.find((m) => m.model_id === currentConversation.model);
            setSelectedModel(model ?? null);
        } else {
            setSelectedModel(null);
        }
    }, [currentConversation, availableModels, setEndpointOptions]);

    useEffect(() => {
        if (error && conversationId) {
            toast.error("Failed to load conversation", {
                description: error.message,
            });
        }
    }, [error, conversationId]);

    const resetComposerState = () => {
        setUploadedFiles([]);
        setShowFileUpload(false);
    };

    const ensureConversationId = async (initialMessage?: string) => {
        if (conversationId) return conversationId;

        if (creatingConversation) throw new Error("Conversation creation already in progress");
        setCreatingConversation(true);
        try {
            const resolvedOptions = resolveEndpointOptions();
            const newConversation = await createConversation({
                title: initialMessage?.slice(0, 60) || "New Chat",
                endpoint: resolvedOptions.endpoint,
                model: resolvedOptions.model,
                modelParameters: resolvedOptions.parameters,
                featureFlags: resolvedOptions.featureFlags,
                systemMessage: resolvedOptions.systemMessage,
            });

            if (!newConversation) {
                throw new Error("Unable to create conversation");
            }

            navigate(`/chat/${newConversation.id}`, { replace: true });
            return newConversation.id;
        } finally {
            setCreatingConversation(false);
        }
    };

    const handleNewConversation = () => {
        clearCurrentConversation();
        setSelectedModel(null);
        resetComposerState();
        navigate("/chat");
    };

    const handleSendMessage = async (content: string) => {
        if (!content.trim()) return;

        let targetConversationId: string;
        try {
            targetConversationId = await ensureConversationId(content);
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            toast.error("Failed to start conversation", {
                description: errorMessage,
            });
            return;
        }

        try {
            const userMessage: Message = {
                id: `temp-user-${Date.now()}`,
                messageId: `temp-user-${Date.now()}`,
                conversationId: targetConversationId,
                role: "user",
                text: content,
                isCreatedByUser: true,
                createdAt: new Date().toISOString(),
            };
            addMessage(userMessage);

            let fileIds: string[] = [];
            if (uploadedFiles.length > 0) {
                try {
                    const uploads = await Promise.all(uploadedFiles.map((file) => librechatClient.files.upload(targetConversationId, file)));
                    fileIds = uploads.map((file) => file.id);
                } catch (err) {
                    console.error("Failed to upload files:", err);
                    toast.error("Failed to upload files", {
                        description: getErrorMessage(err),
                    });
                }
            }

            const assistantMessageId = `temp-assistant-${Date.now()}`;
            let assistantText = "";
            const assistantMessage: Message = {
                id: assistantMessageId,
                messageId: assistantMessageId,
                conversationId: targetConversationId,
                role: "assistant",
                text: "",
                isCreatedByUser: false,
                createdAt: new Date().toISOString(),
            };
            addMessage(assistantMessage);

            const resolvedOptions = resolveEndpointOptions();
            const chatRequest = {
                conversationId: targetConversationId,
                message: content,
                fileIds,
                endpointOptions: resolvedOptions,
            };

            await sendMessage(
                chatRequest,
                (chunk) => {
                    assistantText += chunk.delta;
                    updateMessage(assistantMessageId, { text: assistantText });
                },
                async () => {
                    await loadConversation(targetConversationId);
                    resetComposerState();
                },
                (err) => {
                    console.error("Streaming error:", err);
                    removeMessage(userMessage.id);
                    removeMessage(assistantMessageId);
                },
            );
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            toast.error("Failed to send message", {
                description: errorMessage,
            });
            console.error("Failed to send message:", err);
        }
    };

    const handleModelChange = (model: AIModel) => {
        setSelectedModel(model);
        setEndpointOptions({
            ...activeEndpointOptions,
            endpoint: model.provider as Endpoint,
            model: model.model_id,
            modelLabel: model.display_name,
        });
    };

    const handleRemoveFile = (index: number) => {
        setUploadedFiles((files) => files.filter((_, i) => i !== index));
    };

    const handlePromptPrefill = (prompt: string) => {
        messageInputRef.current?.setContent(prompt);
        if (typeof document === "undefined") return;
        const textarea = document.getElementById("chat-input");
        if (textarea instanceof HTMLTextAreaElement) {
            textarea.focus();
        }
    };

    const renderEmptyState = (message: string) => <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">{message}</div>;

    const renderContent = () => {
        if (loading && conversationId) {
            return renderEmptyState("Loading conversation…");
        }

        if (error && conversationId) {
            return renderEmptyState("Unable to load conversation. Please try again.");
        }

        if (!currentConversation && conversationId) {
            return renderEmptyState("Conversation not found");
        }

        return (
            <div className="flex flex-col gap-5">
                <div className="relative flex-1 overflow-hidden rounded-xl bg-card">
                    <MessageList messages={messages} streaming={streaming} showPlaceholder={!conversationId || messages.length === 0} placeholderUserName={placeholderUserName} onPromptClick={handlePromptPrefill} />
                </div>

                {showFileUpload && (
                    <div className="rounded-xl border border-border bg-card p-4">
                        <div className="mb-2 flex items-center justify-between">
                            <p className="text-sm font-medium">Attach Files</p>
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                    setShowFileUpload(false);
                                    resetComposerState();
                                }}>
                                <X className="h-4 w-4" />
                            </Button>
                        </div>
                        <FileUpload onFilesSelected={setUploadedFiles} maxFiles={5} maxSizeMB={10} />
                        {uploadedFiles.length > 0 && (
                            <div className="mt-2 flex flex-wrap gap-2">
                                {uploadedFiles.map((file, index) => (
                                    <div key={`${file.name}-${index}`} className="flex items-center gap-2 rounded-md border border-border bg-background px-2 py-1 text-xs">
                                        <span className="truncate max-w-[200px]">{file.name}</span>
                                        <Button variant="ghost" size="sm" className="h-4 w-4 p-0" onClick={() => handleRemoveFile(index)}>
                                            <X className="h-3 w-3" />
                                        </Button>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                )}
            </div>
        );
    };

    return (
        <div className="flex h-full flex-col">
            <div className="mb-4 flex-1 overflow-hidden rounded-xl bg-card p-4">{renderContent()}</div>

            <MessageInput ref={messageInputRef} onSend={handleSendMessage} disabled={streaming || creatingConversation} models={availableModels} selectedModel={selectedModel} onModelChange={handleModelChange} onFileAttach={() => setShowFileUpload(true)} />
        </div>
    );
}
