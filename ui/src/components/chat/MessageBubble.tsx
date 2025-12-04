import type { Message } from "@/types/chat";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

interface MessageBubbleProps {
    message: Message;
    streaming?: boolean;
}

export function MessageBubble({ message, streaming }: MessageBubbleProps) {
    const isUser = message.role === "user";

    return (
        <div className={cn("flex w-full gap-3", isUser ? "justify-end" : "justify-start")}>
            {!isUser && (
                <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
                    <AvatarFallback className="text-xs font-medium">AI</AvatarFallback>
                </Avatar>
            )}
            <div className={cn("flex max-w-[80%] flex-col", isUser ? "items-end text-right" : "items-start text-left")}>
                <span className="mb-1.5 text-xs uppercase tracking-[0.16em] text-muted-foreground">{isUser ? "You" : "Assistant"}</span>
                <div
                    className={cn(
                        "w-full rounded-xl border px-4 py-3 shadow-sm transition",
                        isUser ? "border-foreground/20 bg-foreground text-background" : "border-border bg-background text-foreground",
                    )}>
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">
                        {message.content}
                        {streaming && <span className="ml-1 inline-block h-4 w-2 animate-pulse bg-current" />}
                    </p>
                    {(message.model_used || message.tokens_used) && (
                        <div className="mt-2 flex items-center gap-2 border-t border-current/10 pt-2 text-xs opacity-70">
                            {message.model_used && <span>{message.model_used}</span>}
                            {message.tokens_used && <span>• {message.tokens_used} tokens</span>}
                        </div>
                    )}
                </div>
            </div>
            {isUser && (
                <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
                    <AvatarFallback className="text-xs font-medium">You</AvatarFallback>
                </Avatar>
            )}
        </div>
    );
}

