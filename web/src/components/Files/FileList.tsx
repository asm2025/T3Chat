// ============================================================================
// FileList Component - Phase 1B
// ============================================================================
// List of uploaded files with preview and management

import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { DownloadIcon, Trash2Icon, FileIcon, ImageIcon, FileTextIcon, FileAudioIcon, FileVideoIcon } from "lucide-react";
import type { File as LibreChatFile } from "@/types/librechat";
import { cn } from "@/lib/utils";

// ============================================================================
// Props
// ============================================================================

export interface FileListProps {
    files: LibreChatFile[];
    onDownload?: (fileId: string) => void;
    onDelete?: (fileId: string) => void;
    className?: string;
}

// ============================================================================
// Helper Functions
// ============================================================================

const getFileIcon = (fileType: LibreChatFile["fileType"]) => {
    switch (fileType) {
        case "image":
            return <ImageIcon className="h-8 w-8 text-blue-500" />;
        case "document":
            return <FileTextIcon className="h-8 w-8 text-orange-500" />;
        case "audio":
            return <FileAudioIcon className="h-8 w-8 text-purple-500" />;
        case "video":
            return <FileVideoIcon className="h-8 w-8 text-pink-500" />;
        default:
            return <FileIcon className="h-8 w-8 text-muted-foreground" />;
    }
};

const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
};

const formatDate = (dateString: string): string => {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "numeric",
        minute: "2-digit",
    }).format(date);
};

// ============================================================================
// Component
// ============================================================================

export function FileList({ files, onDownload, onDelete, className }: FileListProps) {
    if (files.length === 0) {
        return (
            <div className="p-8 text-center text-sm text-muted-foreground">
                <FileIcon className="mx-auto mb-2 h-12 w-12 opacity-50" />
                <p>No files yet</p>
            </div>
        );
    }

    return (
        <div className={cn("space-y-2", className)}>
            {files.map((file) => (
                <Card key={file.id} className="group flex items-start gap-4 p-4">
                    {/* File Icon or Image Preview */}
                    <div className="flex-shrink-0">
                        {file.fileType === "image" && file.filepath ? (
                            <div className="h-16 w-16 overflow-hidden rounded border">
                                <img
                                    src={file.filepath}
                                    alt={file.filename}
                                    className="h-full w-full object-cover"
                                    onError={(e) => {
                                        // Fallback to icon if image fails to load
                                        (e.target as HTMLImageElement).style.display = "none";
                                    }}
                                />
                            </div>
                        ) : (
                            getFileIcon(file.fileType)
                        )}
                    </div>

                    {/* File Info */}
                    <div className="flex-1 min-w-0">
                        <div className="mb-1 flex items-start justify-between gap-2">
                            <h4 className="truncate font-medium">{file.filename}</h4>
                            <div className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                                {onDownload && (
                                    <Button
                                        size="icon"
                                        variant="ghost"
                                        onClick={() => onDownload(file.id)}
                                        title="Download"
                                    >
                                        <DownloadIcon className="h-4 w-4" />
                                    </Button>
                                )}
                                {onDelete && (
                                    <Button
                                        size="icon"
                                        variant="ghost"
                                        onClick={() => onDelete(file.id)}
                                        className="text-destructive hover:text-destructive"
                                        title="Delete"
                                    >
                                        <Trash2Icon className="h-4 w-4" />
                                    </Button>
                                )}
                            </div>
                        </div>

                        <div className="mb-2 flex flex-wrap gap-2 text-xs text-muted-foreground">
                            <span>{formatFileSize(file.sizeBytes)}</span>
                            <span>•</span>
                            <span>{file.mimeType}</span>
                            <span>•</span>
                            <span>{formatDate(file.createdAt)}</span>
                        </div>

                        {/* Badges */}
                        <div className="flex flex-wrap gap-1.5">
                            {file.isEmbedded && <Badge variant="secondary" className="text-xs">Embedded</Badge>}
                            {file.isTemporary && <Badge variant="outline" className="text-xs">Temporary</Badge>}
                            {file.source === "generated" && <Badge variant="secondary" className="text-xs">Generated</Badge>}
                            {file.usageCount > 0 && (
                                <Badge variant="outline" className="text-xs">
                                    Used {file.usageCount}x
                                </Badge>
                            )}
                        </div>

                        {/* Image dimensions */}
                        {file.fileType === "image" && file.width && file.height && (
                            <p className="mt-1 text-xs text-muted-foreground">
                                {file.width} × {file.height}
                            </p>
                        )}
                    </div>
                </Card>
            ))}
        </div>
    );
}

