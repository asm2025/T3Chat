import { useState, useCallback } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";
import type { ChatRequest } from "@/types/api";
import type { AgentEvent } from "@/types/agent-events";
import { useAgentEvents } from "./useAgentEvents";

export function useStreamingChat() {
    const [streaming, setStreaming] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const agentEvents = useAgentEvents();

    const sendMessage = useCallback(async (request: ChatRequest, onChunk: (chunk: string, mode?: "append" | "replace") => void, onComplete: () => void, onEvent?: (event: AgentEvent) => void) => {
        try {
            setStreaming(true);
            setError(null);
            agentEvents.reset();

            for await (const event of t3ChatClient.streamChat(request)) {
                agentEvents.handleEvent(event);
                onEvent?.(event);

                if (event.type === "text_delta") {
                    onChunk(event.delta, "append");
                } else if (event.type === "message_complete") {
                    onChunk(event.content, "replace");
                }
            }

            onComplete();
        } catch (err) {
            const error = err instanceof Error ? err : new Error(getErrorMessage(err));
            setError(error);
            toast.error("Failed to send message", {
                description: getErrorMessage(err),
            });
            throw error;
        } finally {
            setStreaming(false);
        }
    }, [agentEvents]);

    return { sendMessage, streaming, error, agentEvents };
}
