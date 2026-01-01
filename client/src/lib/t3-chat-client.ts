import { ApiClient, type ApiClientError } from "./api-client";
import type { ChatRequest, ChatResponse, CreateChatRequest, CreateMessageRequest as ApiCreateMessageRequest, CreateUserApiKeyRequest, UserApiKey } from "@/types/api";
import type { AIModel } from "@/types/model";
import type { Chat, ChatWithMessages, Message, AiProvider } from "@/types/chat";
import type {
    Chat as LibreChat,
    ChatWithTags,
    CreateChatRequest as LibreCreateChatRequest,
    UpdateChatRequest,
    Message as LibreMessage,
    MessageRole,
    CreateMessageRequest,
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
    File,
    ChatCompletionRequest,
    ChatCompletionResponse,
    Endpoint,
} from "@/types/librechat";
import type { StartupConfigResponse } from "@/types/config";

// Re-export error type
export type { ApiClientError };

export interface PaginatedResponse<T> {
    data: T[];
    total: number;
    page?: number;
    pageSize?: number;
}

// Backend API response types (matching Rust backend)
interface BackendChatResponse {
    id: string;
    userId: string;
    title: string;
    modelProvider: string;
    modelId: string;
    createdAt: string;
    updatedAt: string;
}

interface BackendMessageResponse {
    id: string;
    chatId: string;
    role: string;
    content: string;
    metadata?: any;
    parentMessageId?: string;
    sequenceNumber: number;
    createdAt: string;
    tokensUsed?: number;
    modelUsed?: string;
}

interface BackendChatWithMessagesResponse {
    chat: BackendChatResponse;
    messages: BackendMessageResponse[];
}

// Mapping functions to convert backend responses to client types
function mapBackendChatToClient(backendChat: BackendChatResponse): ChatWithTags {
    return {
        id: backendChat.id,
        chatId: backendChat.id, // For API compatibility
        userId: backendChat.userId,
        title: backendChat.title,
        endpoint: backendChat.modelProvider as Endpoint,
        model: backendChat.modelId,
        isArchived: false, // Backend doesn't return this in list, default to false
        createdAt: backendChat.createdAt,
        updatedAt: backendChat.updatedAt,
        tags: [], // Backend doesn't return tags in list response
    };
}

function mapBackendMessageToClient(backendMessage: BackendMessageResponse): LibreMessage {
    return {
        id: backendMessage.id,
        messageId: backendMessage.id, // For API compatibility
        chatId: backendMessage.chatId,
        parentMessageId: backendMessage.parentMessageId,
        role: backendMessage.role as MessageRole,
        text: backendMessage.content, // Map content to text
        isCreatedByUser: backendMessage.role === "user",
        model: backendMessage.modelUsed,
        endpoint: undefined, // Not provided in backend response
        tokenCount: backendMessage.tokensUsed,
        createdAt: backendMessage.createdAt,
        updatedAt: backendMessage.createdAt, // Backend doesn't return updatedAt, use createdAt
    };
}

function mapBackendMessageToChatType(backendMessage: BackendMessageResponse): Message {
    return {
        id: backendMessage.id,
        chatId: backendMessage.chatId,
        role: backendMessage.role as MessageRole,
        content: backendMessage.content,
        metadata: backendMessage.metadata,
        parentMessageId: backendMessage.parentMessageId,
        sequenceNumber: backendMessage.sequenceNumber,
        createdAt: backendMessage.createdAt,
        tokensUsed: backendMessage.tokensUsed,
        modelUsed: backendMessage.modelUsed,
    };
}

function mapBackendChatToChatType(backendChat: BackendChatResponse): Chat {
    return {
        id: backendChat.id,
        userId: backendChat.userId,
        title: backendChat.title,
        modelProvider: backendChat.modelProvider as AiProvider,
        modelId: backendChat.modelId,
        createdAt: backendChat.createdAt,
        updatedAt: backendChat.updatedAt,
    };
}

interface UserProfile {
    id: string;
    email: string | null;
    displayName: string | null;
    imageUrl: string | null;
    createdAt: string;
    updatedAt: string;
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

    async updateUser(data: { displayName?: string | null; imageUrl?: string | null }): Promise<UserProfile> {
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
        return this.list<{ data: Chat[]; total: number }>("/v1/chats", { page, pageSize });
    }

    async getChat(id: string): Promise<ChatWithMessages> {
        const backendResponse = await this.get<BackendChatWithMessagesResponse>(`/v1/chats/${id}`);
        return {
            ...mapBackendChatToChatType(backendResponse.chat),
            messages: backendResponse.messages.map(mapBackendMessageToChatType),
        };
    }

    async createChat(data: CreateChatRequest): Promise<Chat> {
        const backendChat = await this.post<BackendChatResponse>("/v1/chats", data);
        return mapBackendChatToChatType(backendChat);
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

    async getModelsConfig(): Promise<{ providers: Record<string, unknown[]> }> {
        return this.get<{ providers: Record<string, unknown[]> }>("/v1/config/models");
    }

    // Messages endpoints
    async getMessages(chatId: string): Promise<Message[]> {
        return this.get<Message[]>(`/v1/chats/${chatId}/messages`);
    }

    async createMessage(chatId: string, data: ApiCreateMessageRequest): Promise<Message> {
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
        let lineBuffer = ""; // Only buffer incomplete lines

        try {
            while (true) {
                const { done, value } = await reader.read();

                if (done) {
                    // Process any remaining buffered line
                    if (lineBuffer.trim()) {
                        const content = this.processSSELine(lineBuffer);
                        if (content !== null && content !== undefined) {
                            yield content;
                        }
                    }
                    break;
                }

                // Decode chunk and append to line buffer
                const chunk = decoder.decode(value, { stream: true });
                lineBuffer += chunk;

                // Process complete lines (lines ending with \n)
                const lines = lineBuffer.split("\n");
                // Keep the last incomplete line in buffer
                lineBuffer = lines.pop() || "";

                // Process each complete line
                for (const line of lines) {
                    if (!line.trim()) continue;

                    // Only process SSE data lines
                    if (line.startsWith("data: ")) {
                        const content = this.processSSELine(line);
                        if (content === null) {
                            // End-of-stream marker received, stop processing
                            return;
                        }
                        if (content !== undefined) {
                            yield content;
                        }
                    }
                }
            }
        } finally {
            reader.releaseLock();
        }
    }

    /**
     * End-of-stream marker: sequence of non-printable control characters
     * that are extremely unlikely to appear in normal AI responses.
     * Uses: NULL, SOH, STX, ETX, EOT, ENQ, ACK, BEL
     */
    private static readonly STREAM_END_MARKER = "\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007";

    /**
     * Process a single SSE data line and extract content.
     * Returns:
     * - string: content to yield
     * - undefined: skip this line (empty delta, etc.)
     * - null: stream is done (end marker received)
     */
    private processSSELine(line: string): string | undefined | null {
        if (!line.startsWith("data: ")) {
            return undefined;
        }

        const data = line.slice(6).trim();

        // Check for end-of-stream marker (sequence of non-printable control characters)
        if (data === T3ChatClient.STREAM_END_MARKER) {
            return null;
        }

        try {
            const parsed = JSON.parse(data);

            // Handle error responses
            if (parsed && typeof parsed === "object") {
                const maybeError = parsed as Record<string, unknown>;
                if (maybeError.error === true) {
                    const message = (typeof maybeError.text === "string" && maybeError.text) || (typeof maybeError.error === "string" && maybeError.error) || "Streaming request failed";
                    throw new Error(message);
                }
            }

            // Extract content from delta or content field
            let content: string | null = null;
            if (typeof parsed?.content === "string") {
                content = parsed.content;
            } else if (typeof parsed?.delta === "string") {
                content = parsed.delta;
            }

            // Yield non-empty content (empty strings are filtered out)
            return content || undefined;
        } catch (error) {
            // If it's an error we threw, re-throw it
            if (error instanceof Error) {
                throw error;
            }
            // Otherwise, skip invalid JSON lines
            return undefined;
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

    chats = {
        list: async (params?: { page?: number; pageSize?: number; isArchived?: boolean }) => {
            const response = await this.list<{ data: BackendChatResponse[]; total: number }>("/v1/chats", params);
            return {
                data: response.data.map(mapBackendChatToClient),
                total: response.total,
                page: params?.page,
                pageSize: params?.pageSize,
            } as PaginatedResponse<ChatWithTags>;
        },
        get: async (id: string) => {
            const backendChat = await this.get<BackendChatResponse>(`/v1/chats/${id}`);
            return mapBackendChatToClient(backendChat);
        },
        create: (data: LibreCreateChatRequest) => this.post<LibreChat>("/v1/chats", data),
        update: (id: string, data: UpdateChatRequest) => this.update<LibreChat>(`/v1/chats/${id}`, data),
        delete: (id: string) => this.delete(`/v1/chats/${id}`),
        archive: (id: string, isArchived: boolean) => this.update<LibreChat>(`/v1/chats/${id}`, { isArchived }),
        addTags: (id: string, tagIds: string[]) => this.post<void>(`/v1/chats/${id}/tags`, { tagIds }),
        removeTags: (id: string, tagIds: string[]) => this.delete(`/v1/chats/${id}/tags`, { body: { tagIds } }),
    };

    messages = {
        list: async (chatId: string) => {
            const backendMessages = await this.list<BackendMessageResponse[]>(`/v1/chats/${chatId}/messages`);
            return backendMessages.map(mapBackendMessageToClient);
        },
        get: async (chatId: string, messageId: string) => {
            const backendMessage = await this.get<BackendMessageResponse>(`/v1/chats/${chatId}/messages/${messageId}`);
            return mapBackendMessageToClient(backendMessage);
        },
        create: (chatId: string, data: CreateMessageRequest) => this.post<LibreMessage>(`/v1/chats/${chatId}/messages`, data),
        delete: (chatId: string, messageId: string) => this.delete(`/v1/chats/${chatId}/messages/${messageId}`),
    };

    chat = {
        sendMessage: (data: ChatCompletionRequest) => this.post<ChatCompletionResponse>("/v1/chat", data),
        streamMessage: (data: ChatCompletionRequest) => this.stream("/v1/chat/stream", data),
        regenerate: (chatId: string, messageId: string) => this.post<ChatCompletionResponse>("/v1/chat/regenerate", { chatId, messageId }),
        continueFrom: (chatId: string, messageId: string, message: string) =>
            this.post<ChatCompletionResponse>("/v1/chat/continue", {
                chatId,
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
        addChatStarters: (id: string, starters: string[]) => this.post<void>(`/v1/agents/${id}/chat-starters`, { starters }),
        updateChatStarters: (id: string, starters: { id?: string; text: string; orderIndex: number }[]) => this.update<void>(`/v1/agents/${id}/chat-starters`, { starters }),
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
        list: (params?: { chatId?: string; fileType?: string }) => this.list<File[]>("/v1/files", params),
        get: (id: string) => this.get<File>(`/v1/files/${id}`),
        upload: async (file: globalThis.File, chatId?: string) => {
            const formData = new FormData();
            formData.append("file", file);
            if (chatId) {
                formData.append("chat_id", chatId);
            }
            return this.post<File>("/v1/files", formData);
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
