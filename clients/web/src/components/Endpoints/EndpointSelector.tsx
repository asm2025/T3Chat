// ============================================================================
// EndpointSelector Component - Phase 1B
// ============================================================================
// Dropdown to select AI provider (OpenAI, Anthropic, Google, etc.)

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { Endpoint } from "@/types/librechat";

// ============================================================================
// Endpoint Options
// ============================================================================

interface EndpointConfig {
    value: Endpoint;
    label: string;
    icon: string;
    description: string;
}

const ENDPOINTS: EndpointConfig[] = [
    {
        value: "openai",
        label: "OpenAI",
        icon: "🤖",
        description: "GPT-4, GPT-3.5, and other OpenAI models",
    },
    {
        value: "anthropic",
        label: "Anthropic",
        icon: "🧠",
        description: "Claude 3 Opus, Sonnet, and Haiku models",
    },
    {
        value: "google",
        label: "Google",
        icon: "✨",
        description: "Gemini models",
    },
    {
        value: "custom",
        label: "Custom",
        icon: "⚙️",
        description: "OpenAI-compatible endpoints",
    },
    {
        value: "bedrock",
        label: "AWS Bedrock",
        icon: "☁️",
        description: "AWS Bedrock models",
    },
];

// ============================================================================
// Props
// ============================================================================

export interface EndpointSelectorProps {
    value: Endpoint;
    onChange: (endpoint: Endpoint) => void;
    disabled?: boolean;
    className?: string;
}

// ============================================================================
// Component
// ============================================================================

export function EndpointSelector({ value, onChange, disabled = false, className }: EndpointSelectorProps) {
    const selectedEndpoint = ENDPOINTS.find((e) => e.value === value);

    return (
        <Select value={value} onValueChange={onChange} disabled={disabled}>
            <SelectTrigger className={className || "w-[200px]"}>
                <SelectValue placeholder="Select AI Provider">
                    {selectedEndpoint && (
                        <span className="flex items-center gap-2">
                            <span>{selectedEndpoint.icon}</span>
                            <span>{selectedEndpoint.label}</span>
                        </span>
                    )}
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                {ENDPOINTS.map((endpoint) => (
                    <SelectItem key={endpoint.value} value={endpoint.value}>
                        <div className="flex items-start gap-2">
                            <span className="text-lg">{endpoint.icon}</span>
                            <div className="flex flex-col">
                                <span className="font-medium">{endpoint.label}</span>
                                <span className="text-xs text-muted-foreground">{endpoint.description}</span>
                            </div>
                        </div>
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}

// ============================================================================
// Export endpoint configurations for use in other components
// ============================================================================

export { ENDPOINTS };
export type { EndpointConfig };

