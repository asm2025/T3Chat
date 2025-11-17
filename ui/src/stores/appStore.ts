import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import type { User } from "firebase/auth";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { Message, Chat, ChatWithMessages } from "@/types/chat";
import type { AIModel } from "@/types/model";
import type { UserApiKey, CreateUserApiKeyRequest } from "@/types/api";

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
    user: User | null;
    userProfile: UserProfile | null;
    loading: boolean; // Renamed from authLoading for consistency
    profileLoading: boolean;
    isLoggedOut: boolean;
    refreshTrigger: number;

    // Actions
    setUser: (user: User | null) => void;
    setUserProfile: (profile: UserProfile | null) => void;
    setLoading: (loading: boolean) => void;
    setProfileLoading: (loading: boolean) => void;
    setIsLoggedOut: (isLoggedOut: boolean) => void;
    setRefreshTrigger: (trigger: number) => void;
    fetchUserProfile: () => Promise<void>;
    logout: () => void;
    forceRefresh: () => void;
}

const createAuthSlice = (set: any, get: any): AuthSlice => ({
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

const createModelsSlice = (set: any, get: any): ModelsSlice => ({
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

const createChatsSlice = (set: any, get: any): ChatsSlice => ({
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

const createChatSlice = (set: any, get: any): ChatSlice => ({
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
        set((state: ChatSlice) => ({
            messages: [...state.messages, message],
        })),
    updateMessage: (messageId, updates) =>
        set((state: ChatSlice) => ({
            messages: state.messages.map((msg) => (msg.id === messageId ? { ...msg, ...updates } : msg)),
        })),
    removeMessage: (messageId) =>
        set((state: ChatSlice) => ({
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

const createFeaturesSlice = (set: any, get: any): FeaturesSlice => ({
    // Initial state
    features: {},
    featuresLoading: false,
    featuresError: null,

    // Actions
    setFeatures: (features) => set({ features }),
    setFeature: (feature, enabled) =>
        set((state: FeaturesSlice) => ({
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

const createUserApiKeysSlice = (set: any, get: any): UserApiKeysSlice => ({
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
// Combined Store
// ============================================================================

type AppStore = AuthSlice & ModelsSlice & ChatsSlice & ChatSlice & UserApiKeysSlice & FeaturesSlice;

export const useAppStore = create<AppStore>()(
    devtools(
        persist(
            (set, get) => ({
                ...createAuthSlice(set, get),
                ...createModelsSlice(set, get),
                ...createChatsSlice(set, get),
                ...createChatSlice(set, get),
                ...createUserApiKeysSlice(set, get),
                ...createFeaturesSlice(set, get),
            }),
            {
                name: "t3chat-store",
                partialize: (state) => ({
                    // Only persist non-sensitive data
                    selectedModel: state.selectedModel,
                    webSearchEnabled: state.webSearchEnabled,
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
