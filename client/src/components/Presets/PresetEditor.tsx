// ============================================================================
// PresetEditor Component - Phase 1B
// ============================================================================
// Dialog for creating/editing presets

import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { EndpointSelector } from "@/components/Endpoints/EndpointSelector";
import { EndpointSettings } from "@/components/Endpoints/EndpointSettings";
import { ModelSelector } from "@/components/model/ModelSelector";
import { usePresets, useModels } from "@/stores/appStore";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "sonner";
import type { EndpointOption, Endpoint } from "@/types/librechat";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

// ============================================================================
// Props
// ============================================================================

export interface PresetEditorProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    presetId?: string; // If provided, edit mode; otherwise create mode
}

// ============================================================================
// Component
// ============================================================================

export function PresetEditor({ open, onOpenChange, presetId }: PresetEditorProps) {
    const { presets, addPreset, updatePreset } = usePresets();
    const { models, fetchModels } = useModels();
    const isEditMode = !!presetId;
    const existingPreset = presets.find((p) => p.id === presetId);

    // Form state
    const [title, setTitle] = useState("");
    const [isDefault, setIsDefault] = useState(false);
    const [endpointOptions, setEndpointOptions] = useState<EndpointOption>({
        endpoint: "openai",
        model: "gpt-4-turbo",
        parameters: {
            temperature: 0.7,
            max_tokens: 2048,
        },
    });
    const [saving, setSaving] = useState(false);

    // Load existing preset data
    useEffect(() => {
        if (isEditMode && existingPreset) {
            setTitle(existingPreset.title);
            setIsDefault(existingPreset.isDefault);
            setEndpointOptions({
                endpoint: existingPreset.endpoint,
                model: existingPreset.model,
                modelLabel: existingPreset.modelLabel,
                parameters: existingPreset.modelParameters || {},
                featureFlags: existingPreset.featureFlags,
                systemMessage: existingPreset.systemMessage,
            });
        } else {
            // Reset for create mode
            setTitle("");
            setIsDefault(false);
            setEndpointOptions({
                endpoint: "openai",
                model: "gpt-4-turbo",
                parameters: {
                    temperature: 0.7,
                    max_tokens: 2048,
                },
            });
        }
    }, [isEditMode, existingPreset, open]);

    // Fetch models on mount
    useEffect(() => {
        if (open && models.length === 0) {
            fetchModels();
        }
    }, [open, models.length, fetchModels]);

    const handleSave = async () => {
        if (!title.trim()) {
            toast.error("Please enter a preset title");
            return;
        }

        setSaving(true);
        try {
            if (isEditMode && presetId) {
                // Update existing preset
                const updated = await t3ChatClient.presets.update(presetId, {
                    title,
                    isDefault,
                    endpoint: endpointOptions.endpoint,
                    model: endpointOptions.model,
                    modelLabel: endpointOptions.modelLabel,
                    modelParameters: endpointOptions.parameters,
                    featureFlags: endpointOptions.featureFlags,
                    systemMessage: endpointOptions.systemMessage,
                });
                updatePreset(presetId, updated);
                toast.success("Preset updated successfully");
            } else {
                // Create new preset
                const created = await t3ChatClient.presets.create({
                    title,
                    isDefault,
                    endpoint: endpointOptions.endpoint,
                    model: endpointOptions.model,
                    modelLabel: endpointOptions.modelLabel,
                    modelParameters: endpointOptions.parameters,
                    featureFlags: endpointOptions.featureFlags,
                    systemMessage: endpointOptions.systemMessage,
                });
                addPreset(created);
                toast.success("Preset created successfully");
            }
            onOpenChange(false);
        } catch (error) {
            console.error("Failed to save preset:", error);
            toast.error(isEditMode ? "Failed to update preset" : "Failed to create preset");
        } finally {
            setSaving(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{isEditMode ? "Edit Preset" : "Create Preset"}</DialogTitle>
                    <DialogDescription>{isEditMode ? "Update your preset configuration" : "Save your current configuration as a preset for quick access"}</DialogDescription>
                </DialogHeader>

                <div className="space-y-4">
                    {/* Preset Name */}
                    <div className="space-y-2">
                        <Label htmlFor="preset-name">Preset Name</Label>
                        <Input id="preset-name" placeholder="My GPT-4 Preset" value={title} onChange={(e) => setTitle(e.target.value)} />
                    </div>

                    {/* Set as Default */}
                    <div className="flex items-center justify-between">
                        <div className="space-y-0.5">
                            <Label htmlFor="default-preset">Set as Default</Label>
                            <p className="text-xs text-muted-foreground">Use this preset for new chats</p>
                        </div>
                        <Switch id="default-preset" checked={isDefault} onCheckedChange={setIsDefault} />
                    </div>

                    {/* Tabs for Basic and Advanced settings */}
                    <Tabs defaultValue="basic" className="w-full">
                        <TabsList className="grid w-full grid-cols-2">
                            <TabsTrigger value="basic">Basic</TabsTrigger>
                            <TabsTrigger value="advanced">Advanced</TabsTrigger>
                        </TabsList>

                        <TabsContent value="basic" className="space-y-4">
                            {/* Endpoint Selector */}
                            <div className="space-y-2">
                                <Label>AI Provider</Label>
                                <EndpointSelector value={endpointOptions.endpoint} onChange={(endpoint: Endpoint) => setEndpointOptions({ ...endpointOptions, endpoint })} className="w-full" />
                            </div>

                            {/* Model Selector */}
                            <div className="space-y-2">
                                <Label>Model</Label>
                                <ModelSelector
                                    models={models.filter((m) => m.provider === (endpointOptions.endpoint as string))}
                                    selectedModel={models.find((m) => m.modelId === endpointOptions.model) || null}
                                    onSelect={(model) =>
                                        setEndpointOptions({
                                            ...endpointOptions,
                                            model: model.modelId,
                                            modelLabel: model.displayName,
                                        })
                                    }
                                />
                            </div>
                        </TabsContent>

                        <TabsContent value="advanced" className="max-h-[400px] overflow-y-auto">
                            <EndpointSettings options={endpointOptions} onChange={setEndpointOptions} />
                        </TabsContent>
                    </Tabs>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
                        Cancel
                    </Button>
                    <Button onClick={handleSave} disabled={saving}>
                        {saving ? "Saving..." : isEditMode ? "Update" : "Create"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
