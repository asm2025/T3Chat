// ============================================================================
// AgentEditor Component - Phase 1B
// ============================================================================
// Dialog for creating/editing agents

import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { EndpointSelector } from "@/components/Endpoints/EndpointSelector";
import { ModelSelector } from "@/components/model/ModelSelector";
import { useAgents, useModels } from "@/stores/appStore";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "sonner";
import type { Endpoint } from "@/types/librechat";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

// ============================================================================
// Props
// ============================================================================

export interface AgentEditorProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    agentId?: string; // If provided, edit mode; otherwise create mode
}

// ============================================================================
// Component
// ============================================================================

export function AgentEditor({ open, onOpenChange, agentId }: AgentEditorProps) {
    const { agents, addAgent, updateAgent } = useAgents();
    const { models } = useModels();
    const isEditMode = !!agentId;
    const existingAgent = agents.find((a) => a.id === agentId);

    // Form state
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [instructions, setInstructions] = useState("");
    const [provider, setProvider] = useState<Endpoint>("openai");
    const [model, setModel] = useState("gpt-4-turbo");
    const [accessLevel, setAccessLevel] = useState(0);
    const [isCollaborative, setIsCollaborative] = useState(false);
    const [hideSequentialOutputs, setHideSequentialOutputs] = useState(false);
    const [endAfterTools, setEndAfterTools] = useState(false);
    const [recursionLimit, setRecursionLimit] = useState(5);
    const [saving, setSaving] = useState(false);

    // Load existing agent data
    useEffect(() => {
        if (isEditMode && existingAgent) {
            setName(existingAgent.name);
            setDescription(existingAgent.description || "");
            setInstructions(existingAgent.instructions || "");
            setProvider(existingAgent.provider as Endpoint);
            setModel(existingAgent.model);
            setAccessLevel(existingAgent.accessLevel);
            setIsCollaborative(existingAgent.isCollaborative);
            setHideSequentialOutputs(existingAgent.hideSequentialOutputs);
            setEndAfterTools(existingAgent.endAfterTools);
            setRecursionLimit(existingAgent.recursionLimit);
        } else {
            // Reset for create mode
            setName("");
            setDescription("");
            setInstructions("");
            setProvider("openai");
            setModel("gpt-4-turbo");
            setAccessLevel(0);
            setIsCollaborative(false);
            setHideSequentialOutputs(false);
            setEndAfterTools(false);
            setRecursionLimit(5);
        }
    }, [isEditMode, existingAgent, open]);

    const handleSave = async () => {
        if (!name.trim()) {
            toast.error("Please enter an agent name");
            return;
        }

        setSaving(true);
        try {
            if (isEditMode && agentId) {
                // Update existing agent
                const updated = await t3ChatClient.agents.update(agentId, {
                    name,
                    description,
                    instructions,
                    provider,
                    model,
                    accessLevel,
                    isCollaborative,
                    hideSequentialOutputs,
                    endAfterTools,
                    recursionLimit,
                });
                updateAgent(agentId, updated);
                toast.success("Agent updated successfully");
            } else {
                // Create new agent
                const created = await t3ChatClient.agents.create({
                    name,
                    description,
                    instructions,
                    provider,
                    model,
                    accessLevel,
                    isCollaborative,
                    hideSequentialOutputs,
                    endAfterTools,
                    recursionLimit,
                });
                addAgent(created);
                toast.success("Agent created successfully");
            }
            onOpenChange(false);
        } catch (error) {
            console.error("Failed to save agent:", error);
            toast.error(isEditMode ? "Failed to update agent" : "Failed to create agent");
        } finally {
            setSaving(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{isEditMode ? "Edit Agent" : "Create Agent"}</DialogTitle>
                    <DialogDescription>
                        {isEditMode ? "Update your agent configuration" : "Create a new AI agent with custom behavior"}
                    </DialogDescription>
                </DialogHeader>

                <Tabs defaultValue="basic" className="w-full">
                    <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="basic">Basic</TabsTrigger>
                        <TabsTrigger value="advanced">Advanced</TabsTrigger>
                    </TabsList>

                    <TabsContent value="basic" className="space-y-4">
                        {/* Agent Name */}
                        <div className="space-y-2">
                            <Label htmlFor="agent-name">Agent Name</Label>
                            <Input id="agent-name" placeholder="My Research Assistant" value={name} onChange={(e) => setName(e.target.value)} />
                        </div>

                        {/* Description */}
                        <div className="space-y-2">
                            <Label htmlFor="agent-description">Description</Label>
                            <Textarea
                                id="agent-description"
                                placeholder="Describe what this agent does..."
                                value={description}
                                onChange={(e) => setDescription(e.target.value)}
                                className="min-h-[80px] resize-none"
                            />
                        </div>

                        {/* Instructions */}
                        <div className="space-y-2">
                            <Label htmlFor="agent-instructions">System Instructions</Label>
                            <Textarea
                                id="agent-instructions"
                                placeholder="You are a helpful research assistant who..."
                                value={instructions}
                                onChange={(e) => setInstructions(e.target.value)}
                                className="min-h-[120px] resize-none"
                            />
                        </div>

                        {/* Provider */}
                        <div className="space-y-2">
                            <Label>AI Provider</Label>
                            <EndpointSelector value={provider} onChange={setProvider} className="w-full" />
                        </div>

                        {/* Model */}
                        <div className="space-y-2">
                            <Label>Model</Label>
                            <ModelSelector
                                models={models.filter((m) => m.provider === provider)}
                                selectedModel={models.find((m) => m.id === model) ?? null}
                                onSelect={(m) => setModel(m.id)}
                            />
                        </div>
                    </TabsContent>

                    <TabsContent value="advanced" className="space-y-4">
                        {/* Access Level */}
                        <div className="space-y-2">
                            <Label htmlFor="access-level">Access Level</Label>
                            <Select value={String(accessLevel)} onValueChange={(value) => setAccessLevel(parseInt(value))}>
                                <SelectTrigger id="access-level">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="0">🔒 Private (Only me)</SelectItem>
                                    <SelectItem value="1">👥 Shared (Selected users)</SelectItem>
                                    <SelectItem value="2">🌐 Public (Everyone)</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>

                        {/* Recursion Limit */}
                        <div className="space-y-2">
                            <Label htmlFor="recursion-limit">Recursion Limit</Label>
                            <Input
                                id="recursion-limit"
                                type="number"
                                min={1}
                                max={20}
                                value={recursionLimit}
                                onChange={(e) => setRecursionLimit(parseInt(e.target.value) || 5)}
                            />
                            <p className="text-xs text-muted-foreground">Maximum depth for agent-to-agent calls</p>
                        </div>

                        {/* Collaborative Mode */}
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label htmlFor="collaborative">Collaborative Mode</Label>
                                <p className="text-xs text-muted-foreground">Allow multiple agents to work together</p>
                            </div>
                            <Switch id="collaborative" checked={isCollaborative} onCheckedChange={setIsCollaborative} />
                        </div>

                        {/* Hide Sequential Outputs */}
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label htmlFor="hide-outputs">Hide Sequential Outputs</Label>
                                <p className="text-xs text-muted-foreground">Hide intermediate tool outputs</p>
                            </div>
                            <Switch id="hide-outputs" checked={hideSequentialOutputs} onCheckedChange={setHideSequentialOutputs} />
                        </div>

                        {/* End After Tools */}
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label htmlFor="end-after-tools">End After Tools</Label>
                                <p className="text-xs text-muted-foreground">Automatically finish after tool execution</p>
                            </div>
                            <Switch id="end-after-tools" checked={endAfterTools} onCheckedChange={setEndAfterTools} />
                        </div>
                    </TabsContent>
                </Tabs>

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

