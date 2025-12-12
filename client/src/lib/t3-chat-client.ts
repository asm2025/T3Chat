import { ApiClient, type ApiClientError } from "./api-client";
import type { ChatRequest, ChatResponse, CreateChatRequest, CreateMessageRequest, CreateUserApiKeyRequest, UserApiKey } from "@/types/api";
import type { AIModel } from "@/types/model";
import type { Chat, ChatWithMessages, Message } from "@/types/chat";
import type {
    Conversation,
    ConversationWithTags,
    CreateConversationRequest,
    UpdateConversationRequest,
    Message as LibreConversationMessage,
    CreateMessageRequest as LibreCreateMessageRequest,
    Preset,
    CreatePresetRequest,
    UpdatePresetRequest,
    Agent,
    AgentWithDetails,
    CreateAgentRequest,
    Assistant,
    Tag,
    CreateTagRequest,
    Tool,
    File as LibreFile,
    ChatCompletionRequest,
    ChatCompletionResponse,
} from "@/types/librechat";
import type { StartupConfigResponse } from "@/types/config";

// Re-export error type
export type { ApiClientError };

export interface PaginatedResponse<T> {
    data: T[];
    total: number;
    page: number;
    pageSize: number;
}

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

    // Config endpoints
    async getStartupConfig(): Promise<StartupConfigResponse> {
        return this.get<StartupConfigResponse>("/v1/config/startup");
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
        return this.list<UserApiKey[]>("/v1/keys");
    }

    async createUserApiKey(data: CreateUserApiKeyRequest): Promise<UserApiKey> {
        return this.post<UserApiKey>("/v1/keys", data);
    }

    async deleteUserApiKey(id: string): Promise<void> {
        return this.delete(`/v1/keys/${id}`);
    }

    // Features endpoints
    async listFeatures(): Promise<{ features: Array<{ feature: string; enabled: boolean }> }> {
        return this.get<{ features: Array<{ feature: string; enabled: boolean }> }>("/v1/features");
    }

    async updateFeature(feature: string, enabled: boolean): Promise<{ feature: string; enabled: boolean }> {
        return this.update<{ feature: string; enabled: boolean }>(`/v1/features/${feature}`, { enabled });
    }

    conversations = {
        list: (params?: { page?: number; pageSize?: number; isArchived?: boolean }) => this.list<PaginatedResponse<ConversationWithTags>>("/v1/conversations", params),
        get: (id: string) => this.get<ConversationWithTags>(`/v1/conversations/${id}`),
        create: (data: CreateConversationRequest) => this.post<Conversation>("/v1/conversations", data),
        update: (id: string, data: UpdateConversationRequest) => this.update<Conversation>(`/v1/conversations/${id}`, data),
        delete: (id: string) => this.delete(`/v1/conversations/${id}`),
        archive: (id: string, isArchived: boolean) => this.update<Conversation>(`/v1/conversations/${id}`, { isArchived }),
        addTags: (id: string, tagIds: string[]) => this.post<void>(`/v1/conversations/${id}/tags`, { tagIds }),
        removeTags: (id: string, tagIds: string[]) => this.delete(`/v1/conversations/${id}/tags`, { body: { tagIds } }),
    };

    messages = {
        list: (conversationId: string) => this.list<LibreConversationMessage[]>(`/v1/conversations/${conversationId}/messages`),
        get: (conversationId: string, messageId: string) => this.get<LibreConversationMessage>(`/v1/conversations/${conversationId}/messages/${messageId}`),
        create: (conversationId: string, data: LibreCreateMessageRequest) => this.post<LibreConversationMessage>(`/v1/conversations/${conversationId}/messages`, data),
        delete: (conversationId: string, messageId: string) => this.delete(`/v1/conversations/${conversationId}/messages/${messageId}`),
    };

    chat = {
        sendMessage: (data: ChatCompletionRequest) => this.post<ChatCompletionResponse>("/v1/chat", data),
        streamMessage: (data: ChatCompletionRequest) => this.stream("/v1/chat/stream", data),
        regenerate: (conversationId: string, messageId: string) => this.post<ChatCompletionResponse>("/v1/chat/regenerate", { conversationId, messageId }),
        continueFrom: (conversationId: string, messageId: string, message: string) =>
            this.post<ChatCompletionResponse>("/v1/chat/continue", {
                conversationId,
                messageId,
                message,
            }),
    };

    presets = {
        list: () => this.list<Preset[]>("/v1/presets"),
        get: (id: string) => this.get<Preset>(`/v1/presets/${id}`),
        create: (data: CreatePresetRequest) => this.post<Preset>("/v1/presets", data),
        update: (id: string, data: UpdatePresetRequest) => this.update<Preset>(`/v1/presets/${id}`, data),
        delete: (id: string) => this.delete(`/v1/presets/${id}`),
        setDefault: (id: string) => this.update<Preset>(`/v1/presets/${id}`, { isDefault: true }),
        reorder: (presetIds: string[]) => this.post<void>("/v1/presets/reorder", { presetIds }),
    };

    agents = {
        list: (params?: { accessLevel?: number }) => this.list<Agent[]>("/v1/agents", params),
        get: (id: string) => this.get<AgentWithDetails>(`/v1/agents/${id}`),
        create: (data: CreateAgentRequest) => this.post<Agent>("/v1/agents", data),
        update: (id: string, data: Partial<CreateAgentRequest>) => this.update<Agent>(`/v1/agents/${id}`, data),
        delete: (id: string) => this.delete(`/v1/agents/${id}`),
        addTools: (id: string, toolIds: string[], configuration?: Record<string, unknown>) => this.post<void>(`/v1/agents/${id}/tools`, { toolIds, configuration }),
        removeTools: (id: string, toolIds: string[]) => this.delete(`/v1/agents/${id}/tools`, { body: { toolIds } }),
        addConversationStarters: (id: string, starters: string[]) => this.post<void>(`/v1/agents/${id}/conversation-starters`, { starters }),
        updateConversationStarters: (id: string, starters: { id?: string; text: string; orderIndex: number }[]) => this.update<void>(`/v1/agents/${id}/conversation-starters`, { starters }),
    };

    assistants = {
        list: () => this.list<Assistant[]>("/v1/assistants"),
        get: (id: string) => this.get<Assistant>(`/v1/assistants/${id}`),
        sync: () => this.post<Assistant[]>("/v1/assistants/sync", {}),
    };

    tags = {
        list: () => this.list<Tag[]>("/v1/tags"),
        get: (id: string) => this.get<Tag>(`/v1/tags/${id}`),
        create: (data: CreateTagRequest) => this.post<Tag>("/v1/tags", data),
        update: (id: string, data: Partial<CreateTagRequest>) => this.update<Tag>(`/v1/tags/${id}`, data),
        delete: (id: string) => this.delete(`/v1/tags/${id}`),
        reorder: (tagIds: string[]) => this.post<void>("/v1/tags/reorder", { tagIds }),
    };

    tools = {
        list: (params?: { isActive?: boolean; toolType?: string }) => this.list<Tool[]>("/v1/tools", params),
        get: (id: string) => this.get<Tool>(`/v1/tools/${id}`),
    };

    files = {
        list: (params?: { conversationId?: string; fileType?: string }) => this.list<LibreFile[]>("/v1/files", params),
        get: (id: string) => this.get<LibreFile>(`/v1/files/${id}`),
        upload: async (file: globalThis.File, conversationId?: string) => {
            const formData = new FormData();
            formData.append("file", file);
            if (conversationId) {
                formData.append("conversation_id", conversationId);
            }
            return this.post<LibreFile>("/v1/files", formData);
        },
        stt: async (file: globalThis.File, opts?: { language?: string; model?: string }) => {
            const formData = new FormData();
            formData.append("file", file);
            if (opts?.language) {
                formData.append("language", opts.language);
            }
            if (opts?.model) {
                formData.append("model", opts.model);
            }
            return this.post<{ text: string }>("/v1/files/stt", formData);
        },
        delete: (id: string) => this.delete(`/v1/files/${id}`),
        getDownloadUrl: (id: string) => this.get<{ url: string }>(`/v1/files/${id}/download`),
    };
}

// Export a singleton instance
export const t3ChatClient = new T3ChatClient();
