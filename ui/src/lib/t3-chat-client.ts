import { ApiClient, type ApiClientError } from "./api-client";
import type { ChatRequest, ChatResponse, CreateChatRequest, CreateMessageRequest, CreateUserApiKeyRequest, UserApiKey } from "@/types/api";
import type { AIModel } from "@/types/model";
import type { Chat, ChatWithMessages, Message } from "@/types/chat";

// Re-export error type
export type { ApiClientError };

interface UserProfile {
    id: string;
    email: string | null;
    display_name: string | null;
    image_url: string | null;
    created_at: string;
    updated_at: string;
}

/**
 * T3Chat-specific API client that extends the abstract ApiClient.
 * Provides typed methods for all T3Chat API endpoints.
 */
export class T3ChatClient extends ApiClient {
    // User profile endpoints
    async getCurrentUser(): Promise<UserProfile> {
        return this.get<UserProfile>("/v1/me");
    }

    async updateUser(data: { display_name?: string | null; image_url?: string | null }): Promise<UserProfile> {
        return this.update<UserProfile>("/v1/me", data);
    }

    // Models endpoints
    async listModels(): Promise<AIModel[]> {
        return this.list<AIModel[]>("/v1/models");
    }

    async listAllModels(): Promise<AIModel[]> {
        return this.list<AIModel[]>("/v1/models/all");
    }

    async getModel(id: string): Promise<AIModel> {
        return this.get<AIModel>(`/v1/models/${id}`);
    }

    // Chats endpoints
    async listChats(page = 1, pageSize = 20): Promise<{ data: Chat[]; total: number }> {
        return this.list<{ data: Chat[]; total: number }>("/v1/chats", { page, page_size: pageSize });
    }

    async getChat(id: string): Promise<ChatWithMessages> {
        return this.get<ChatWithMessages>(`/v1/chats/${id}`);
    }

    async createChat(data: CreateChatRequest): Promise<Chat> {
        return this.post<Chat>("/v1/chats", data);
    }

    async updateChat(id: string, data: { title?: string }): Promise<Chat> {
        return this.update<Chat>(`/v1/chats/${id}`, data);
    }

    async deleteChat(id: string): Promise<void> {
        return this.delete(`/v1/chats/${id}`);
    }

    // Messages endpoints
    async getMessages(chatId: string): Promise<Message[]> {
        return this.get<Message[]>(`/v1/chats/${chatId}/messages`);
    }

    async createMessage(chatId: string, data: CreateMessageRequest): Promise<Message> {
        return this.post<Message>(`/v1/chats/${chatId}/messages`, data);
    }

    // Chat endpoints
    async sendChat(data: ChatRequest): Promise<ChatResponse> {
        return this.post<ChatResponse>("/v1/chat", data);
    }

    async *streamChat(data: ChatRequest): AsyncGenerator<string, void, unknown> {
        const response = await this.stream("/v1/chat/stream", { ...data, stream: true });

        if (!response.body) {
            throw new Error("Response body is null");
        }

        const reader = response.body.getReader();
        const decoder = new TextDecoder();

        try {
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;

                const chunk = decoder.decode(value, { stream: true });
                const lines = chunk.split("\n").filter((line) => line.trim());

                for (const line of lines) {
                    if (line.startsWith("data: ")) {
                        const data = line.slice(6);
                        if (data === "[DONE]") return;
                        try {
                            const parsed = JSON.parse(data);
                            yield parsed.content || "";
                        } catch {
                            // Skip invalid JSON
                        }
                    }
                }
            }
        } finally {
            reader.releaseLock();
        }
    }

    // User API Keys endpoints
    async listUserApiKeys(): Promise<UserApiKey[]> {
        return this.list<UserApiKey[]>("/v1/user-api-keys");
    }

    async createUserApiKey(data: CreateUserApiKeyRequest): Promise<UserApiKey> {
        return this.post<UserApiKey>("/v1/user-api-keys", data);
    }

    async deleteUserApiKey(id: string): Promise<void> {
        return this.delete(`/v1/user-api-keys/${id}`);
    }

    // Features endpoints
    async listFeatures(): Promise<{ features: Array<{ feature: string; enabled: boolean }> }> {
        return this.get<{ features: Array<{ feature: string; enabled: boolean }> }>("/v1/features");
    }

    async updateFeature(feature: string, enabled: boolean): Promise<{ feature: string; enabled: boolean }> {
        return this.update<{ feature: string; enabled: boolean }>(`/v1/features/${feature}`, { enabled });
    }
}

// Export a singleton instance
export const t3ChatClient = new T3ChatClient();
