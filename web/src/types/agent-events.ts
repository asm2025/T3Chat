export type AgentEventType =
    | "thinking"
    | "tool_start"
    | "tool_output"
    | "tool_end"
    | "text_delta"
    | "message_complete"
    | "error";

export interface BaseAgentEvent {
    type: AgentEventType;
    timestamp?: string;
}

export interface ThinkingEvent extends BaseAgentEvent {
    type: "thinking";
    message: string;
    metadata?: Record<string, unknown>;
}

export interface ToolStartEvent extends BaseAgentEvent {
    type: "tool_start";
    tool_call_id: string;
    tool_name: string;
    tool_type: string;
    arguments: Record<string, unknown>;
    metadata?: Record<string, unknown>;
}

export interface ToolOutputEvent extends BaseAgentEvent {
    type: "tool_output";
    tool_call_id: string;
    tool_name: string;
    output: unknown;
    is_partial?: boolean;
}

export interface ToolEndEvent extends BaseAgentEvent {
    type: "tool_end";
    tool_call_id: string;
    tool_name: string;
    status: "completed" | "failed";
    error?: string;
    result?: unknown;
}

export interface TextDeltaEvent extends BaseAgentEvent {
    type: "text_delta";
    delta: string;
}

export interface MessageCompleteEvent extends BaseAgentEvent {
    type: "message_complete";
    content: string;
    finish_reason?: string;
    usage?: {
        prompt_tokens: number;
        completion_tokens: number;
        total_tokens: number;
    };
}

export interface ErrorEvent extends BaseAgentEvent {
    type: "error";
    message: string;
    code?: string;
}

export type AgentEvent =
    | ThinkingEvent
    | ToolStartEvent
    | ToolOutputEvent
    | ToolEndEvent
    | TextDeltaEvent
    | MessageCompleteEvent
    | ErrorEvent;

export type ToolExecutionStatus = "running" | "completed" | "failed";

export interface ToolExecutionState {
    tool_call_id: string;
    tool_name: string;
    tool_type: string;
    status: ToolExecutionStatus;
    arguments?: Record<string, unknown>;
    output?: unknown;
    error?: string;
    startedAt: Date;
    completedAt?: Date;
}

export interface AgentEventsSnapshot {
    thinkingMessage?: string | null;
    toolExecutions: ToolExecutionState[];
    textContent: string;
}

// Helper to check if an event is a valid agent event
export const AGENT_EVENT_TYPES: readonly AgentEventType[] = [
    "thinking",
    "tool_start",
    "tool_output",
    "tool_end",
    "text_delta",
    "message_complete",
    "error",
] as const;
