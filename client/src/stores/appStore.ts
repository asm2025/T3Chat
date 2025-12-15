import { create } from "zustand";
import type { StoreApi } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { Message, Chat, ChatWithMessages } from "@/types/chat";
import type { AIModel } from "@/types/model";
import type { UserApiKey, CreateUserApiKeyRequest } from "@/types/api";
import type { Conversation, ConversationWithTags, Preset, Agent, AgentWithDetails, Tag, Tool, EndpointOption, Message as LibreChatMessage, CreateConversationRequest } from "@/types/librechat";
import type { StartupConfigResponse } from "@/types/config";

// ============================================================================
// Types
// ============================================================================

interface UserProfile {
    id: string;
    email: string | null;
    display_name: string | null;
    image_url: string | null;
    created_at: string;
    updated_at: string;
}

// ============================================================================
// Auth Slice
// ============================================================================

interface AuthSlice {
    // State
    user: UserProfile | null;
    userProfile: UserProfile | null;
    loading: boolean; // Renamed from authLoading for consistency
    profileLoading: boolean;
    isLoggedOut: boolean;
    refreshTrigger: number;

    // Actions
    setUser: (user: UserProfile | null) => void;
    setUserProfile: (profile: UserProfile | null) => void;
    setLoading: (loading: boolean) => void;
    setProfileLoading: (loading: boolean) => void;
    setIsLoggedOut: (isLoggedOut: boolean) => void;
    setRefreshTrigger: (trigger: number) => void;
    fetchUserProfile: () => Promise<void>;
    logout: () => void;
    forceRefresh: () => void;
}

const createAuthSlice = (set: StoreSet, get: StoreGet): AuthSlice => ({
    // Initial state
    user: null,
    userProfile: null,
    loading: true,
    profileLoading: true,
    isLoggedOut: false,
    refreshTrigger: 0,

    // Actions
    setUser: (user) => set({ user }),
    setUserProfile: (userProfile) => set({ userProfile }),
    setLoading: (loading) => set({ loading }),
    setProfileLoading: (profileLoading) => set({ profileLoading }),
    setIsLoggedOut: (isLoggedOut) => set({ isLoggedOut }),
    setRefreshTrigger: (refreshTrigger) => set({ refreshTrigger }),

    fetchUserProfile: async () => {
        const state = get();
        try {
            state.setProfileLoading(true);
            const profile = await t3ChatClient.getCurrentUser();
            state.setUserProfile(profile);
        } catch (error) {
            if (error instanceof Error && !error.message.includes("Authentication required")) {
                console.error("Failed to fetch user profile:", error);
            }
            state.setUserProfile(null);
        } finally {
            state.setProfileLoading(false);
        }
    },

    logout: () => {
        const state = get();
        state.setIsLoggedOut(true);
        set({
            user: null,
            userProfile: null,
            loading: false,
            profileLoading: false,
        });
    },

    forceRefresh: () => {
        const state = get();
        state.setRefreshTrigger(state.refreshTrigger + 1);
    },
});

// ============================================================================
// Config Slice
// ============================================================================

interface ConfigSlice {
    startupConfig: StartupConfigResponse | null;
    configLoading: boolean;
    configError: Error | null;

    setStartupConfig: (config: StartupConfigResponse | null) => void;
    setConfigLoading: (loading: boolean) => void;
    setConfigError: (error: Error | null) => void;
    fetchStartupConfig: () => Promise<void>;
}

const createConfigSlice = (set: StoreSet, get: StoreGet): ConfigSlice => ({
    startupConfig: null,
    configLoading: false,
    configError: null,

    setStartupConfig: (startupConfig) => set({ startupConfig }),
    setConfigLoading: (configLoading) => set({ configLoading }),
    setConfigError: (configError) => set({ configError }),

    fetchStartupConfig: async () => {
        const state = get();
        try {
            state.setConfigLoading(true);
            state.setConfigError(null);
            const config = await t3ChatClient.getStartupConfig();
            state.setStartupConfig(config);
        } catch (error) {
            state.setConfigError(error as Error);
        } finally {
            state.setConfigLoading(false);
        }
    },
});

// ============================================================================
// Models Slice
// ============================================================================

interface ModelsSlice {
    // State
    models: AIModel[];
    modelsLoading: boolean;
    modelsError: Error | null;

    // Actions
    setModels: (models: AIModel[]) => void;
    setModelsLoading: (loading: boolean) => void;
    setModelsError: (error: Error | null) => void;
    fetchModels: () => Promise<void>;
}

const createModelsSlice = (set: StoreSet, get: StoreGet): ModelsSlice => ({
    // Initial state
    models: [],
    modelsLoading: false,
    modelsError: null,

    // Actions
    setModels: (models) => set({ models }),
    setModelsLoading: (modelsLoading) => set({ modelsLoading }),
    setModelsError: (modelsError) => set({ modelsError }),

    fetchModels: async () => {
        const state = get();
        try {
            state.setModelsLoading(true);
            state.setModelsError(null);
            const models = await t3ChatClient.listModels();
            state.setModels(models);
        } catch (error) {
            state.setModelsError(error as Error);
        } finally {
            state.setModelsLoading(false);
        }
    },
});

// ============================================================================
// Chats Slice
// ============================================================================

interface ChatsSlice {
    // State
    chats: Chat[];
    chatsLoading: boolean;
    chatsError: Error | null;
    chatsTotal: number;

    // Actions
    setChats: (chats: Chat[]) => void;
    setChatsLoading: (loading: boolean) => void;
    setChatsError: (error: Error | null) => void;
    setChatsTotal: (total: number) => void;
    fetchChats: (page?: number, pageSize?: number) => Promise<void>;
    addChat: (chat: Chat) => void;
    updateChat: (id: string, updates: Partial<Chat>) => void;
    removeChat: (id: string) => void;
}

const createChatsSlice = (set: StoreSet, get: StoreGet): ChatsSlice => ({
    // Initial state
    chats: [],
    chatsLoading: false,
    chatsError: null,
    chatsTotal: 0,

    // Actions
    setChats: (chats) => set({ chats }),
    setChatsLoading: (chatsLoading) => set({ chatsLoading }),
    setChatsError: (chatsError) => set({ chatsError }),
    setChatsTotal: (chatsTotal) => set({ chatsTotal }),

    fetchChats: async (page = 1, pageSize = 20) => {
        const state = get();
        try {
            state.setChatsLoading(true);
            state.setChatsError(null);
            const result = await t3ChatClient.listChats(page, pageSize);
            state.setChats(result.data);
            state.setChatsTotal(result.total);
        } catch (error) {
            state.setChatsError(error as Error);
        } finally {
            state.setChatsLoading(false);
        }
    },

    addChat: (chat: Chat) => {
        const state = get();
        state.setChats([chat, ...state.chats]);
    },

    updateChat: (id, updates) => {
        const state = get();
        state.setChats(state.chats.map((chat: Chat) => (chat.id === id ? { ...chat, ...updates } : chat)));
    },

    removeChat: (id) => {
        const state = get();
        state.setChats(state.chats.filter((chat: Chat) => chat.id !== id));
    },
});

// ============================================================================
// Chat Slice (Current Chat)
// ============================================================================

interface ChatSlice {
    // State
    currentChatId: string | null;
    currentChat: ChatWithMessages | null;
    messages: Message[];
    selectedModel: AIModel | null;
    webSearchEnabled: boolean;
    chatLoading: boolean;
    chatError: Error | null;

    // Actions
    setCurrentChatId: (chatId: string | null) => void;
    setCurrentChat: (chat: ChatWithMessages | null) => void;
    setMessages: (messages: Message[]) => void;
    addMessage: (message: Message) => void;
    updateMessage: (messageId: string, updates: Partial<Message>) => void;
    removeMessage: (messageId: string) => void;
    setSelectedModel: (model: AIModel | null) => void;
    setWebSearchEnabled: (enabled: boolean) => void;
    setChatLoading: (loading: boolean) => void;
    setChatError: (error: Error | null) => void;
    fetchChat: (chatId: string) => Promise<void>;
    clearChat: () => void;
    resetChat: () => void;
}

const createChatSlice = (set: StoreSet, get: StoreGet): ChatSlice => ({
    // Initial state
    currentChatId: null,
    currentChat: null,
    messages: [],
    selectedModel: null,
    webSearchEnabled: false,
    chatLoading: false,
    chatError: null,

    // Actions
    setCurrentChatId: (currentChatId) => set({ currentChatId }),
    setCurrentChat: (currentChat) =>
        set({
            currentChat,
            currentChatId: currentChat?.id ?? null,
            messages: currentChat?.messages ?? [],
        }),
    setMessages: (messages) => set({ messages }),
    addMessage: (message) =>
        set((state) => ({
            messages: [...state.messages, message],
        })),
    updateMessage: (messageId, updates) =>
        set((state) => ({
            messages: state.messages.map((msg) => (msg.id === messageId ? { ...msg, ...updates } : msg)),
        })),
    removeMessage: (messageId) =>
        set((state) => ({
            messages: state.messages.filter((msg) => msg.id !== messageId),
        })),
    setSelectedModel: (selectedModel) => set({ selectedModel }),
    setWebSearchEnabled: (webSearchEnabled) => set({ webSearchEnabled }),

    fetchChat: async (chatId) => {
        const state = get();
        try {
            state.setChatLoading(true);
            state.setChatError(null);
            const chat = await t3ChatClient.getChat(chatId);
            state.setCurrentChat(chat);
        } catch (error) {
            state.setChatError(error as Error);
        } finally {
            state.setChatLoading(false);
        }
    },

    clearChat: () =>
        set({
            currentChatId: null,
            currentChat: null,
            messages: [],
        }),

    resetChat: () =>
        set({
            currentChatId: null,
            currentChat: null,
            messages: [],
            selectedModel: null,
            webSearchEnabled: false,
        }),

    setChatLoading: (chatLoading) => set({ chatLoading }),
    setChatError: (chatError) => set({ chatError }),
});

// ============================================================================
// Features Slice
// ============================================================================

interface FeaturesSlice {
    // State
    features: Record<string, boolean>; // feature name -> enabled
    featuresLoading: boolean;
    featuresError: Error | null;

    // Actions
    setFeatures: (features: Record<string, boolean>) => void;
    setFeature: (feature: string, enabled: boolean) => void;
    setFeaturesLoading: (loading: boolean) => void;
    setFeaturesError: (error: Error | null) => void;
    fetchFeatures: () => Promise<void>;
    updateFeature: (feature: string, enabled: boolean) => Promise<void>;
    isFeatureEnabled: (feature: string) => boolean;
}

const createFeaturesSlice = (set: StoreSet, get: StoreGet): FeaturesSlice => ({
    // Initial state
    features: {},
    featuresLoading: false,
    featuresError: null,

    // Actions
    setFeatures: (features) => set({ features }),
    setFeature: (feature, enabled) =>
        set((state) => ({
            features: { ...state.features, [feature]: enabled },
        })),
    setFeaturesLoading: (featuresLoading) => set({ featuresLoading }),
    setFeaturesError: (featuresError) => set({ featuresError }),

    fetchFeatures: async () => {
        const state = get();
        try {
            state.setFeaturesLoading(true);
            state.setFeaturesError(null);
            const response = await t3ChatClient.listFeatures();
            const featuresMap: Record<string, boolean> = {};
            response.features.forEach((f) => {
                featuresMap[f.feature] = f.enabled;
            });
            state.setFeatures(featuresMap);
        } catch (error) {
            state.setFeaturesError(error as Error);
        } finally {
            state.setFeaturesLoading(false);
        }
    },

    updateFeature: async (feature, enabled) => {
        const state = get();
        try {
            await t3ChatClient.updateFeature(feature, enabled);
            state.setFeature(feature, enabled);
        } catch (error) {
            state.setFeaturesError(error as Error);
            throw error;
        }
    },

    isFeatureEnabled: (feature) => {
        const state = get();
        return state.features[feature] ?? false;
    },
});

// ============================================================================
// User API Keys Slice
// ============================================================================

interface UserApiKeysSlice {
    // State
    apiKeys: UserApiKey[];
    apiKeysLoading: boolean;
    apiKeysError: Error | null;

    // Actions
    setApiKeys: (keys: UserApiKey[]) => void;
    setApiKeysLoading: (loading: boolean) => void;
    setApiKeysError: (error: Error | null) => void;
    fetchApiKeys: () => Promise<void>;
    createApiKey: (data: CreateUserApiKeyRequest) => Promise<UserApiKey>;
    deleteApiKey: (id: string) => Promise<void>;
}

const createUserApiKeysSlice = (set: StoreSet, get: StoreGet): UserApiKeysSlice => ({
    // Initial state
    apiKeys: [],
    apiKeysLoading: false,
    apiKeysError: null,

    // Actions
    setApiKeys: (apiKeys) => set({ apiKeys }),
    setApiKeysLoading: (apiKeysLoading) => set({ apiKeysLoading }),
    setApiKeysError: (apiKeysError) => set({ apiKeysError }),

    fetchApiKeys: async () => {
        const state = get();
        try {
            state.setApiKeysLoading(true);
            state.setApiKeysError(null);
            const keys = await t3ChatClient.listUserApiKeys();
            state.setApiKeys(keys);
        } catch (error) {
            state.setApiKeysError(error as Error);
        } finally {
            state.setApiKeysLoading(false);
        }
    },

    createApiKey: async (data) => {
        const state = get();
        const newKey = await t3ChatClient.createUserApiKey(data);
        state.setApiKeys([...state.apiKeys, newKey]);
        return newKey;
    },

    deleteApiKey: async (id) => {
        const state = get();
        await t3ChatClient.deleteUserApiKey(id);
        state.setApiKeys(state.apiKeys.filter((k: UserApiKey) => k.id !== id));
    },
});

// ============================================================================
// LibreChat Conversations Slice
// ============================================================================

interface LibreChatConversationsSlice {
    // State
    conversations: ConversationWithTags[];
    conversationsLoading: boolean;
    conversationsError: Error | null;

    // Actions
    setConversations: (conversations: ConversationWithTags[]) => void;
    setConversationsLoading: (loading: boolean) => void;
    setConversationsError: (error: Error | null) => void;
    fetchConversations: (params?: { page?: number; pageSize?: number; isArchived?: boolean }) => Promise<void>;
    createConversation: (data: CreateConversationRequest) => Promise<ConversationWithTags | null>;
    addConversation: (conversation: ConversationWithTags) => void;
    updateConversation: (id: string, updates: Partial<Conversation>) => void;
    removeConversation: (id: string) => void;
}

const createLibreChatConversationsSlice = (set: StoreSet, get: StoreGet): LibreChatConversationsSlice => ({
    // Initial state
    conversations: [],
    conversationsLoading: false,
    conversationsError: null,

    // Actions
    setConversations: (conversations) => set({ conversations }),
    setConversationsLoading: (conversationsLoading) => set({ conversationsLoading }),
    setConversationsError: (conversationsError) => set({ conversationsError }),

    fetchConversations: async (params) => {
        const state = get();
        try {
            state.setConversationsLoading(true);
            state.setConversationsError(null);
            const result = await t3ChatClient.conversations.list(params);
            state.setConversations(result.data);
        } catch (error) {
            state.setConversationsError(error as Error);
        } finally {
            state.setConversationsLoading(false);
        }
    },

    createConversation: async (data: CreateConversationRequest) => {
        const state = get();
        try {
            const conversation = await t3ChatClient.conversations.create(data);
            state.addConversation(conversation);
            return conversation;
        } catch (error) {
            console.error("Failed to create conversation:", error);
            return null;
        }
    },

    addConversation: (conversation: ConversationWithTags) => {
        const state = get();
        state.setConversations([conversation, ...state.conversations]);
    },

    updateConversation: (id, updates) => {
        const state = get();
        state.setConversations(state.conversations.map((conv: ConversationWithTags) => (conv.id === id ? { ...conv, ...updates } : conv)));
    },

    removeConversation: (id) => {
        const state = get();
        state.setConversations(state.conversations.filter((conv: ConversationWithTags) => conv.id !== id));
    },
});

// ============================================================================
// LibreChat Current Conversation Slice
// ============================================================================

interface LibreChatCurrentConversationSlice {
    // State
    currentConversation: ConversationWithTags | null;
    currentMessages: LibreChatMessage[];
    endpointOptions: EndpointOption | null;
    currentConversationLoading: boolean;
    currentConversationError: Error | null;

    // Actions
    setCurrentConversation: (conversation: ConversationWithTags | null) => void;
    setCurrentMessages: (messages: LibreChatMessage[]) => void;
    addCurrentMessage: (message: LibreChatMessage) => void;
    updateCurrentMessage: (messageId: string, updates: Partial<LibreChatMessage>) => void;
    removeCurrentMessage: (messageId: string) => void;
    setEndpointOptions: (options: EndpointOption | null) => void;
    clearCurrentConversation: () => void;
    loadConversation: (conversationId: string) => Promise<void>;
    setCurrentConversationLoading: (loading: boolean) => void;
    setCurrentConversationError: (error: Error | null) => void;
}

const createLibreChatCurrentConversationSlice = (set: StoreSet, get: StoreGet): LibreChatCurrentConversationSlice => ({
    // Initial state
    currentConversation: null,
    currentMessages: [],
    endpointOptions: null,
    currentConversationLoading: false,
    currentConversationError: null,

    // Actions
    setCurrentConversation: (currentConversation) =>
        set({
            currentConversation,
            currentMessages: [],
        }),

    setCurrentMessages: (currentMessages) => set({ currentMessages }),

    addCurrentMessage: (message) =>
        set((state) => ({
            currentMessages: [...state.currentMessages, message],
        })),

    updateCurrentMessage: (messageId, updates) =>
        set((state) => ({
            currentMessages: state.currentMessages.map((msg) => (msg.id === messageId ? { ...msg, ...updates } : msg)),
        })),

    removeCurrentMessage: (messageId) =>
        set((state) => ({
            currentMessages: state.currentMessages.filter((msg) => msg.id !== messageId),
        })),

    setEndpointOptions: (endpointOptions) => set({ endpointOptions }),

    clearCurrentConversation: () =>
        set({
            currentConversation: null,
            currentMessages: [],
        }),

    loadConversation: async (conversationId: string) => {
        const state = get();
        try {
            state.setCurrentConversationLoading(true);
            state.setCurrentConversationError(null);
            const conversation = await t3ChatClient.conversations.get(conversationId);
            const messages = await t3ChatClient.messages.list(conversationId);
            state.setCurrentConversation(conversation);
            state.setCurrentMessages(messages);
        } catch (error) {
            state.setCurrentConversationError(error as Error);
        } finally {
            state.setCurrentConversationLoading(false);
        }
    },

    setCurrentConversationLoading: (currentConversationLoading) => set({ currentConversationLoading }),
    setCurrentConversationError: (currentConversationError) => set({ currentConversationError }),
});

// ============================================================================
// Presets Slice
// ============================================================================

interface PresetsSlice {
    // State
    presets: Preset[];
    presetsLoading: boolean;
    presetsError: Error | null;

    // Actions
    setPresets: (presets: Preset[]) => void;
    setPresetsLoading: (loading: boolean) => void;
    setPresetsError: (error: Error | null) => void;
    fetchPresets: () => Promise<void>;
    addPreset: (preset: Preset) => void;
    updatePreset: (id: string, updates: Partial<Preset>) => void;
    removePreset: (id: string) => void;
}

const createPresetsSlice = (set: StoreSet, get: StoreGet): PresetsSlice => ({
    // Initial state
    presets: [],
    presetsLoading: false,
    presetsError: null,

    // Actions
    setPresets: (presets) => set({ presets }),
    setPresetsLoading: (presetsLoading) => set({ presetsLoading }),
    setPresetsError: (presetsError) => set({ presetsError }),

    fetchPresets: async () => {
        const state = get();
        try {
            state.setPresetsLoading(true);
            state.setPresetsError(null);
            const presets = await t3ChatClient.presets.list();
            state.setPresets(presets);
        } catch (error) {
            state.setPresetsError(error as Error);
        } finally {
            state.setPresetsLoading(false);
        }
    },

    addPreset: (preset: Preset) => {
        const state = get();
        state.setPresets([...state.presets, preset]);
    },

    updatePreset: (id, updates) => {
        const state = get();
        state.setPresets(state.presets.map((preset: Preset) => (preset.id === id ? { ...preset, ...updates } : preset)));
    },

    removePreset: (id) => {
        const state = get();
        state.setPresets(state.presets.filter((preset: Preset) => preset.id !== id));
    },
});

// ============================================================================
// Agents Slice
// ============================================================================

interface AgentsSlice {
    // State
    agents: Agent[];
    agentsLoading: boolean;
    agentsError: Error | null;
    currentAgent: AgentWithDetails | null;

    // Actions
    setAgents: (agents: Agent[]) => void;
    setAgentsLoading: (loading: boolean) => void;
    setAgentsError: (error: Error | null) => void;
    setCurrentAgent: (agent: AgentWithDetails | null) => void;
    fetchAgents: (params?: { accessLevel?: number }) => Promise<void>;
    fetchAgent: (id: string) => Promise<void>;
    addAgent: (agent: Agent) => void;
    updateAgent: (id: string, updates: Partial<Agent>) => void;
    removeAgent: (id: string) => void;
}

const createAgentsSlice = (set: StoreSet, get: StoreGet): AgentsSlice => ({
    // Initial state
    agents: [],
    agentsLoading: false,
    agentsError: null,
    currentAgent: null,

    // Actions
    setAgents: (agents) => set({ agents }),
    setAgentsLoading: (agentsLoading) => set({ agentsLoading }),
    setAgentsError: (agentsError) => set({ agentsError }),
    setCurrentAgent: (currentAgent) => set({ currentAgent }),

    fetchAgents: async (params) => {
        const state = get();
        try {
            state.setAgentsLoading(true);
            state.setAgentsError(null);
            const agents = await t3ChatClient.agents.list(params);
            state.setAgents(agents);
        } catch (error) {
            state.setAgentsError(error as Error);
        } finally {
            state.setAgentsLoading(false);
        }
    },

    fetchAgent: async (id) => {
        const state = get();
        try {
            state.setAgentsLoading(true);
            state.setAgentsError(null);
            const agent = await t3ChatClient.agents.get(id);
            state.setCurrentAgent(agent);
        } catch (error) {
            state.setAgentsError(error as Error);
        } finally {
            state.setAgentsLoading(false);
        }
    },

    addAgent: (agent: Agent) => {
        const state = get();
        state.setAgents([...state.agents, agent]);
    },

    updateAgent: (id, updates) => {
        const state = get();
        state.setAgents(state.agents.map((agent: Agent) => (agent.id === id ? { ...agent, ...updates } : agent)));
    },

    removeAgent: (id) => {
        const state = get();
        state.setAgents(state.agents.filter((agent: Agent) => agent.id !== id));
    },
});

// ============================================================================
// Tags Slice
// ============================================================================

interface TagsSlice {
    // State
    tags: Tag[];
    tagsLoading: boolean;
    tagsError: Error | null;

    // Actions
    setTags: (tags: Tag[]) => void;
    setTagsLoading: (loading: boolean) => void;
    setTagsError: (error: Error | null) => void;
    fetchTags: () => Promise<void>;
    addTag: (tag: Tag) => void;
    updateTag: (id: string, updates: Partial<Tag>) => void;
    removeTag: (id: string) => void;
}

const createTagsSlice = (set: StoreSet, get: StoreGet): TagsSlice => ({
    // Initial state
    tags: [],
    tagsLoading: false,
    tagsError: null,

    // Actions
    setTags: (tags) => set({ tags }),
    setTagsLoading: (tagsLoading) => set({ tagsLoading }),
    setTagsError: (tagsError) => set({ tagsError }),

    fetchTags: async () => {
        const state = get();
        try {
            state.setTagsLoading(true);
            state.setTagsError(null);
            const tags = await t3ChatClient.tags.list();
            state.setTags(tags);
        } catch (error) {
            state.setTagsError(error as Error);
        } finally {
            state.setTagsLoading(false);
        }
    },

    addTag: (tag: Tag) => {
        const state = get();
        state.setTags([...state.tags, tag]);
    },

    updateTag: (id, updates) => {
        const state = get();
        state.setTags(state.tags.map((tag: Tag) => (tag.id === id ? { ...tag, ...updates } : tag)));
    },

    removeTag: (id) => {
        const state = get();
        state.setTags(state.tags.filter((tag: Tag) => tag.id !== id));
    },
});

// ============================================================================
// Tools Slice
// ============================================================================

interface ToolsSlice {
    // State
    tools: Tool[];
    toolsLoading: boolean;
    toolsError: Error | null;

    // Actions
    setTools: (tools: Tool[]) => void;
    setToolsLoading: (loading: boolean) => void;
    setToolsError: (error: Error | null) => void;
    fetchTools: (params?: { isActive?: boolean; toolType?: string }) => Promise<void>;
}

const createToolsSlice = (set: StoreSet, get: StoreGet): ToolsSlice => ({
    // Initial state
    tools: [],
    toolsLoading: false,
    toolsError: null,

    // Actions
    setTools: (tools) => set({ tools }),
    setToolsLoading: (toolsLoading) => set({ toolsLoading }),
    setToolsError: (toolsError) => set({ toolsError }),

    fetchTools: async (params) => {
        const state = get();
        try {
            state.setToolsLoading(true);
            state.setToolsError(null);
            const tools = await t3ChatClient.tools.list(params);
            state.setTools(tools);
        } catch (error) {
            state.setToolsError(error as Error);
        } finally {
            state.setToolsLoading(false);
        }
    },
});

// ============================================================================
// Combined Store
// ============================================================================

type AppStore = AuthSlice & ConfigSlice & ModelsSlice & ChatsSlice & ChatSlice & UserApiKeysSlice & FeaturesSlice & LibreChatConversationsSlice & LibreChatCurrentConversationSlice & PresetsSlice & AgentsSlice & TagsSlice & ToolsSlice;

type StoreSet = StoreApi<AppStore>["setState"];
type StoreGet = StoreApi<AppStore>["getState"];

export const useAppStore = create<AppStore>()(
    devtools(
        persist(
            (set, get) => ({
                ...createAuthSlice(set, get),
                ...createConfigSlice(set, get),
                ...createModelsSlice(set, get),
                ...createChatsSlice(set, get),
                ...createChatSlice(set, get),
                ...createUserApiKeysSlice(set, get),
                ...createFeaturesSlice(set, get),
                ...createLibreChatConversationsSlice(set, get),
                ...createLibreChatCurrentConversationSlice(set, get),
                ...createPresetsSlice(set, get),
                ...createAgentsSlice(set, get),
                ...createTagsSlice(set, get),
                ...createToolsSlice(set, get),
            }),
            {
                name: "t3chat-store",
                partialize: (state) => ({
                    // Only persist non-sensitive data
                    selectedModel: state.selectedModel,
                    webSearchEnabled: state.webSearchEnabled,
                    endpointOptions: state.endpointOptions,
                }),
            },
        ),
        { name: "T3Chat Store" },
    ),
);

// Export selectors for convenience
export const useAuth = () =>
    useAppStore(
        useShallow((state) => ({
            user: state.user,
            userProfile: state.userProfile,
            loading: state.loading,
            profileLoading: state.profileLoading,
            setUser: state.setUser,
            setUserProfile: state.setUserProfile,
            setLoading: state.setLoading,
            fetchUserProfile: state.fetchUserProfile,
            logout: state.logout,
            forceRefresh: state.forceRefresh,
        })),
    );

export const useConfig = () =>
    useAppStore(
        useShallow((state) => ({
            config: state.startupConfig,
            loading: state.configLoading,
            error: state.configError,
            fetchStartupConfig: state.fetchStartupConfig,
        })),
    );

export const useModels = () =>
    useAppStore(
        useShallow((state) => ({
            models: state.models,
            loading: state.modelsLoading,
            error: state.modelsError,
            fetchModels: state.fetchModels,
        })),
    );

export const useChats = () =>
    useAppStore(
        useShallow((state) => ({
            chats: state.chats,
            loading: state.chatsLoading,
            error: state.chatsError,
            total: state.chatsTotal,
            fetchChats: state.fetchChats,
            addChat: state.addChat,
            updateChat: state.updateChat,
            removeChat: state.removeChat,
        })),
    );

export const useChat = () =>
    useAppStore(
        useShallow((state) => ({
            currentChatId: state.currentChatId,
            currentChat: state.currentChat,
            messages: state.messages,
            selectedModel: state.selectedModel,
            webSearchEnabled: state.webSearchEnabled,
            loading: state.chatLoading,
            error: state.chatError,
            setCurrentChatId: state.setCurrentChatId,
            setCurrentChat: state.setCurrentChat,
            setMessages: state.setMessages,
            addMessage: state.addMessage,
            updateMessage: state.updateMessage,
            removeMessage: state.removeMessage,
            setSelectedModel: state.setSelectedModel,
            setWebSearchEnabled: state.setWebSearchEnabled,
            fetchChat: state.fetchChat,
            clearChat: state.clearChat,
            resetChat: state.resetChat,
        })),
    );

export const useUserApiKeys = () =>
    useAppStore(
        useShallow((state) => ({
            keys: state.apiKeys,
            loading: state.apiKeysLoading,
            error: state.apiKeysError,
            fetchApiKeys: state.fetchApiKeys,
            createApiKey: state.createApiKey,
            deleteApiKey: state.deleteApiKey,
        })),
    );

// ============================================================================
// LibreChat Selectors
// ============================================================================

export const useLibreChatConversations = () =>
    useAppStore(
        useShallow((state) => ({
            conversations: state.conversations,
            loading: state.conversationsLoading,
            error: state.conversationsError,
            fetchConversations: state.fetchConversations,
            createConversation: state.createConversation,
            addConversation: state.addConversation,
            updateConversation: state.updateConversation,
            removeConversation: state.removeConversation,
        })),
    );

export const useLibreChatCurrentConversation = () =>
    useAppStore(
        useShallow((state) => ({
            currentConversation: state.currentConversation,
            currentMessages: state.currentMessages,
            messages: state.currentMessages,
            endpointOptions: state.endpointOptions,
            loading: state.currentConversationLoading,
            error: state.currentConversationError,
            setCurrentConversation: state.setCurrentConversation,
            setCurrentMessages: state.setCurrentMessages,
            addCurrentMessage: state.addCurrentMessage,
            addMessage: state.addCurrentMessage,
            updateCurrentMessage: state.updateCurrentMessage,
            updateMessage: state.updateCurrentMessage,
            removeCurrentMessage: state.removeCurrentMessage,
            removeMessage: state.removeCurrentMessage,
            setEndpointOptions: state.setEndpointOptions,
            clearCurrentConversation: state.clearCurrentConversation,
            loadConversation: state.loadConversation,
        })),
    );

export const usePresets = () =>
    useAppStore(
        useShallow((state) => ({
            presets: state.presets,
            loading: state.presetsLoading,
            error: state.presetsError,
            fetchPresets: state.fetchPresets,
            addPreset: state.addPreset,
            updatePreset: state.updatePreset,
            removePreset: state.removePreset,
        })),
    );

export const useAgents = () =>
    useAppStore(
        useShallow((state) => ({
            agents: state.agents,
            loading: state.agentsLoading,
            error: state.agentsError,
            currentAgent: state.currentAgent,
            fetchAgents: state.fetchAgents,
            fetchAgent: state.fetchAgent,
            addAgent: state.addAgent,
            updateAgent: state.updateAgent,
            removeAgent: state.removeAgent,
        })),
    );

export const useTags = () =>
    useAppStore(
        useShallow((state) => ({
            tags: state.tags,
            loading: state.tagsLoading,
            error: state.tagsError,
            fetchTags: state.fetchTags,
            addTag: state.addTag,
            updateTag: state.updateTag,
            removeTag: state.removeTag,
        })),
    );

export const useTools = () =>
    useAppStore(
        useShallow((state) => ({
            tools: state.tools,
            loading: state.toolsLoading,
            error: state.toolsError,
            fetchTools: state.fetchTools,
        })),
    );
