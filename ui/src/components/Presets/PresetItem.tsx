// ============================================================================
// PresetItem Component - Phase 1B
// ============================================================================
// Individual preset card in the list

import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Edit2Icon, Trash2Icon, StarIcon } from "lucide-react";
import type { Preset } from "@/types/librechat";
import { cn } from "@/lib/utils";
import { ENDPOINTS } from "@/components/Endpoints";

// ============================================================================
// Props
// ============================================================================

export interface PresetItemProps {
    preset: Preset;
    isSelected?: boolean;
    onSelect?: () => void;
    onEdit?: () => void;
    onDelete?: () => void;
}

// ============================================================================
// Component
// ============================================================================

export function PresetItem({ preset, isSelected = false, onSelect, onEdit, onDelete }: PresetItemProps) {
    const endpointConfig = ENDPOINTS.find((e) => e.value === preset.endpoint);

    return (
        <Card
            className={cn(
                "group relative cursor-pointer p-4 transition-all hover:border-primary",
                isSelected && "border-primary bg-primary/5",
            )}
            onClick={onSelect}
        >
            {/* Default badge */}
            {preset.isDefault && (
                <div className="absolute right-2 top-2">
                    <Badge variant="secondary" className="gap-1">
                        <StarIcon className="h-3 w-3 fill-current" />
                        Default
                    </Badge>
                </div>
            )}

            {/* Title */}
            <h3 className="mb-2 font-semibold">{preset.title}</h3>

            {/* Endpoint & Model */}
            <div className="mb-3 flex items-center gap-2 text-sm">
                <span>{endpointConfig?.icon}</span>
                <span className="text-muted-foreground">
                    {endpointConfig?.label} - {preset.modelLabel || preset.model}
                </span>
            </div>

            {/* Parameters summary */}
            <div className="mb-3 flex flex-wrap gap-2 text-xs">
                {preset.modelParameters?.temperature !== undefined && (
                    <Badge variant="outline">Temp: {preset.modelParameters.temperature.toFixed(2)}</Badge>
                )}
                {preset.modelParameters?.max_tokens !== undefined && (
                    <Badge variant="outline">Max: {preset.modelParameters.max_tokens}</Badge>
                )}
                {preset.systemMessage && <Badge variant="outline">System msg</Badge>}
            </div>

            {/* Actions */}
            <div className="flex gap-2 opacity-0 transition-opacity group-hover:opacity-100">
                <Button
                    size="sm"
                    variant="ghost"
                    onClick={(e) => {
                        e.stopPropagation();
                        onEdit?.();
                    }}
                >
                    <Edit2Icon className="h-3 w-3" />
                </Button>
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
            </div>
        </Card>
    );
}

