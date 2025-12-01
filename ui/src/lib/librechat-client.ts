// ============================================================================
// LibreChat API Client - Phase 1B
// ============================================================================
// API client for LibreChat-specific endpoints using the normalized schema

import { api } from "./api-client";
import type {
    Conversation,
    CreateConversationRequest,
    UpdateConversationRequest,
    Message,
    CreateMessageRequest,
    Preset,
    CreatePresetRequest,
    UpdatePresetRequest,
    Agent,
    CreateAgentRequest,
    Assistant,
    Tag,
    CreateTagRequest,
    Tool,
    File,
    ChatCompletionRequest,
    ChatCompletionResponse,
    ConversationWithTags,
    AgentWithDetails,
} from "@/types/librechat";

// ============================================================================
// Paginated Response Type
// ============================================================================

export interface PaginatedResponse<T> {
    data: T[];
    total: number;
    page: number;
    pageSize: number;
}

// ============================================================================
// Conversations API
// ============================================================================

export const conversations = {
    /**
     * List all conversations for the current user
     */
    list: async (params?: { page?: number; pageSize?: number; isArchived?: boolean }): Promise<PaginatedResponse<ConversationWithTags>> => {
        return api.list<PaginatedResponse<ConversationWithTags>>("/v1/conversations", params);
    },

    /**
     * Get a single conversation by ID
     */
    get: async (id: string): Promise<ConversationWithTags> => {
        return api.get<ConversationWithTags>(`/v1/conversations/${id}`);
    },

    /**
     * Create a new conversation
     */
    create: async (data: CreateConversationRequest): Promise<Conversation> => {
        return api.post<Conversation>("/v1/conversations", data);
    },

    /**
     * Update an existing conversation
     */
    update: async (id: string, data: UpdateConversationRequest): Promise<Conversation> => {
        return api.update<Conversation>(`/v1/conversations/${id}`, data);
    },

    /**
     * Delete a conversation
     */
    delete: async (id: string): Promise<void> => {
        return api.delete(`/v1/conversations/${id}`);
    },

    /**
     * Archive/unarchive a conversation
     */
    archive: async (id: string, isArchived: boolean): Promise<Conversation> => {
        return api.update<Conversation>(`/v1/conversations/${id}`, { isArchived });
    },

    /**
     * Add tags to a conversation
     */
    addTags: async (id: string, tagIds: string[]): Promise<void> => {
        return api.post<void>(`/v1/conversations/${id}/tags`, { tagIds });
    },

    /**
     * Remove tags from a conversation
     */
    removeTags: async (id: string, tagIds: string[]): Promise<void> => {
        return api.delete(`/v1/conversations/${id}/tags`, { body: { tagIds } });
    },
};

// ============================================================================
// Messages API
// ============================================================================

export const messages = {
    /**
     * List all messages in a conversation
     */
    list: async (conversationId: string): Promise<Message[]> => {
        return api.list<Message[]>(`/v1/conversations/${conversationId}/messages`);
    },

    /**
     * Get a single message by ID
     */
    get: async (conversationId: string, messageId: string): Promise<Message> => {
        return api.get<Message>(`/v1/conversations/${conversationId}/messages/${messageId}`);
    },

    /**
     * Create a new message (without AI response - use chat API for that)
     */
    create: async (conversationId: string, data: CreateMessageRequest): Promise<Message> => {
        return api.post<Message>(`/v1/conversations/${conversationId}/messages`, data);
    },

    /**
     * Delete a message
     */
    delete: async (conversationId: string, messageId: string): Promise<void> => {
        return api.delete(`/v1/conversations/${conversationId}/messages/${messageId}`);
    },
};

// ============================================================================
// Chat API (AI Completions)
// ============================================================================

export const chat = {
    /**
     * Send a message and get AI completion (non-streaming)
     */
    sendMessage: async (data: ChatCompletionRequest): Promise<ChatCompletionResponse> => {
        return api.post<ChatCompletionResponse>("/v1/chat", data);
    },

    /**
     * Send a message and stream the AI completion
     * Returns a Response object that can be used with ReadableStream
     */
    streamMessage: async (data: ChatCompletionRequest): Promise<Response> => {
        return api.stream("/v1/chat/stream", data);
    },

    /**
     * Regenerate the last assistant message in a conversation
     */
    regenerate: async (conversationId: string, messageId: string): Promise<ChatCompletionResponse> => {
        return api.post<ChatCompletionResponse>("/v1/chat/regenerate", { conversationId, messageId });
    },

    /**
     * Continue from a specific message (for branching conversations)
     */
    continueFrom: async (conversationId: string, messageId: string, message: string): Promise<ChatCompletionResponse> => {
        return api.post<ChatCompletionResponse>("/v1/chat/continue", {
            conversationId,
            messageId,
            message,
        });
    },
};

// ============================================================================
// Presets API
// ============================================================================

export const presets = {
    /**
     * List all presets for the current user
     */
    list: async (): Promise<Preset[]> => {
        return api.list<Preset[]>("/v1/presets");
    },

    /**
     * Get a single preset by ID
     */
    get: async (id: string): Promise<Preset> => {
        return api.get<Preset>(`/v1/presets/${id}`);
    },

    /**
     * Create a new preset
     */
    create: async (data: CreatePresetRequest): Promise<Preset> => {
        return api.post<Preset>("/v1/presets", data);
    },

    /**
     * Update an existing preset
     */
    update: async (id: string, data: UpdatePresetRequest): Promise<Preset> => {
        return api.update<Preset>(`/v1/presets/${id}`, data);
    },

    /**
     * Delete a preset
     */
    delete: async (id: string): Promise<void> => {
        return api.delete(`/v1/presets/${id}`);
    },

    /**
     * Set a preset as default
     */
    setDefault: async (id: string): Promise<Preset> => {
        return api.update<Preset>(`/v1/presets/${id}`, { isDefault: true });
    },

    /**
     * Reorder presets
     */
    reorder: async (presetIds: string[]): Promise<void> => {
        return api.post<void>("/v1/presets/reorder", { presetIds });
    },
};

// ============================================================================
// Agents API
// ============================================================================

export const agents = {
    /**
     * List all agents (respecting access levels)
     */
    list: async (params?: { accessLevel?: number }): Promise<Agent[]> => {
        return api.list<Agent[]>("/v1/agents", params);
    },

    /**
     * Get a single agent by ID with full details
     */
    get: async (id: string): Promise<AgentWithDetails> => {
        return api.get<AgentWithDetails>(`/v1/agents/${id}`);
    },

    /**
     * Create a new agent
     */
    create: async (data: CreateAgentRequest): Promise<Agent> => {
        return api.post<Agent>("/v1/agents", data);
    },

    /**
     * Update an existing agent
     */
    update: async (id: string, data: Partial<CreateAgentRequest>): Promise<Agent> => {
        return api.update<Agent>(`/v1/agents/${id}`, data);
    },

    /**
     * Delete an agent
     */
    delete: async (id: string): Promise<void> => {
        return api.delete(`/v1/agents/${id}`);
    },

    /**
     * Add tools to an agent
     */
    addTools: async (id: string, toolIds: string[], configuration?: Record<string, unknown>): Promise<void> => {
        return api.post<void>(`/v1/agents/${id}/tools`, { toolIds, configuration });
    },

    /**
     * Remove tools from an agent
     */
    removeTools: async (id: string, toolIds: string[]): Promise<void> => {
        return api.delete(`/v1/agents/${id}/tools`, { body: { toolIds } });
    },

    /**
     * Add conversation starters to an agent
     */
    addConversationStarters: async (id: string, starters: string[]): Promise<void> => {
        return api.post<void>(`/v1/agents/${id}/conversation-starters`, { starters });
    },

    /**
     * Update conversation starters for an agent
     */
    updateConversationStarters: async (id: string, starters: { id?: string; text: string; orderIndex: number }[]): Promise<void> => {
        return api.update<void>(`/v1/agents/${id}/conversation-starters`, { starters });
    },
};

// ============================================================================
// Assistants API (OpenAI Assistants API compatibility)
// ============================================================================

export const assistants = {
    /**
     * List all assistants for the current user
     */
    list: async (): Promise<Assistant[]> => {
        return api.list<Assistant[]>("/v1/assistants");
    },

    /**
     * Get a single assistant by ID
     */
    get: async (id: string): Promise<Assistant> => {
        return api.get<Assistant>(`/v1/assistants/${id}`);
    },

    /**
     * Sync with OpenAI Assistants API
     */
    sync: async (): Promise<Assistant[]> => {
        return api.post<Assistant[]>("/v1/assistants/sync", {});
    },
};

// ============================================================================
// Tags API
// ============================================================================

export const tags = {
    /**
     * List all tags for the current user
     */
    list: async (): Promise<Tag[]> => {
        return api.list<Tag[]>("/v1/tags");
    },

    /**
     * Get a single tag by ID
     */
    get: async (id: string): Promise<Tag> => {
        return api.get<Tag>(`/v1/tags/${id}`);
    },

    /**
     * Create a new tag
     */
    create: async (data: CreateTagRequest): Promise<Tag> => {
        return api.post<Tag>("/v1/tags", data);
    },

    /**
     * Update an existing tag
     */
    update: async (id: string, data: Partial<CreateTagRequest>): Promise<Tag> => {
        return api.update<Tag>(`/v1/tags/${id}`, data);
    },

    /**
     * Delete a tag
     */
    delete: async (id: string): Promise<void> => {
        return api.delete(`/v1/tags/${id}`);
    },

    /**
     * Reorder tags
     */
    reorder: async (tagIds: string[]): Promise<void> => {
        return api.post<void>("/v1/tags/reorder", { tagIds });
    },
};

// ============================================================================
// Tools API
// ============================================================================

export const tools = {
    /**
     * List all available tools
     */
    list: async (params?: { isActive?: boolean; toolType?: string }): Promise<Tool[]> => {
        return api.list<Tool[]>("/v1/tools", params);
    },

    /**
     * Get a single tool by ID
     */
    get: async (id: string): Promise<Tool> => {
        return api.get<Tool>(`/v1/tools/${id}`);
    },
};

// ============================================================================
// Files API
// ============================================================================

export const files = {
    /**
     * List all files for the current user
     */
    list: async (params?: { conversationId?: string; fileType?: string }): Promise<File[]> => {
        return api.list<File[]>("/v1/files", params);
    },

    /**
     * Get a single file by ID
     */
    get: async (id: string): Promise<File> => {
        return api.get<File>(`/v1/files/${id}`);
    },

    /**
     * Upload a file
     */
    upload: async (file: globalThis.File, conversationId?: string): Promise<File> => {
        const formData = new FormData();
        formData.append("file", file);
        if (conversationId) {
            formData.append("conversationId", conversationId);
        }

        // TODO: Implement file upload with multipart/form-data
        // This will need a custom request method in api-client.ts
        throw new Error("File upload not yet implemented");
    },

    /**
     * Delete a file
     */
    delete: async (id: string): Promise<void> => {
        return api.delete(`/v1/files/${id}`);
    },

    /**
     * Get file download URL
     */
    getDownloadUrl: async (id: string): Promise<{ url: string }> => {
        return api.get<{ url: string }>(`/v1/files/${id}/download`);
    },
};

// ============================================================================
// Export all as librechatClient
// ============================================================================

export const librechatClient = {
    conversations,
    messages,
    chat,
    presets,
    agents,
    assistants,
    tags,
    tools,
    files,
};

// Individual modules are already exported above with 'export const'

