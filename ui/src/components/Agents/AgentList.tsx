// ============================================================================
// AgentList Component - Phase 1B
// ============================================================================
// List of AI agents (system and user-created)

import { useEffect } from "react";
import { useAgents } from "@/stores/appStore";
import { AgentCard } from "./AgentCard";
import { Button } from "@/components/ui/button";
import { PlusIcon } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

// ============================================================================
// Props
// ============================================================================

export interface AgentListProps {
    onSelect?: (agentId: string) => void;
    onEdit?: (agentId: string) => void;
    onDelete?: (agentId: string) => void;
    onCreate?: () => void;
    selectedAgentId?: string;
}

// ============================================================================
// Component
// ============================================================================

export function AgentList({ onSelect, onEdit, onDelete, onCreate, selectedAgentId }: AgentListProps) {
    const { agents, loading, error, fetchAgents } = useAgents();

    useEffect(() => {
        fetchAgents();
    }, [fetchAgents]);

    if (loading) {
        return (
            <div className="space-y-2 p-4">
                <Skeleton className="h-24 w-full" />
                <Skeleton className="h-24 w-full" />
                <Skeleton className="h-24 w-full" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="p-4 text-center text-sm text-destructive">
                <p>Failed to load agents</p>
                <p className="text-xs text-muted-foreground">{error.message}</p>
            </div>
        );
    }

    // Categorize agents by access level
    const myAgents = agents.filter((a) => a.accessLevel === 0);
    const sharedAgents = agents.filter((a) => a.accessLevel === 1);
    const publicAgents = agents.filter((a) => a.accessLevel === 2);

    const AgentGrid = ({ agents: agentList }: { agents: typeof agents }) =>
        agentList.length === 0 ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
                <p>No agents in this category</p>
            </div>
        ) : (
            <div className="grid gap-4 p-4 sm:grid-cols-2 lg:grid-cols-3">
                {agentList.map((agent) => (
                    <AgentCard
                        key={agent.id}
                        agent={agent}
                        isSelected={selectedAgentId === agent.id}
                        onSelect={() => onSelect?.(agent.id)}
                        onEdit={() => onEdit?.(agent.id)}
                        onDelete={() => onDelete?.(agent.id)}
                    />
                ))}
            </div>
        );

    return (
        <div className="flex h-full flex-col">
            {/* Header with Create button */}
            <div className="border-b p-4">
                <Button onClick={onCreate} className="w-full" variant="outline">
                    <PlusIcon className="mr-2 h-4 w-4" />
                    Create Agent
                </Button>
            </div>

            {/* Agents list with tabs */}
            <div className="flex-1 overflow-hidden">
                <Tabs defaultValue="my" className="flex h-full flex-col">
                    <TabsList className="grid w-full grid-cols-3">
                        <TabsTrigger value="my">
                            My Agents
                            {myAgents.length > 0 && (
                                <span className="ml-2 rounded-full bg-primary px-2 py-0.5 text-xs text-primary-foreground">
                                    {myAgents.length}
                                </span>
                            )}
                        </TabsTrigger>
                        <TabsTrigger value="shared">
                            Shared
                            {sharedAgents.length > 0 && (
                                <span className="ml-2 rounded-full bg-primary px-2 py-0.5 text-xs text-primary-foreground">
                                    {sharedAgents.length}
                                </span>
                            )}
                        </TabsTrigger>
                        <TabsTrigger value="public">
                            Public
                            {publicAgents.length > 0 && (
                                <span className="ml-2 rounded-full bg-primary px-2 py-0.5 text-xs text-primary-foreground">
                                    {publicAgents.length}
                                </span>
                            )}
                        </TabsTrigger>
                    </TabsList>

                    <div className="flex-1 overflow-y-auto">
                        <TabsContent value="my" className="m-0">
                            <AgentGrid agents={myAgents} />
                        </TabsContent>
                        <TabsContent value="shared" className="m-0">
                            <AgentGrid agents={sharedAgents} />
                        </TabsContent>
                        <TabsContent value="public" className="m-0">
                            <AgentGrid agents={publicAgents} />
                        </TabsContent>
                    </div>
                </Tabs>
            </div>
        </div>
    );
}

