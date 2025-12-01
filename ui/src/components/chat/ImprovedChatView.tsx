import { useEffect, useState } from 'react';
import { useLibreChatStreaming } from '@/hooks/useLibreChatStreaming';
import { MessageList } from './MessageList';
import { ImprovedMessageInput } from './ImprovedMessageInput';
import { EndpointSelector } from '@/components/Endpoints/EndpointSelector';
import { EndpointSettings } from '@/components/Endpoints/EndpointSettings';
import { FileUpload } from '@/components/Files/FileUpload';
import { 
  useLibreChatCurrentConversation,
  useLibreChatConversations 
} from '@/stores/appStore';
import { Button } from '@/components/ui/button';
import { Plus, X } from 'lucide-react';
import { librechatClient } from '@/lib/librechat-client';
import { toast } from '@/lib/toast';
import { getErrorMessage, cn } from '@/lib/utils';
import type { Endpoint, EndpointOption, Message } from '@/types/librechat';
import type { AIModel } from '@/types/model';
import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { t3ChatClient } from '@/lib/t3-chat-client';

export function ImprovedChatView({ conversationId }: { conversationId: string | null }) {
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

  const { sendMessage, streaming, cancelStreaming } = useLibreChatStreaming();

  // Endpoint configuration state
  const [endpointOptions, setEndpointOptions] = useState<EndpointOption>({
    endpoint: 'openai',
    model: 'gpt-4-turbo',
    temperature: 0.7,
    maxTokens: 2048,
  });

  // File upload state
  const [uploadedFiles, setUploadedFiles] = useState<File[]>([]);
  const [showFileUpload, setShowFileUpload] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  // Model selection state
  const [availableModels, setAvailableModels] = useState<AIModel[]>([]);
  const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);

  // Load conversation when conversationId changes
  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId);
    } else {
      clearCurrentConversation();
    }
  }, [conversationId, loadConversation, clearCurrentConversation]);

  // Fetch available models
  useEffect(() => {
    t3ChatClient.listModels()
      .then(setAvailableModels)
      .catch((err) => {
        console.error('Failed to load models:', err);
        setAvailableModels([]);
      });
  }, []);

  // Sync endpoint options from current conversation
  useEffect(() => {
    if (currentConversation) {
      setEndpointOptions({
        endpoint: currentConversation.endpoint,
        model: currentConversation.model || 'gpt-4-turbo',
        temperature: currentConversation.modelParameters?.temperature,
        maxTokens: currentConversation.modelParameters?.maxTokens,
        topP: currentConversation.modelParameters?.topP,
        topK: currentConversation.modelParameters?.topK,
        presencePenalty: currentConversation.modelParameters?.presencePenalty,
        frequencyPenalty: currentConversation.modelParameters?.frequencyPenalty,
        stopSequences: currentConversation.modelParameters?.stopSequences,
        systemMessage: currentConversation.systemMessage,
      });

      // Find and set selected model based on current conversation
      const model = availableModels.find(m => m.model_id === currentConversation.model);
      if (model) {
        setSelectedModel(model);
      }
    }
  }, [currentConversation, availableModels]);

  const handleNewConversation = async () => {
    try {
      const newConvo = await createConversation({
        title: 'New Chat',
        endpoint: endpointOptions.endpoint,
        model: endpointOptions.model,
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
      
      if (newConvo) {
        window.location.href = `/chat/${newConvo.id}`;
      }
    } catch (err) {
      const errorMessage = getErrorMessage(err);
      toast.error('Failed to create conversation', {
        description: errorMessage,
      });
      console.error('Failed to create conversation:', err);
    }
  };

  // Show toast when error occurs
  useEffect(() => {
    if (error && conversationId) {
      toast.error('Failed to load conversation', {
        description: error.message,
      });
    }
  }, [error, conversationId]);

  const handleSendMessage = async (content: string) => {
    if (!conversationId) return;

    try {
      // Add user message optimistically
      const userMessage: Message = {
        id: `temp-user-${Date.now()}`,
        messageId: `temp-user-${Date.now()}`,
        conversationId,
        role: 'user',
        text: content,
        isCreatedByUser: true,
        createdAt: new Date().toISOString(),
      };
      addMessage(userMessage);

      // Upload files if any
      let fileIds: string[] = [];
      if (uploadedFiles.length > 0) {
        try {
          const uploadPromises = uploadedFiles.map(file =>
            librechatClient.files.upload(conversationId, file)
          );
          const uploadedFileObjs = await Promise.all(uploadPromises);
          fileIds = uploadedFileObjs.map(f => f.id);
        } catch (err) {
          console.error('Failed to upload files:', err);
          toast.error('Failed to upload files', {
            description: getErrorMessage(err),
          });
        }
      }

      // Create assistant message placeholder
      const assistantMessageId = `temp-assistant-${Date.now()}`;
      let assistantText = '';
      const assistantMessage: Message = {
        id: assistantMessageId,
        messageId: assistantMessageId,
        conversationId,
        role: 'assistant',
        text: '',
        isCreatedByUser: false,
        createdAt: new Date().toISOString(),
      };
      addMessage(assistantMessage);

      // Prepare chat request
      const chatRequest = {
        conversationId,
        message: content,
        fileIds,
        endpointOptions: {
          endpoint: endpointOptions.endpoint,
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

      // Stream assistant response
      await sendMessage(
        chatRequest,
        (chunk) => {
          assistantText += chunk.delta;
          updateMessage(assistantMessageId, { text: assistantText });
        },
        async () => {
          // Reload conversation to get server-side IDs
          if (conversationId) {
            await loadConversation(conversationId);
          }
          // Clear uploaded files
          setUploadedFiles([]);
          setShowFileUpload(false);
        },
        (err) => {
          console.error('Streaming error:', err);
          // Remove optimistic messages on error
          removeMessage(userMessage.id);
          removeMessage(assistantMessageId);
        }
      );
    } catch (err) {
      const errorMessage = getErrorMessage(err);
      toast.error('Failed to send message', {
        description: errorMessage,
      });
      console.error('Failed to send message:', err);
    }
  };

  const handleModelChange = (model: AIModel) => {
    setSelectedModel(model);
    setEndpointOptions(prev => ({
      ...prev,
      endpoint: model.provider,
      model: model.model_id,
    }));
  };

  const handleFileAttachment = (source: 'device' | 'url' | 'cloud') => {
    // TODO: Implement different file attachment sources
    console.log('File attachment from:', source);
    if (source === 'device') {
      setShowFileUpload(true);
    } else {
      toast.info('Coming soon', {
        description: `Uploading from ${source} will be available soon.`,
      });
    }
  };

  if (loading && conversationId) {
    return (
      <div className="flex h-full flex-col">
        <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
          Loading conversation…
        </div>
      </div>
    );
  }

  if (error && conversationId) {
    return (
      <div className="flex h-full flex-col">
        <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
          Unable to load conversation. Please try again.
        </div>
      </div>
    );
  }

  if (!currentConversation && conversationId) {
    return (
      <div className="flex h-full flex-col">
        <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
          Conversation not found
        </div>
      </div>
    );
  }

  if (!conversationId) {
    return (
      <div className="flex h-full flex-col">
        <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-center text-sm text-muted-foreground">
          Select a conversation or start a new one to begin.
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header with new chat button */}
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-xl font-semibold truncate">{currentConversation?.title || 'Chat'}</h1>
        <Button
          onClick={handleNewConversation}
          variant="outline"
          size="sm"
          className="rounded-full">
          <Plus className="mr-1 h-4 w-4" />
          New
        </Button>
      </div>

      {/* File upload area */}
      {showFileUpload && (
        <div className="mb-4 rounded-xl border border-border bg-card p-4">
          <div className="flex items-center justify-between mb-2">
            <p className="text-sm font-medium">Attach Files</p>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                setShowFileUpload(false);
                setUploadedFiles([]);
              }}>
              <X className="h-4 w-4" />
            </Button>
          </div>
          <FileUpload
            onFilesSelected={setUploadedFiles}
            maxFiles={5}
            maxSizeMB={10}
          />
          {uploadedFiles.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-2">
              {uploadedFiles.map((file, index) => (
                <div
                  key={index}
                  className="flex items-center gap-2 rounded-md border border-border bg-background px-2 py-1 text-xs">
                  <span className="truncate max-w-[200px]">{file.name}</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-4 w-4 p-0"
                    onClick={() => {
                      setUploadedFiles(files => files.filter((_, i) => i !== index));
                    }}>
                    <X className="h-3 w-3" />
                  </Button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Messages area */}
      <div className="flex-1 overflow-hidden rounded-xl border border-border bg-card shadow-sm mb-4">
        <MessageList messages={messages} streaming={streaming} />
      </div>

      {/* Message Input - Pinned at bottom */}
      <div className="mt-auto">
        <ImprovedMessageInput
          onSend={handleSendMessage}
          disabled={streaming}
          models={availableModels}
          selectedModel={selectedModel}
          onModelChange={handleModelChange}
          onSettingsClick={() => setShowSettings(true)}
          onFileAttach={handleFileAttachment}
        />
      </div>

      {/* Settings Sheet */}
      <Sheet open={showSettings} onOpenChange={setShowSettings}>
        <SheetContent side="right" className="w-full sm:max-w-md overflow-y-auto">
          <SheetHeader>
            <SheetTitle>Model Settings</SheetTitle>
          </SheetHeader>
          <EndpointSettings
            options={endpointOptions}
            onChange={setEndpointOptions}
          />
        </SheetContent>
      </Sheet>
    </div>
  );
}

