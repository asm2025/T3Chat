import type { MessageRole } from "./librechat";

export interface Conversation {
    id: string;
    userId: string;
    title: string;
    modelProvider: AiProvider;
    modelId: string;
    createdAt: string;
    updatedAt: string;
    deletedAt?: string;
}

export interface Message {
    id: string;
    chatId: string;
    role: MessageRole;
    content: string;
    metadata?: Record<string, unknown>;
    parentMessageId?: string;
    sequenceNumber: number;
    createdAt: string;
    tokensUsed?: number;
    modelUsed?: string;
}

export interface ConversationWithMessages extends Conversation {
    messages: Message[];
}

export type AiProvider = "openai" | "anthropic" | "google" | "deepseek" | "ollama" | "openrouter" | "chatllm";
