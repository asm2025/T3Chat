import type { AiProvider } from "./chat";

export interface CreateChatRequest {
    model_provider: AiProvider;
    model_id: string;
    title?: string;
}

export interface CreateMessageRequest {
    content: string;
    role?: "user" | "assistant" | "system";
}

export interface ChatRequest {
    chat_id: string;
    message: string;
    model_provider: AiProvider;
    model_id: string;
    temperature?: number;
    max_tokens?: number;
    stream?: boolean;
}

export interface ChatResponse {
    content: string;
    model: string;
    tokens_used?: number;
    finish_reason?: string;
}

export interface UserApiKey {
    id: string;
    user_id: string;
    provider: AiProvider;
    is_default: boolean;
    created_at: string;
    updated_at: string;
}

export interface CreateUserApiKeyRequest {
    provider: AiProvider;
    api_key: string;
    is_default?: boolean;
}
