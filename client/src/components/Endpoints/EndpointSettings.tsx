// ============================================================================
// EndpointSettings Component - Phase 1B
// ============================================================================
// Settings panel for AI provider parameters (temperature, tokens, etc.)

import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import type { EndpointOption, ModelParameters, FeatureFlags } from "@/types/librechat";
import { Separator } from "@/components/ui/separator";

// ============================================================================
// Props
// ============================================================================

export interface EndpointSettingsProps {
    options: EndpointOption;
    onChange: (options: EndpointOption) => void;
    className?: string;
}

// ============================================================================
// Component
// ============================================================================

export function EndpointSettings({ options, onChange }: EndpointSettingsProps) {
    const { endpoint, parameters = {}, featureFlags = {}, systemMessage } = options;

    // Helper to update parameters
    const updateParameter = <K extends keyof ModelParameters>(key: K, value: ModelParameters[K]) => {
        onChange({
            ...options,
            parameters: {
                ...parameters,
                [key]: value,
            },
        });
    };

    // Helper to update feature flags
    const updateFeatureFlag = <K extends keyof FeatureFlags>(key: K, value: FeatureFlags[K]) => {
        onChange({
            ...options,
            featureFlags: {
                ...featureFlags,
                [key]: value,
            },
        });
    };

    // Helper to update system message
    const updateSystemMessage = (value: string) => {
        onChange({
            ...options,
            systemMessage: value,
        });
    };

    return (
        <div className="space-y-6 p-4">
            {/* System Message */}
            <div className="space-y-2">
                <Label htmlFor="system-message">System Message</Label>
                <Textarea
                    id="system-message"
                    placeholder="Enter system instructions for the AI..."
                    value={systemMessage || ""}
                    onChange={(e) => updateSystemMessage(e.target.value)}
                    className="min-h-[100px] resize-none"
                />
                <p className="text-xs text-muted-foreground">
                    System instructions that guide the AI's behavior and responses
                </p>
            </div>

            <Separator />

            {/* Model Parameters */}
            <div className="space-y-4">
                <h3 className="text-sm font-semibold">Model Parameters</h3>

                {/* Temperature */}
                <div className="space-y-2">
                    <div className="flex items-center justify-between">
                        <Label htmlFor="temperature">Temperature</Label>
                        <span className="text-sm text-muted-foreground">{(parameters.temperature ?? 0.7).toFixed(2)}</span>
                    </div>
                    <Slider
                        id="temperature"
                        value={[parameters.temperature ?? 0.7]}
                        onValueChange={([value]: number[]) => updateParameter("temperature", value)}
                        min={0}
                        max={2}
                        step={0.01}
                        className="w-full"
                    />
                    <p className="text-xs text-muted-foreground">Controls randomness. Lower is more focused and deterministic.</p>
                </div>

                {/* Max Tokens */}
                <div className="space-y-2">
                    <Label htmlFor="max-tokens">Max Tokens</Label>
                    <Input
                        id="max-tokens"
                        type="number"
                        min={1}
                        max={128000}
                        value={parameters.max_tokens ?? 2048}
                        onChange={(e) => updateParameter("max_tokens", parseInt(e.target.value) || 2048)}
                    />
                    <p className="text-xs text-muted-foreground">Maximum length of the response</p>
                </div>

                {/* OpenAI-specific parameters */}
                {endpoint === "openai" && (
                    <>
                        {/* Top P */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <Label htmlFor="top-p">Top P</Label>
                                <span className="text-sm text-muted-foreground">{(parameters.top_p ?? 1).toFixed(2)}</span>
                            </div>
                            <Slider
                                id="top-p"
                                value={[parameters.top_p ?? 1]}
                                onValueChange={([value]: number[]) => updateParameter("top_p", value)}
                                min={0}
                                max={1}
                                step={0.01}
                                className="w-full"
                            />
                            <p className="text-xs text-muted-foreground">Nucleus sampling threshold</p>
                        </div>

                        {/* Frequency Penalty */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <Label htmlFor="frequency-penalty">Frequency Penalty</Label>
                                <span className="text-sm text-muted-foreground">
                                    {(parameters.frequency_penalty ?? 0).toFixed(2)}
                                </span>
                            </div>
                            <Slider
                                id="frequency-penalty"
                                value={[parameters.frequency_penalty ?? 0]}
                                onValueChange={([value]: number[]) => updateParameter("frequency_penalty", value)}
                                min={-2}
                                max={2}
                                step={0.01}
                                className="w-full"
                            />
                            <p className="text-xs text-muted-foreground">Penalize repeated tokens</p>
                        </div>

                        {/* Presence Penalty */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <Label htmlFor="presence-penalty">Presence Penalty</Label>
                                <span className="text-sm text-muted-foreground">
                                    {(parameters.presence_penalty ?? 0).toFixed(2)}
                                </span>
                            </div>
                            <Slider
                                id="presence-penalty"
                                value={[parameters.presence_penalty ?? 0]}
                                onValueChange={([value]: number[]) => updateParameter("presence_penalty", value)}
                                min={-2}
                                max={2}
                                step={0.01}
                                className="w-full"
                            />
                            <p className="text-xs text-muted-foreground">Penalize new topics</p>
                        </div>
                    </>
                )}

                {/* Google/Anthropic-specific parameters */}
                {(endpoint === "google" || endpoint === "anthropic") && (
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <Label htmlFor="top-k">Top K</Label>
                            <span className="text-sm text-muted-foreground">{parameters.top_k ?? 40}</span>
                        </div>
                        <Slider
                            id="top-k"
                            value={[parameters.top_k ?? 40]}
                            onValueChange={([value]: number[]) => updateParameter("top_k", value)}
                            min={1}
                            max={100}
                            step={1}
                            className="w-full"
                        />
                        <p className="text-xs text-muted-foreground">Limit token selection to top K tokens</p>
                    </div>
                )}
            </div>

            <Separator />

            {/* Feature Flags */}
            <div className="space-y-4">
                <h3 className="text-sm font-semibold">Advanced Features</h3>

                {/* Resend Files */}
                <div className="flex items-center justify-between">
                    <div className="space-y-0.5">
                        <Label htmlFor="resend-files">Resend Files</Label>
                        <p className="text-xs text-muted-foreground">Include file attachments in every message</p>
                    </div>
                    <Switch
                        id="resend-files"
                        checked={featureFlags.resend_files ?? false}
                        onCheckedChange={(checked) => updateFeatureFlag("resend_files", checked)}
                    />
                </div>

                {/* Resend Images */}
                <div className="flex items-center justify-between">
                    <div className="space-y-0.5">
                        <Label htmlFor="resend-images">Resend Images</Label>
                        <p className="text-xs text-muted-foreground">Include images in conversation context</p>
                    </div>
                    <Switch
                        id="resend-images"
                        checked={featureFlags.resend_images ?? false}
                        onCheckedChange={(checked) => updateFeatureFlag("resend_images", checked)}
                    />
                </div>

                {/* Prompt Cache (if supported) */}
                {endpoint === "anthropic" && (
                    <div className="flex items-center justify-between">
                        <div className="space-y-0.5">
                            <Label htmlFor="prompt-cache">Prompt Caching</Label>
                            <p className="text-xs text-muted-foreground">
                                Cache prompts for faster responses (Anthropic feature)
                            </p>
                        </div>
                        <Switch
                            id="prompt-cache"
                            checked={featureFlags.prompt_cache ?? false}
                            onCheckedChange={(checked) => updateFeatureFlag("prompt_cache", checked)}
                        />
                    </div>
                )}

                {/* Thinking (Extended Thinking for OpenAI) */}
                {endpoint === "openai" && (
                    <>
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label htmlFor="thinking">Extended Thinking</Label>
                                <p className="text-xs text-muted-foreground">Enable extended reasoning (o1/o3 models)</p>
                            </div>
                            <Switch
                                id="thinking"
                                checked={featureFlags.thinking ?? false}
                                onCheckedChange={(checked) => updateFeatureFlag("thinking", checked)}
                            />
                        </div>

                        {featureFlags.thinking && (
                            <div className="space-y-2 pl-4">
                                <Label htmlFor="thinking-budget">Thinking Budget (tokens)</Label>
                                <Input
                                    id="thinking-budget"
                                    type="number"
                                    min={1000}
                                    max={100000}
                                    value={featureFlags.thinking_budget ?? 10000}
                                    onChange={(e) => updateFeatureFlag("thinking_budget", parseInt(e.target.value) || 10000)}
                                />
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
}

