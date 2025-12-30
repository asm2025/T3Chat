export type Endpoint = "openai" | "anthropic" | "google" | "custom" | "bedrock";

export interface ModelParameters {
    temperature?: number;
    top_p?: number;
    top_k?: number;
    max_tokens?: number;
    max_output_tokens?: number;
    presence_penalty?: number;
    frequency_penalty?: number;
    stop_sequences?: string[];
    reasoning_effort?: string;
    // Provider-specific fields can be added here
    [key: string]: unknown;
}

export interface FeatureFlags {
    resend_files?: boolean;
    resend_images?: boolean;
    image_detail?: "auto" | "low" | "high";
    prompt_cache?: boolean;
    thinking?: boolean;
    thinking_budget?: number;
    // Provider-specific flags can be added here
    [key: string]: unknown;
}

export interface Chat {
    id: string; // UUID
    chatId: string; // TEXT for API compatibility
    userId: string;
    title: string;

    // Current provider/model (user's last selection)
    endpoint: Endpoint;
    model: string;
    modelLabel?: string;

    // AI Parameters (stored as JSONB)
    modelParameters?: ModelParameters;

    // System/Instructions
    systemMessage?: string;
    instructions?: string;

    // Feature Flags (stored as JSONB)
    featureFlags?: FeatureFlags;

    // Agent/Assistant references
    agentId?: string;
    assistantId?: string;
    agentOptions?: Record<string, unknown>;

    // Metadata
    isArchived: boolean;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

export interface CreateChatRequest {
    title?: string;
    endpoint: Endpoint;
    model: string;
    modelLabel?: string;
    modelParameters?: ModelParameters;
    systemMessage?: string;
    instructions?: string;
    featureFlags?: FeatureFlags;
    agentId?: string;
    assistantId?: string;
}

export interface UpdateChatRequest {
    title?: string;
    endpoint?: Endpoint;
    model?: string;
    modelLabel?: string;
    modelParameters?: ModelParameters;
    systemMessage?: string;
    instructions?: string;
    featureFlags?: FeatureFlags;
    isArchived?: boolean;
}

export type MessageRole = "user" | "assistant" | "system" | "tool";

export interface Message {
    id: string; // UUID
    messageId: string; // TEXT for API compatibility
    chatId: string;
    parentMessageId?: string;

    // Message basics
    role: MessageRole;
    text?: string;
    isCreatedByUser: boolean;

    // AI/Model info (stored for historical accuracy)
    model?: string;
    endpoint?: string;

    // Content (for multimodal messages)
    content?: ContentBlock[];

    // Completion info
    tokenCount?: number;
    finishReason?: string;
    error?: boolean;

    // File attachments
    fileIds?: string[]; // UUID[]

    // Tool/Plugin data
    toolCallId?: string;
    pluginData?: Record<string, unknown>;

    // Metadata
    threadId?: string;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

export interface ContentBlock {
    type: "text" | "image_url" | "image_file" | "file";
    text?: string;
    image_url?: {
        url: string;
        detail?: "auto" | "low" | "high";
    };
    file?: {
        id: string;
        filename: string;
        mimeType: string;
    };
}

export interface CreateMessageRequest {
    text: string;
    role?: MessageRole;
    parentMessageId?: string;
    content?: ContentBlock[];
    fileIds?: string[];
}

// ============================================================================
// Preset Types
// ============================================================================

export interface Preset {
    id: string; // UUID
    presetId: string; // TEXT for API compatibility
    userId: string;
    title: string;
    isDefault: boolean;
    orderIndex: number;

    // Provider/Model
    endpoint: Endpoint;
    model: string;
    modelLabel?: string;

    // AI Parameters
    modelParameters?: ModelParameters;

    // System/Instructions
    systemMessage?: string;
    instructions?: string;

    // Feature Flags
    featureFlags?: FeatureFlags;

    // Agent reference
    agentId?: string;
    agentOptions?: Record<string, unknown>;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

export interface CreatePresetRequest {
    title: string;
    endpoint: Endpoint;
    model: string;
    modelLabel?: string;
    modelParameters?: ModelParameters;
    systemMessage?: string;
    instructions?: string;
    featureFlags?: FeatureFlags;
    agentId?: string;
    isDefault?: boolean;
}

export interface UpdatePresetRequest {
    title?: string;
    endpoint?: Endpoint;
    model?: string;
    modelLabel?: string;
    modelParameters?: ModelParameters;
    systemMessage?: string;
    instructions?: string;
    featureFlags?: FeatureFlags;
    isDefault?: boolean;
    orderIndex?: number;
}

// ============================================================================
// Agent Types
// ============================================================================

export interface Agent {
    id: string; // UUID
    agentId: string; // TEXT for API compatibility
    authorId: string;

    // Basic info
    name: string;
    description?: string;
    instructions?: string;

    // Avatar
    avatarFilepath?: string;
    avatarSource?: "url" | "upload" | "default";

    // Model configuration
    provider: string;
    model: string;
    modelParameters?: ModelParameters;

    // Behavior
    accessLevel: number; // 0=private, 1=shared, 2=public
    recursionLimit: number;
    hideSequentialOutputs: boolean;
    endAfterTools: boolean;
    isCollaborative: boolean;

    // Tool resources
    toolResources?: Record<string, unknown>;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

export interface CreateAgentRequest {
    name: string;
    description?: string;
    instructions?: string;
    provider: string;
    model: string;
    modelParameters?: ModelParameters;
    avatarFilepath?: string;
    avatarSource?: "url" | "upload" | "default";
    accessLevel?: number;
    recursionLimit?: number;
    hideSequentialOutputs?: boolean;
    endAfterTools?: boolean;
    isCollaborative?: boolean;
}

// ============================================================================
// Assistant Types (OpenAI Assistants API compatibility)
// ============================================================================

export interface Assistant {
    id: string; // UUID
    assistantId: string; // OpenAI assistant ID
    userId: string;

    // Basic info
    name?: string;
    description?: string;
    instructions?: string;

    // Avatar
    avatarFilepath?: string;
    avatarSource?: "url" | "upload" | "default";

    // Configuration
    model: string;
    tools?: unknown[]; // OpenAI tools format (JSONB)
    fileIds?: string[]; // UUID[]

    // Behavior
    accessLevel: number;
    appendCurrentDatetime: boolean;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

// ============================================================================
// Tag Types
// ============================================================================

export interface Tag {
    id: string; // UUID
    userId: string;
    name: string;
    description?: string;
    color?: string; // Hex color
    position: number;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

export interface CreateTagRequest {
    name: string;
    description?: string;
    color?: string;
    position?: number;
}

// ============================================================================
// Tool Types
// ============================================================================

export type ToolType = "system" | "plugin" | "function" | "action";

export interface Tool {
    id: string; // UUID
    name: string;
    displayName: string;
    description?: string;
    toolType: ToolType;
    iconUrl?: string;
    isActive: boolean;
    isSystem: boolean;
    configurationSchema?: Record<string, unknown>;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

// ============================================================================
// File Types
// ============================================================================

export type FileType = "image" | "document" | "audio" | "video" | "other";

export interface File {
    id: string; // UUID
    fileId: string; // TEXT for API compatibility
    userId: string;
    chatId?: string;

    // File info
    filename: string;
    filepath: string;
    mimeType: string;
    sizeBytes: number;
    fileType: FileType;

    // Content
    textContent?: string;
    isEmbedded: boolean;

    // Image-specific
    width?: number;
    height?: number;

    // Metadata
    source: "upload" | "url" | "generated";
    metadata?: Record<string, unknown>;

    // Usage tracking
    usageCount: number;
    lastUsedAt?: string;

    // Temporary files
    isTemporary: boolean;
    expiresAt?: string;

    // Timestamps
    createdAt: string;
    updatedAt: string;
}

// ============================================================================
// Endpoint Configuration (UI state)
// ============================================================================

export interface EndpointOption {
    endpoint: Endpoint;
    model: string;
    modelLabel?: string;
    parameters: ModelParameters;
    featureFlags?: FeatureFlags;
    systemMessage?: string;
}

// ============================================================================
// Chat with Tags
// ============================================================================

export interface ChatWithTags extends Chat {
    tags?: Tag[];
}

// ============================================================================
// Agent with Tools and Starters
// ============================================================================

export interface ChatStarter {
    id: string;
    text: string;
    orderIndex: number;
    createdAt: string;
}

export interface AgentWithDetails extends Agent {
    tools?: Tool[];
    chatStarters?: ChatStarter[];
}

// ============================================================================
// Chat API Request/Response Types
// ============================================================================

export interface ChatCompletionRequest {
    chatId?: string;
    message: string;
    endpointOptions: EndpointOption;
    parentMessageId?: string;
    fileIds?: string[];
}

export interface ChatCompletionResponse {
    chat: Chat;
    message: Message;
    usage?: {
        promptTokens: number;
        completionTokens: number;
        totalTokens: number;
    };
}

export interface StreamChunk {
    delta: string;
    finishReason?: string;
}
