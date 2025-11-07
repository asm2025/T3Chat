export interface Chat {
  id: string;
  user_id: string;
  title: string;
  model_provider: AiProvider;
  model_id: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface Message {
  id: string;
  chat_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata?: Record<string, any>;
  parent_message_id?: string;
  sequence_number: number;
  created_at: string;
  tokens_used?: number;
  model_used?: string;
}

export interface ChatWithMessages extends Chat {
  messages: Message[];
}

export type AiProvider = 'openai' | 'anthropic' | 'google' | 'deepseek' | 'ollama';

