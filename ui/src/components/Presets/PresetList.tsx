// ============================================================================
// PresetList Component - Phase 1B
// ============================================================================
// List of saved presets with quick load functionality

import { useEffect } from "react";
import { usePresets } from "@/stores/appStore";
import { PresetItem } from "./PresetItem";
import { Button } from "@/components/ui/button";
import { PlusIcon } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";

// ============================================================================
// Props
// ============================================================================

export interface PresetListProps {
    onSelect?: (presetId: string) => void;
    onEdit?: (presetId: string) => void;
    onDelete?: (presetId: string) => void;
    onCreate?: () => void;
    selectedPresetId?: string;
}

// ============================================================================
// Component
// ============================================================================

export function PresetList({ onSelect, onEdit, onDelete, onCreate, selectedPresetId }: PresetListProps) {
    const { presets, loading, error, fetchPresets } = usePresets();

    useEffect(() => {
        fetchPresets();
    }, [fetchPresets]);

    if (loading) {
        return (
            <div className="space-y-2 p-4">
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
                <Skeleton className="h-16 w-full" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="p-4 text-center text-sm text-destructive">
                <p>Failed to load presets</p>
                <p className="text-xs text-muted-foreground">{error.message}</p>
            </div>
        );
    }

    // Sort presets: default first, then by order index
    const sortedPresets = [...presets].sort((a, b) => {
        if (a.isDefault && !b.isDefault) return -1;
        if (!a.isDefault && b.isDefault) return 1;
        return a.orderIndex - b.orderIndex;
    });

    return (
        <div className="flex h-full flex-col">
            {/* Header with Create button */}
            <div className="border-b p-4">
                <Button onClick={onCreate} className="w-full" variant="outline">
                    <PlusIcon className="mr-2 h-4 w-4" />
                    Create Preset
                </Button>
            </div>

            {/* Presets list */}
            <div className="flex-1 overflow-y-auto">
                {sortedPresets.length === 0 ? (
                    <div className="p-8 text-center text-sm text-muted-foreground">
                        <p>No presets yet</p>
                        <p className="mt-2 text-xs">Create a preset to save your favorite settings</p>
                    </div>
                ) : (
                    <div className="space-y-2 p-2">
                        {sortedPresets.map((preset) => (
                            <PresetItem
                                key={preset.id}
                                preset={preset}
                                isSelected={selectedPresetId === preset.id}
                                onSelect={() => onSelect?.(preset.id)}
                                onEdit={() => onEdit?.(preset.id)}
                                onDelete={() => onDelete?.(preset.id)}
                            />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

