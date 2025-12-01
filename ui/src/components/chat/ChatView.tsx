import { useEffect, useState, type ReactNode } from 'react';
import { useLibreChatStreaming } from '@/hooks/useLibreChatStreaming';
import { MessageList } from './MessageList';
import { LibreChatMessageInput } from './LibreChatMessageInput';
import { EndpointSelector } from '@/components/Endpoints/EndpointSelector';
import { EndpointSettings } from '@/components/Endpoints/EndpointSettings';
import { FileUpload } from '@/components/Files/FileUpload';
import { 
  useLibreChatCurrentConversation,
  useLibreChatConversations 
} from '@/stores/appStore';
import { Button } from '@/components/ui/button';
import { Settings2, Plus, Paperclip, X } from 'lucide-react';
import { librechatClient } from '@/lib/librechat-client';
import { toast } from '@/lib/toast';
import { getErrorMessage } from '@/lib/utils';
import type { Endpoint, EndpointOption, Message } from '@/types/librechat';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';

export function ChatView({ conversationId }: { conversationId: string | null }) {
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

  // Load conversation when conversationId changes
  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId);
    } else {
      clearCurrentConversation();
    }
  }, [conversationId, loadConversation, clearCurrentConversation]);

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
    }
  }, [currentConversation]);

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
        // Navigate to new conversation (you'll need to add navigation logic)
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
          model: endpointOptions.model,
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

  const renderShell = (content: ReactNode) => (
    <div className="flex h-full flex-col bg-background px-4 py-6 sm:px-6">
      <div className="mx-auto flex w-full max-w-4xl flex-1 flex-col gap-5">
        {/* Header with endpoint/model selector and settings */}
        <div className="rounded-xl border border-border bg-card p-4 shadow-sm sm:p-5">
          <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
            <div className="flex flex-1 items-center gap-3">
              <div className="space-y-2">
                <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Provider</p>
                <EndpointSelector
                  value={endpointOptions.endpoint}
                  onChange={(endpoint) => setEndpointOptions({ ...endpointOptions, endpoint })}
                />
              </div>
              
              <div className="space-y-2 flex-1 min-w-[180px]">
                <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Model</p>
                <input
                  type="text"
                  value={endpointOptions.model || ''}
                  onChange={(e) => setEndpointOptions({ ...endpointOptions, model: e.target.value })}
                  className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm"
                  placeholder="Enter model name"
                />
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Sheet open={showSettings} onOpenChange={setShowSettings}>
                <SheetTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    className="rounded-full border-border bg-background text-sm font-medium shadow-sm"
                  >
                    <Settings2 className="mr-2 h-4 w-4" />
                    Settings
                  </Button>
                </SheetTrigger>
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

              <Button
                onClick={handleNewConversation}
                variant="outline"
                size="sm"
                className="rounded-full border-border bg-background text-sm font-medium shadow-sm"
              >
                <Plus className="mr-2 h-4 w-4" />
                New chat
              </Button>
            </div>
          </div>

          {/* File upload area */}
          {showFileUpload && (
            <div className="mt-4 pt-4 border-t border-border">
              <div className="flex items-center justify-between mb-2">
                <p className="text-sm font-medium">Attach Files</p>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setShowFileUpload(false);
                    setUploadedFiles([]);
                  }}
                >
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
                      className="flex items-center gap-2 rounded-md border border-border bg-background px-2 py-1 text-xs"
                    >
                      <span className="truncate max-w-[200px]">{file.name}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-4 w-4 p-0"
                        onClick={() => {
                          setUploadedFiles(files => files.filter((_, i) => i !== index));
                        }}
                      >
                        <X className="h-3 w-3" />
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
        {content}
      </div>
    </div>
  );

  if (loading && conversationId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
        Loading conversation…
      </div>
    );
  }

  // Error is now handled via toast, but we still show a fallback UI
  if (error && conversationId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
        Unable to load conversation. Please try again.
      </div>
    );
  }

  if (!currentConversation && conversationId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
        Conversation not found
      </div>
    );
  }

  if (!conversationId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-center text-sm text-muted-foreground">
        Select a conversation or start a new one to begin.
      </div>
    );
  }

  return renderShell(
    <div className="flex flex-1 flex-col gap-5">
      <div className="relative flex-1 overflow-hidden rounded-xl border border-border bg-card shadow-sm">
        <MessageList messages={messages} streaming={streaming} />
      </div>
      <div className="space-y-2">
        {!showFileUpload && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowFileUpload(true)}
            className="w-full sm:w-auto"
          >
            <Paperclip className="mr-2 h-4 w-4" />
            Attach Files
          </Button>
        )}
        <LibreChatMessageInput
          onSend={handleSendMessage}
          disabled={streaming}
          onCancel={streaming ? cancelStreaming : undefined}
        />
      </div>
    </div>
  );
}

