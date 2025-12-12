// ============================================================================
// AgentCard Component - Phase 1B
// ============================================================================
// Card displaying agent information

import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Edit2Icon, Trash2Icon, LockIcon, UsersIcon, GlobeIcon } from "lucide-react";
import type { Agent } from "@/types/librechat";
import { cn } from "@/lib/utils";
import { ENDPOINTS } from "@/components/Endpoints";

// ============================================================================
// Props
// ============================================================================

export interface AgentCardProps {
    agent: Agent;
    isSelected?: boolean;
    onSelect?: () => void;
    onEdit?: () => void;
    onDelete?: () => void;
}

// ============================================================================
// Component
// ============================================================================

export function AgentCard({ agent, isSelected = false, onSelect, onEdit, onDelete }: AgentCardProps) {
    const endpointConfig = ENDPOINTS.find((e) => e.value === agent.provider);

    // Access level icon
    const AccessLevelIcon = agent.accessLevel === 0 ? LockIcon : agent.accessLevel === 1 ? UsersIcon : GlobeIcon;
    const accessLevelLabel = agent.accessLevel === 0 ? "Private" : agent.accessLevel === 1 ? "Shared" : "Public";

    return (
        <Card
            className={cn(
                "group relative flex cursor-pointer flex-col p-4 transition-all hover:border-primary",
                isSelected && "border-primary bg-primary/5",
            )}
            onClick={onSelect}
        >
            {/* Avatar and Title */}
            <div className="mb-3 flex items-start gap-3">
                <Avatar className="h-12 w-12">
                    <AvatarImage src={agent.avatarFilepath} />
                    <AvatarFallback className="bg-primary/10 text-primary">
                        {agent.name.slice(0, 2).toUpperCase()}
                    </AvatarFallback>
                </Avatar>
                <div className="flex-1 min-w-0">
                    <h3 className="mb-1 truncate font-semibold">{agent.name}</h3>
                    <div className="flex items-center gap-1 text-xs text-muted-foreground">
                        <AccessLevelIcon className="h-3 w-3" />
                        <span>{accessLevelLabel}</span>
                    </div>
                </div>
            </div>

            {/* Description */}
            {agent.description && (
                <p className="mb-3 line-clamp-2 text-sm text-muted-foreground">{agent.description}</p>
            )}

            {/* Model Info */}
            <div className="mb-3 flex items-center gap-2 text-xs">
                <span>{endpointConfig?.icon}</span>
                <span className="text-muted-foreground">
                    {endpointConfig?.label} - {agent.model}
                </span>
            </div>

            {/* Features */}
            <div className="mb-3 flex flex-wrap gap-1.5">
                {agent.isCollaborative && <Badge variant="secondary" className="text-xs">Collaborative</Badge>}
                {agent.endAfterTools && <Badge variant="secondary" className="text-xs">Auto-end</Badge>}
                {agent.hideSequentialOutputs && <Badge variant="secondary" className="text-xs">Hide outputs</Badge>}
            </div>

            {/* Actions */}
            <div className="mt-auto flex gap-2 border-t pt-3 opacity-0 transition-opacity group-hover:opacity-100">
                <Button
                    size="sm"
                    variant="ghost"
                    className="flex-1"
                    onClick={(e) => {
                        e.stopPropagation();
                        onEdit?.();
                    }}
                >
                    <Edit2Icon className="mr-1 h-3 w-3" />
                    Edit
                </Button>
                {agent.accessLevel === 0 && (
                    <Button
                        size="sm"
                        variant="ghost"
                        onClick={(e) => {
                            e.stopPropagation();
                            onDelete?.();
                        }}
                        className="text-destructive hover:text-destructive"
                    >
                        <Trash2Icon className="h-3 w-3" />
                    </Button>
                )}
            </div>
        </Card>
    );
}

