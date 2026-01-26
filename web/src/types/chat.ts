import type { MessageRole } from "./librechat";
import type { AgentEventsSnapshot } from "./agent-events";

export interface Chat {
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
    agentEvents?: AgentEventsSnapshot;
    metadata?: Record<string, unknown>;
    parentMessageId?: string;
    sequenceNumber: number;
    createdAt: string;
    tokensUsed?: number;
    modelUsed?: string;
}

export interface ChatWithMessages extends Chat {
    messages: Message[];
}

export type AiProvider = "openai" | "anthropic" | "google" | "deepseek" | "ollama" | "openrouter" | "chatllm";
