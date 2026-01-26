import { useCallback, useState } from "react";
import type { AgentEvent, ToolExecutionState } from "@/types/agent-events";

export function useAgentEvents() {
    const [thinkingMessage, setThinkingMessage] = useState<string | null>(null);
    const [toolExecutions, setToolExecutions] = useState<Map<string, ToolExecutionState>>(new Map());
    const [textContent, setTextContent] = useState<string>("");

    const handleEvent = useCallback((event: AgentEvent) => {
        switch (event.type) {
            case "thinking":
                setThinkingMessage(event.message);
                break;
            case "tool_start":
                setToolExecutions((prev) => {
                    const next = new Map(prev);
                    next.set(event.tool_call_id, {
                        tool_call_id: event.tool_call_id,
                        tool_name: event.tool_name,
                        tool_type: event.tool_type,
                        status: "running",
                        arguments: event.arguments,
                        startedAt: new Date(),
                    });
                    return next;
                });
                break;
            case "tool_output":
                setToolExecutions((prev) => {
                    const next = new Map(prev);
                    const existing = next.get(event.tool_call_id);
                    if (!existing) {
                        return prev;
                    }

                    const shouldAppend = event.is_partial && typeof event.output === "string" && typeof existing.output === "string";
                    next.set(event.tool_call_id, {
                        ...existing,
                        output: shouldAppend ? `${existing.output}${event.output}` : event.output,
                    });
                    return next;
                });
                break;
            case "tool_end":
                setToolExecutions((prev) => {
                    const next = new Map(prev);
                    const existing = next.get(event.tool_call_id);
                    if (!existing) {
                        return prev;
                    }
                    next.set(event.tool_call_id, {
                        ...existing,
                        status: event.status,
                        error: event.error,
                        output: event.result ?? existing.output,
                        completedAt: new Date(),
                    });
                    return next;
                });
                break;
            case "text_delta":
                setTextContent((prev) => prev + event.delta);
                break;
            case "message_complete":
                setTextContent(event.content);
                setThinkingMessage(null);
                break;
            case "error":
                break;
        }
    }, []);

    const reset = useCallback(() => {
        setThinkingMessage(null);
        setToolExecutions(new Map());
        setTextContent("");
    }, []);

    return {
        thinkingMessage,
        toolExecutions: Array.from(toolExecutions.values()),
        textContent,
        handleEvent,
        reset,
    };
}
