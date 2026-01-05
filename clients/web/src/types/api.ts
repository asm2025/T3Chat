import type { AiProvider } from "./chat";
import type { MessageRole } from "./librechat";

export interface CreateChatRequest {
    modelProvider: AiProvider;
    modelId: string;
    title?: string;
}

export interface CreateMessageRequest {
    content: string;
    role?: MessageRole;
}

export interface ChatRequest {
    chatId: string;
    message: string;
    modelProvider: AiProvider;
    modelId: string;
    temperature?: number;
    maxTokens?: number;
    stream?: boolean;
}

export interface ChatResponse {
    content: string;
    model: string;
    usage?: {
        promptTokens: number;
        completionTokens: number;
        totalTokens: number;
    };
    finishReason?: string;
}

export interface UserApiKey {
    id: string;
    userId: string;
    provider: AiProvider;
    isDefault: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface CreateUserApiKeyRequest {
    provider: AiProvider;
    apiKey: string;
    isDefault?: boolean;
}
