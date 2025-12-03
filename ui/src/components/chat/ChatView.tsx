import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useLibreChatStreaming } from "@/hooks/useLibreChatStreaming";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { FileUpload } from "@/components/Files/FileUpload";
import { useLibreChatCurrentConversation, useLibreChatConversations } from "@/stores/appStore";
import { Button } from "@/components/ui/button";
import { Plus, X } from "lucide-react";
import { librechatClient } from "@/lib/librechat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import type { EndpointOption, Message } from "@/types/librechat";
import type { AIModel } from "@/types/model";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { EndpointSettings } from "@/components/Endpoints/EndpointSettings";
import { t3ChatClient } from "@/lib/t3-chat-client";

interface ChatViewProps {
    conversationId: string | null;
}

export function ChatView({ conversationId }: ChatViewProps) {
    const navigate = useNavigate();
    const {
        currentConversation,
        messages,
        loading,
        error,
        loadConversation,
        addMessage,
        updateMessage,
        removeMessage,
        clearCurrentConversation,
    } = useLibreChatCurrentConversation();
    const { createConversation } = useLibreChatConversations();
    const { sendMessage, streaming } = useLibreChatStreaming();

    const [endpointOptions, setEndpointOptions] = useState<EndpointOption>({
        endpoint: "openai",
        model: "gpt-4-turbo",
        temperature: 0.7,
        maxTokens: 2048,
    });
    const [uploadedFiles, setUploadedFiles] = useState<File[]>([]);
    const [showFileUpload, setShowFileUpload] = useState(false);
    const [showSettings, setShowSettings] = useState(false);
    const [availableModels, setAvailableModels] = useState<AIModel[]>([]);
    const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);
    const [creatingConversation, setCreatingConversation] = useState(false);

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
        if (currentConversation) {
            setEndpointOptions({
                endpoint: currentConversation.endpoint,
                model: currentConversation.model || "gpt-4-turbo",
                temperature: currentConversation.modelParameters?.temperature,
                maxTokens: currentConversation.modelParameters?.maxTokens,
                topP: currentConversation.modelParameters?.topP,
                topK: currentConversation.modelParameters?.topK,
                presencePenalty: currentConversation.modelParameters?.presencePenalty,
                frequencyPenalty: currentConversation.modelParameters?.frequencyPenalty,
                stopSequences: currentConversation.modelParameters?.stopSequences,
                systemMessage: currentConversation.systemMessage,
            });

            const model = availableModels.find((m) => m.model_id === currentConversation.model);
            if (model) {
                setSelectedModel(model);
            }
        }
    }, [currentConversation, availableModels]);

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
            const newConversation = await createConversation({
                title: initialMessage?.slice(0, 60) || "New Chat",
                endpoint: selectedModel?.provider || endpointOptions.endpoint,
                model: selectedModel?.model_id || endpointOptions.model,
                modelParameters: {
                    temperature: endpointOptions.temperature,
                    maxTokens: endpointOptions.maxTokens,
                    topP: endpointOptions.topP,
                    topK: endpointOptions.topK,
                    presencePenalty: endpointOptions.presencePenalty,
                    frequencyPenalty: endpointOptions.frequencyPenalty,
                    stopSequences: endpointOptions.stopSequences,
                },
                systemMessage: endpointOptions.systemMessage,
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
                    const uploads = await Promise.all(
                        uploadedFiles.map((file) => librechatClient.files.upload(targetConversationId, file)),
                    );
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

            const chatRequest = {
                conversationId: targetConversationId,
                message: content,
                fileIds,
                endpointOptions: {
                    endpoint: selectedModel?.provider || endpointOptions.endpoint,
                    model: selectedModel?.model_id || endpointOptions.model,
                    temperature: endpointOptions.temperature,
                    maxTokens: endpointOptions.maxTokens,
                    topP: endpointOptions.topP,
                    topK: endpointOptions.topK,
                    presencePenalty: endpointOptions.presencePenalty,
                    frequencyPenalty: endpointOptions.frequencyPenalty,
                    stopSequences: endpointOptions.stopSequences,
                    systemMessage: endpointOptions.systemMessage,
                },
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
        setEndpointOptions((prev) => ({
            ...prev,
            endpoint: model.provider,
            model: model.model_id,
        }));
    };

    const handleRemoveFile = (index: number) => {
        setUploadedFiles((files) => files.filter((_, i) => i !== index));
    };

    const renderEmptyState = (message: string) => (
        <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
            {message}
        </div>
    );

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

        if (!conversationId) {
            return renderEmptyState("Select a conversation or start a new one to begin.");
        }

        return (
            <div className="flex flex-col gap-5">
                <div className="relative flex-1 overflow-hidden rounded-xl border border-border bg-card shadow-sm">
                    <MessageList messages={messages} streaming={streaming} />
                </div>

                {showFileUpload && (
                    <div className="rounded-xl border border-border bg-card p-4 shadow-sm">
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
                                    <div
                                        key={`${file.name}-${index}`}
                                        className="flex items-center gap-2 rounded-md border border-border bg-background px-2 py-1 text-xs">
                                        <span className="truncate max-w-[200px]">{file.name}</span>
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            className="h-4 w-4 p-0"
                                            onClick={() => handleRemoveFile(index)}>
                                            <X className="h-3 w-3" />
                                        </Button>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                )}

                <MessageInput
                    onSend={handleSendMessage}
                    disabled={streaming || creatingConversation}
                    models={availableModels}
                    selectedModel={selectedModel}
                    onModelChange={handleModelChange}
                    onFileAttach={() => setShowFileUpload(true)}
                />
            </div>
        );
    };

    return (
        <div className="flex h-full flex-col">
            <div className="mb-4 flex items-center justify-between">
                <h1 className="truncate text-xl font-semibold">{currentConversation?.title || "Chat"}</h1>
                <div className="flex items-center gap-2">
                    <Button onClick={handleNewConversation} variant="outline" size="sm" className="rounded-full">
                        <Plus className="mr-1 h-4 w-4" />
                        New
                    </Button>
                </div>
            </div>

            <div className="mb-4 flex flex-col gap-4">
                <Button variant="outline" size="sm" onClick={() => setShowSettings(true)} className="w-fit rounded-full">
                    Settings
                </Button>
                <Sheet open={showSettings} onOpenChange={setShowSettings}>
                    <SheetContent side="right" className="w-full overflow-y-auto sm:max-w-md">
                        <SheetHeader>
                            <SheetTitle>Model Settings</SheetTitle>
                        </SheetHeader>
                        <EndpointSettings options={endpointOptions} onChange={setEndpointOptions} />
                    </SheetContent>
                </Sheet>
            </div>

            <div className="flex-1 overflow-hidden rounded-xl border border-border bg-card p-4 shadow-sm">{renderContent()}</div>
        </div>
    );
}
