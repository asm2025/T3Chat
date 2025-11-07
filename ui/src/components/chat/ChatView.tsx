import { useState, useEffect } from 'react';
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
    navigate(`/chat/${newChat.id}`);
  };

  // Update messages when chat loads
  useEffect(() => {
    if (chat) {
      setMessages(chat.messages || []);
    }
  }, [chat]);

  // Set selected model based on chat or default
  useEffect(() => {
    if (chat && models.length > 0) {
      const model = models.find(m => 
        m.provider === chat.model_provider && m.model_id === chat.model_id
      );
      setSelectedModel(model || models[0]);
    } else if (models.length > 0 && !selectedModel) {
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

  if (loading && chatId) {
    return (
      <div className="flex flex-col h-full">
        <div className="border-b p-4 flex items-center justify-end">
          <Button
            onClick={handleNewChat}
            size="icon"
            variant="ghost"
          >
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        <div className="flex items-center justify-center flex-1">Loading...</div>
      </div>
    );
  }

  if (error && chatId) {
    return (
      <div className="flex flex-col h-full">
        <div className="border-b p-4 flex items-center justify-end">
          <Button
            onClick={handleNewChat}
            size="icon"
            variant="ghost"
          >
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        <div className="flex items-center justify-center flex-1 text-destructive">Error: {error.message}</div>
      </div>
    );
  }

  if (!chat && chatId) {
    return (
      <div className="flex flex-col h-full">
        <div className="border-b p-4 flex items-center justify-end">
          <Button
            onClick={handleNewChat}
            size="icon"
            variant="ghost"
          >
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        <div className="flex items-center justify-center flex-1">Chat not found</div>
      </div>
    );
  }

  if (!chatId) {
    return (
      <div className="flex flex-col h-full">
        <div className="border-b p-4 flex items-center justify-end">
          <Button
            onClick={handleNewChat}
            size="icon"
            variant="ghost"
          >
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        <div className="flex items-center justify-center flex-1 text-muted-foreground">
          Select a chat or create a new one
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4 flex items-center justify-between">
        <ModelSelector
          models={models}
          selectedModel={selectedModel}
          onSelect={setSelectedModel}
        />
        <Button
          onClick={handleNewChat}
          size="icon"
          variant="ghost"
          className="ml-auto"
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      <div className="flex-1 overflow-hidden">
        <MessageList messages={messages} />
      </div>
      <MessageInput onSend={handleSendMessage} disabled={streaming} />
    </div>
  );
}

