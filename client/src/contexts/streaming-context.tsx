import { createContext, useContext, useState, ReactNode } from "react";

interface StreamingContextValue {
    streamingChatId: string | null;
    setStreamingChatId: (chatId: string | null) => void;
}

const StreamingContext = createContext<StreamingContextValue | undefined>(undefined);

export function StreamingProvider({ children }: { children: ReactNode }) {
    const [streamingChatId, setStreamingChatId] = useState<string | null>(null);

    return (
        <StreamingContext.Provider value={{ streamingChatId, setStreamingChatId }}>
            {children}
        </StreamingContext.Provider>
    );
}

export function useStreamingContext() {
    const context = useContext(StreamingContext);
    if (context === undefined) {
        throw new Error("useStreamingContext must be used within a StreamingProvider");
    }
    return context;
}

