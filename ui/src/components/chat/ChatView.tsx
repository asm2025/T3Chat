import { useState, useEffect, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { useChat } from '@/hooks/useChat';
import { useStreamingChat } from '@/hooks/useStreamingChat';
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';
import { ModelSelector } from '@/components/model/ModelSelector';
import { useModels } from '@/hooks/useModels';
import { Button } from '@/components/ui/button';
import { Plus } from 'lucide-react';
import * as api from '@/lib/serverComm';
import type { AIModel } from '@/types/model';
import type { Message } from '@/types/chat';

export function ChatView({ chatId }: { chatId: string | null }) {
  const { chat, loading, error } = useChat(chatId);
  const { models } = useModels();
  const navigate = useNavigate();
  const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const { sendMessage, streaming } = useStreamingChat();

  const handleNewChat = async () => {
    // Create new chat with default model
    const newChat = await api.createChat({
      model_provider: 'openai',
      model_id: 'gpt-3.5-turbo',
    });
    navigate(`/${newChat.id}`);
  };

  // Update messages when chat loads
  useEffect(() => {
    if (chat) {
      setMessages(chat.messages || []);
    }
  }, [chat]);

  // Set selected model based on chat or default
  useEffect(() => {
    if (models.length === 0) {
      return;
    }

    if (chat) {
      const match = models.find(
        (m) => m.provider === chat.model_provider && m.model_id === chat.model_id
      );
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
    if (!chatId || !selectedModel) return;

    try {
      // Add user message optimistically
      const userMessage: Message = {
        id: `temp-${Date.now()}`,
        chat_id: chatId,
        role: 'user',
        content,
        sequence_number: messages.length,
        created_at: new Date().toISOString(),
      };
      setMessages(prev => [...prev, userMessage]);

      // Create user message on server
      await api.createMessage(chatId, { content, role: 'user' });

      // Create assistant message placeholder
      const assistantMessageId = `temp-assistant-${Date.now()}`;
      let assistantContent = '';
      const assistantMessage: Message = {
        id: assistantMessageId,
        chat_id: chatId,
        role: 'assistant',
        content: '',
        sequence_number: messages.length + 1,
        created_at: new Date().toISOString(),
      };
      setMessages(prev => [...prev, assistantMessage]);

      // Stream assistant response
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
          // Update assistant message in real-time
          setMessages(prev => prev.map(msg => 
            msg.id === assistantMessageId 
              ? { ...msg, content: assistantContent }
              : msg
          ));
        },
        async () => {
          try {
            // Save complete message to server
            await api.createMessage(chatId, {
              content: assistantContent,
              role: 'assistant',
            });
            // Reload chat to get proper IDs
            if (chatId) {
              const updatedChat = await api.getChat(chatId);
              setMessages(updatedChat.messages);
            }
          } catch (err) {
            console.error('Failed to save assistant message:', err);
          }
        }
      );
    } catch (err) {
      console.error('Failed to send message:', err);
      // Remove optimistic messages on error
      setMessages(prev => prev.filter(msg => !msg.id.startsWith('temp-')));
    }
  };

  const renderShell = (content: ReactNode) => (
    <div className="flex h-full flex-col bg-background px-4 py-6 sm:px-6">
      <div className="mx-auto flex w-full max-w-4xl flex-1 flex-col gap-5">
        <div className="rounded-xl border border-border bg-card p-4 shadow-sm sm:p-5">
          <div className="flex flex-col gap-4 md:flex-row md:items-center">
            <div className="flex-1 space-y-2">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Model</p>
              <div className="min-w-[220px] max-w-xs">
                {models.length > 0 && selectedModel ? (
                  <ModelSelector
                    models={models}
                    selectedModel={selectedModel}
                    onSelect={setSelectedModel}
                  />
                ) : (
                  <div className="h-10 rounded-lg border border-dashed border-border bg-background" />
                )}
              </div>
            </div>
            <Button
              onClick={handleNewChat}
              variant="outline"
              className="w-full rounded-full border-border bg-background text-sm font-medium shadow-sm md:w-auto"
            >
              <Plus className="mr-2 h-4 w-4" />
              New chat
            </Button>
          </div>
        </div>
        {content}
      </div>
    </div>
  );

  if (loading && chatId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
        Loading conversation…
      </div>
    );
  }

  if (error && chatId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-destructive">
        Error: {error.message}
      </div>
    );
  }

  if (!chat && chatId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-sm text-muted-foreground">
        Chat not found
      </div>
    );
  }

  if (!chatId) {
    return renderShell(
      <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-border bg-card text-center text-sm text-muted-foreground">
        Select a conversation or start a new one to begin.
      </div>
    );
  }

  return renderShell(
    <div className="flex flex-1 flex-col gap-5">
      <div className="relative flex-1 overflow-hidden rounded-xl border border-border bg-card shadow-sm">
        <MessageList messages={messages} />
      </div>
      <MessageInput onSend={handleSendMessage} disabled={streaming} />
    </div>
  );
}

