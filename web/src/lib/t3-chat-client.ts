import { ApiClient, type ApiClientError } from "./api-client";
import type { ChatRequest, ChatResponse, CreateChatRequest, CreateMessageRequest as ApiCreateMessageRequest, CreateUserApiKeyRequest, UserApiKey } from "@/types/api";
import type { AIModel } from "@/types/model";
import type { Chat, ChatWithMessages, Message, AiProvider } from "@/types/chat";
import type { AgentEvent, AgentEventType } from "@/types/agent-events";
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

const AGENT_EVENT_TYPES = new Set<AgentEventType>(["thinking", "tool_start", "tool_output", "tool_end", "text_delta", "message_complete", "error"]);

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
        return this.get<UserProfile>("/me");
    }

    async updateUser(data: { displayName?: string | null; imageUrl?: string | null }): Promise<UserProfile> {
        return this.update<UserProfile>("/me", data);
    }

    // Models endpoints
    async listModels(): Promise<AIModel[]> {
        return this.list<AIModel[]>("/models");
    }

    async listAllModels(): Promise<AIModel[]> {
        return this.list<AIModel[]>("/models/all");
    }

    async getModel(id: string): Promise<AIModel> {
        return this.get<AIModel>(`/models/${id}`);
    }

    // Chats endpoints
    async listChats(page = 1, pageSize = 20): Promise<{ data: Chat[]; total: number }> {
        return this.list<{ data: Chat[]; total: number }>("/chats", { page, pageSize });
    }

    async getChat(id: string): Promise<ChatWithMessages> {
        const backendResponse = await this.get<BackendChatWithMessagesResponse>(`/chats/${id}`);
        return {
            ...mapBackendChatToChatType(backendResponse.chat),
            messages: backendResponse.messages.map(mapBackendMessageToChatType),
        };
    }

    async createChat(data: CreateChatRequest): Promise<Chat> {
        const backendChat = await this.post<BackendChatResponse>("/chats", data);
        return mapBackendChatToChatType(backendChat);
    }

    async updateChat(id: string, data: { title?: string }): Promise<Chat> {
        return this.update<Chat>(`/chats/${id}`, data);
    }

    async deleteChat(id: string): Promise<void> {
        return this.delete(`/chats/${id}`);
    }

    // Config endpoints
    async getStartupConfig(): Promise<StartupConfigResponse> {
        return this.get<StartupConfigResponse>("/config/startup");
    }

    async getModelsConfig(): Promise<{ providers: Record<string, unknown[]> }> {
        return this.get<{ providers: Record<string, unknown[]> }>("/config/models");
    }

    // Messages endpoints
    async getMessages(chatId: string): Promise<Message[]> {
        return this.get<Message[]>(`/chats/${chatId}/messages`);
    }

    async createMessage(chatId: string, data: ApiCreateMessageRequest): Promise<Message> {
        return this.post<Message>(`/chats/${chatId}/messages`, data);
    }

    // Chat endpoints
    async sendChat(data: ChatRequest): Promise<ChatResponse> {
        return this.post<ChatResponse>("/chat", data);
    }

    async *streamChat(data: ChatRequest): AsyncGenerator<AgentEvent, void, unknown> {
        const response = await this.stream("/chat/stream", { ...data, stream: true });

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
                        const event = this.processSSELine(lineBuffer);
                        if (event !== null && event !== undefined) {
                            yield event;
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
                        const event = this.processSSELine(line);
                        if (event === null) {
                            // End-of-stream marker received, stop processing
                            return;
                        }
                        if (event !== undefined) {
                            yield event;
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
     * Process a single SSE data line and extract an agent event.
     * Returns:
     * - AgentEvent: event to yield
     * - undefined: skip this line
     * - null: stream is done (end marker received)
     */
    private processSSELine(line: string): AgentEvent | undefined | null {
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
                
                // Check for error event type
                if (maybeError.type === "error") {
                    let message = "Streaming request failed";
                    if (typeof maybeError.message === "string" && maybeError.message) {
                        message = maybeError.message;
                    } else if (typeof maybeError.text === "string" && maybeError.text) {
                        message = maybeError.text;
                    } else if (typeof maybeError.error === "string" && maybeError.error) {
                        message = maybeError.error;
                    }
                    // Handle nested error structure if present
                    if (maybeError.error && typeof maybeError.error === "object") {
                        const errorDetail = maybeError.error as Record<string, unknown>;
                        if (typeof errorDetail.message === "string" && errorDetail.message) {
                            message = errorDetail.message;
                            if (typeof errorDetail.code === "string" && errorDetail.code) {
                                message = `[${errorDetail.code}] ${message}`;
                            }
                            if (errorDetail.details) {
                                try {
                                    const detailsStr = JSON.stringify(errorDetail.details, null, 2);
                                    if (detailsStr && detailsStr !== "{}" && detailsStr !== "null") {
                                        message = `${message}\n\nDetails:\n${detailsStr}`;
                                    }
                                } catch {
                                    // Ignore JSON stringify errors
                                }
                            }
                        }
                    }
                    throw new Error(message);
                }
                
                // Legacy error format: { error: true, text: "..." }
                if (maybeError.error === true) {
                    const message = (typeof maybeError.text === "string" && maybeError.text) || (typeof maybeError.error === "string" && maybeError.error) || "Streaming request failed";
                    throw new Error(message);
                }
            }

            if (parsed && typeof parsed === "object") {
                const eventType = (parsed as { type?: unknown }).type;
                if (typeof eventType === "string" && AGENT_EVENT_TYPES.has(eventType as AgentEventType)) {
                    return parsed as AgentEvent;
                }
            }

            return undefined;
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
        return this.list<UserApiKey[]>("/keys");
    }

    async createUserApiKey(data: CreateUserApiKeyRequest): Promise<UserApiKey> {
        return this.post<UserApiKey>("/keys", data);
    }

    async deleteUserApiKey(id: string): Promise<void> {
        return this.delete(`/keys/${id}`);
    }

    // Features endpoints
    async listFeatures(): Promise<{ features: Array<{ feature: string; enabled: boolean }> }> {
        return this.get<{ features: Array<{ feature: string; enabled: boolean }> }>("/features");
    }

    async updateFeature(feature: string, enabled: boolean): Promise<{ feature: string; enabled: boolean }> {
        return this.update<{ feature: string; enabled: boolean }>(`/features/${feature}`, { enabled });
    }

    chats = {
        list: async (params?: { page?: number; pageSize?: number; isArchived?: boolean }) => {
            const response = await this.list<{ data: BackendChatResponse[]; total: number }>("/chats", params);
            return {
                data: response.data.map(mapBackendChatToClient),
                total: response.total,
                page: params?.page,
                pageSize: params?.pageSize,
            } as PaginatedResponse<ChatWithTags>;
        },
        get: async (id: string) => {
            const backendChat = await this.get<BackendChatResponse>(`/chats/${id}`);
            return mapBackendChatToClient(backendChat);
        },
        create: (data: LibreCreateChatRequest) => this.post<LibreChat>("/chats", data),
        update: (id: string, data: UpdateChatRequest) => this.update<LibreChat>(`/chats/${id}`, data),
        delete: (id: string) => this.delete(`/chats/${id}`),
        archive: (id: string, isArchived: boolean) => this.update<LibreChat>(`/chats/${id}`, { isArchived }),
        addTags: (id: string, tagIds: string[]) => this.post<void>(`/chats/${id}/tags`, { tagIds }),
        removeTags: (id: string, tagIds: string[]) => this.delete(`/chats/${id}/tags`, { body: { tagIds } }),
    };

    messages = {
        list: async (chatId: string) => {
            const backendMessages = await this.list<BackendMessageResponse[]>(`/chats/${chatId}/messages`);
            return backendMessages.map(mapBackendMessageToClient);
        },
        get: async (chatId: string, messageId: string) => {
            const backendMessage = await this.get<BackendMessageResponse>(`/chats/${chatId}/messages/${messageId}`);
            return mapBackendMessageToClient(backendMessage);
        },
        create: (chatId: string, data: CreateMessageRequest) => this.post<LibreMessage>(`/chats/${chatId}/messages`, data),
        delete: (chatId: string, messageId: string) => this.delete(`/chats/${chatId}/messages/${messageId}`),
    };

    chat = {
        sendMessage: (data: ChatCompletionRequest) => this.post<ChatCompletionResponse>("/chat", data),
        streamMessage: (data: ChatCompletionRequest) => this.stream("/chat/stream", data),
        regenerate: (chatId: string, messageId: string) => this.post<ChatCompletionResponse>("/chat/regenerate", { chatId, messageId }),
        continueFrom: (chatId: string, messageId: string, message: string) =>
            this.post<ChatCompletionResponse>("/chat/continue", {
                chatId,
                messageId,
                message,
            }),
    };

    presets = {
        list: () => this.list<Preset[]>("/presets"),
        get: (id: string) => this.get<Preset>(`/presets/${id}`),
        create: (data: CreatePresetRequest) => this.post<Preset>("/presets", data),
        update: (id: string, data: UpdatePresetRequest) => this.update<Preset>(`/presets/${id}`, data),
        delete: (id: string) => this.delete(`/presets/${id}`),
        setDefault: (id: string) => this.update<Preset>(`/presets/${id}`, { isDefault: true }),
        reorder: (presetIds: string[]) => this.post<void>("/presets/reorder", { presetIds }),
    };

    agents = {
        list: (params?: { accessLevel?: number }) => this.list<Agent[]>("/agents", params),
        get: (id: string) => this.get<AgentWithDetails>(`/agents/${id}`),
        create: (data: CreateAgentRequest) => this.post<Agent>("/agents", data),
        update: (id: string, data: Partial<CreateAgentRequest>) => this.update<Agent>(`/agents/${id}`, data),
        delete: (id: string) => this.delete(`/agents/${id}`),
        addTools: (id: string, toolIds: string[], configuration?: Record<string, unknown>) => this.post<void>(`/agents/${id}/tools`, { toolIds, configuration }),
        removeTools: (id: string, toolIds: string[]) => this.delete(`/agents/${id}/tools`, { body: { toolIds } }),
        addChatStarters: (id: string, starters: string[]) => this.post<void>(`/agents/${id}/chat-starters`, { starters }),
        updateChatStarters: (id: string, starters: { id?: string; text: string; orderIndex: number }[]) => this.update<void>(`/agents/${id}/chat-starters`, { starters }),
    };

    assistants = {
        list: () => this.list<Assistant[]>("/assistants"),
        get: (id: string) => this.get<Assistant>(`/assistants/${id}`),
        sync: () => this.post<Assistant[]>("/assistants/sync", {}),
    };

    tags = {
        list: () => this.list<Tag[]>("/tags"),
        get: (id: string) => this.get<Tag>(`/tags/${id}`),
        create: (data: CreateTagRequest) => this.post<Tag>("/tags", data),
        update: (id: string, data: Partial<CreateTagRequest>) => this.update<Tag>(`/tags/${id}`, data),
        delete: (id: string) => this.delete(`/tags/${id}`),
        reorder: (tagIds: string[]) => this.post<void>("/tags/reorder", { tagIds }),
    };

    tools = {
        list: (params?: { isActive?: boolean; toolType?: string }) => this.list<Tool[]>("/tools", params),
        get: (id: string) => this.get<Tool>(`/tools/${id}`),
    };

    files = {
        list: (params?: { chatId?: string; fileType?: string }) => this.list<File[]>("/files", params),
        get: (id: string) => this.get<File>(`/files/${id}`),
        upload: async (file: globalThis.File, chatId?: string) => {
            const formData = new FormData();
            formData.append("file", file);
            if (chatId) {
                formData.append("chat_id", chatId);
            }
            return this.post<File>("/files", formData);
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
            return this.post<{ text: string }>("/files/stt", formData);
        },
        delete: (id: string) => this.delete(`/files/${id}`),
        getDownloadUrl: (id: string) => this.get<{ url: string }>(`/files/${id}/download`),
    };
}

// Export a singleton instance
export const t3ChatClient = new T3ChatClient();
