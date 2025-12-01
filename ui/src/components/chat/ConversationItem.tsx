// ============================================================================
// ConversationItem Component - Phase 1B
// ============================================================================
// Individual conversation in the sidebar list

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ArchiveIcon, Edit2Icon, MoreVerticalIcon, Trash2Icon } from "lucide-react";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { ConversationWithTags } from "@/types/librechat";
import { cn } from "@/lib/utils";
import { ENDPOINTS } from "@/components/Endpoints";

// ============================================================================
// Props
// ============================================================================

export interface ConversationItemProps {
    conversation: ConversationWithTags;
    isActive?: boolean;
    onClick?: () => void;
    onEdit?: () => void;
    onDelete?: () => void;
    onArchive?: () => void;
}

// ============================================================================
// Component
// ============================================================================

export function ConversationItem({ conversation, isActive = false, onClick, onEdit, onDelete, onArchive }: ConversationItemProps) {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const endpointConfig = ENDPOINTS.find((e) => e.value === conversation.endpoint);

    // Format date
    const formatDate = (dateString: string): string => {
        const date = new Date(dateString);
        const now = new Date();
        const diffMs = now.getTime() - date.getTime();
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 1) return "Just now";
        if (diffMins < 60) return `${diffMins}m ago`;
        if (diffHours < 24) return `${diffHours}h ago`;
        if (diffDays < 7) return `${diffDays}d ago`;

        return new Intl.DateTimeFormat("en-US", {
            month: "short",
            day: "numeric",
        }).format(date);
    };

    return (
        <div
            className={cn(
                "group relative rounded-lg p-3 transition-colors cursor-pointer",
                "hover:bg-accent",
                isActive && "bg-accent",
            )}
            onClick={onClick}
        >
            {/* Conversation Title */}
            <div className="mb-1 flex items-start justify-between gap-2">
                <h4 className="flex-1 truncate text-sm font-medium">{conversation.title}</h4>

                {/* Menu Button */}
                <DropdownMenu open={isMenuOpen} onOpenChange={setIsMenuOpen}>
                    <DropdownMenuTrigger asChild>
                        <Button
                            size="icon"
                            variant="ghost"
                            className="h-6 w-6 flex-shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                            onClick={(e) => {
                                e.stopPropagation();
                            }}
                        >
                            <MoreVerticalIcon className="h-3.5 w-3.5" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        {onEdit && (
                            <DropdownMenuItem
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onEdit();
                                }}
                            >
                                <Edit2Icon className="mr-2 h-4 w-4" />
                                Rename
                            </DropdownMenuItem>
                        )}
                        {onArchive && (
                            <DropdownMenuItem
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onArchive();
                                }}
                            >
                                <ArchiveIcon className="mr-2 h-4 w-4" />
                                Archive
                            </DropdownMenuItem>
                        )}
                        {onDelete && (
                            <>
                                <DropdownMenuSeparator />
                                <DropdownMenuItem
                                    className="text-destructive focus:text-destructive"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        onDelete();
                                    }}
                                >
                                    <Trash2Icon className="mr-2 h-4 w-4" />
                                    Delete
                                </DropdownMenuItem>
                            </>
                        )}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>

            {/* Metadata */}
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span>{endpointConfig?.icon}</span>
                <span className="truncate">
                    {conversation.modelLabel || conversation.model}
                </span>
                <span>•</span>
                <span>{formatDate(conversation.updatedAt)}</span>
            </div>

            {/* Tags */}
            {conversation.tags && conversation.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                    {conversation.tags.slice(0, 3).map((tag) => (
                        <Badge
                            key={tag.id}
                            variant="outline"
                            className="text-xs"
                            style={{
                                borderColor: tag.color || undefined,
                                color: tag.color || undefined,
                            }}
                        >
                            {tag.name}
                        </Badge>
                    ))}
                    {conversation.tags.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                            +{conversation.tags.length - 3}
                        </Badge>
                    )}
                </div>
            )}
        </div>
    );
}

