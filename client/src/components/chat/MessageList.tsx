import { useEffect, useRef } from "react";
import type { Message } from "@/types/chat";
import { MessageBubble } from "./MessageBubble";
import { ChatPlaceholder } from "./ChatPlaceholder";

interface MessageListProps {
    messages: Message[];
    streaming?: boolean;
    showPlaceholder?: boolean;
    placeholderUserName?: string;
    onPromptClick?: (prompt: string) => void;
}

export function MessageList({ messages, streaming, showPlaceholder = true, placeholderUserName, onPromptClick }: MessageListProps) {
    const messagesEndRef = useRef<HTMLDivElement>(null);
    const scrollContainerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [messages]);

    if (messages.length === 0 && showPlaceholder) {
        return (
            <div className="flex h-full flex-col overflow-hidden">
                <div className="flex-1 overflow-y-auto px-4 py-6 sm:px-8">
                    <ChatPlaceholder userName={placeholderUserName} onPromptClick={onPromptClick} />
                </div>
            </div>
        );
    }

    return (
        <div className="flex h-full flex-col overflow-hidden">
            <div ref={scrollContainerRef} className="flex-1 overflow-y-auto px-4 py-6 sm:px-8 scroll-smooth">
                <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
                    {messages.map((message, index) => {
                        const isLastMessage = index === messages.length - 1;
                        const isStreaming = streaming && isLastMessage;

                        return <MessageBubble key={message.id} message={message} streaming={isStreaming} />;
                    })}

                    {streaming && (
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <div className="flex gap-1">
                                <span className="h-2 w-2 animate-bounce rounded-full bg-current" style={{ animationDelay: "0ms" }} />
                                <span className="h-2 w-2 animate-bounce rounded-full bg-current" style={{ animationDelay: "150ms" }} />
                                <span className="h-2 w-2 animate-bounce rounded-full bg-current" style={{ animationDelay: "300ms" }} />
                            </div>
                            <span>Generating response...</span>
                        </div>
                    )}
                </div>
                <div ref={messagesEndRef} />
            </div>
        </div>
    );
}

