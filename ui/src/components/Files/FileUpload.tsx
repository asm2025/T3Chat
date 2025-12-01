// ============================================================================
// FileUpload Component - Phase 1B
// ============================================================================
// Drag-and-drop file upload component for multimodal conversations

import { useCallback, useState } from "react";
import { useDropzone } from "react-dropzone";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { UploadIcon, XIcon, FileIcon, ImageIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { toast } from "sonner";

// ============================================================================
// Props
// ============================================================================

export interface FileUploadProps {
    onFilesSelected: (files: File[]) => void;
    maxFiles?: number;
    maxSize?: number; // in bytes
    accept?: Record<string, string[]>;
    className?: string;
}

// ============================================================================
// Component
// ============================================================================

export function FileUpload({ onFilesSelected, maxFiles = 10, maxSize = 10 * 1024 * 1024, accept, className }: FileUploadProps) {
    const [selectedFiles, setSelectedFiles] = useState<File[]>([]);

    const onDrop = useCallback(
        (acceptedFiles: File[]) => {
            // Validate file size
            const oversizedFiles = acceptedFiles.filter((file) => file.size > maxSize);
            if (oversizedFiles.length > 0) {
                toast.error(`Some files exceed the maximum size of ${(maxSize / 1024 / 1024).toFixed(0)}MB`);
                return;
            }

            // Check max files
            const totalFiles = selectedFiles.length + acceptedFiles.length;
            if (totalFiles > maxFiles) {
                toast.error(`Maximum ${maxFiles} files allowed`);
                return;
            }

            const newFiles = [...selectedFiles, ...acceptedFiles];
            setSelectedFiles(newFiles);
            onFilesSelected(newFiles);
        },
        [selectedFiles, maxFiles, maxSize, onFilesSelected],
    );

    const { getRootProps, getInputProps, isDragActive } = useDropzone({
        onDrop,
        accept,
        maxFiles,
        maxSize,
    });

    const removeFile = (index: number) => {
        const newFiles = selectedFiles.filter((_, i) => i !== index);
        setSelectedFiles(newFiles);
        onFilesSelected(newFiles);
    };

    const isImage = (file: File) => file.type.startsWith("image/");

    return (
        <div className={cn("space-y-4", className)}>
            {/* Dropzone */}
            <Card
                {...getRootProps()}
                className={cn(
                    "cursor-pointer border-2 border-dashed p-8 text-center transition-colors",
                    isDragActive && "border-primary bg-primary/5",
                )}
            >
                <input {...getInputProps()} />
                <div className="flex flex-col items-center gap-2">
                    <UploadIcon className={cn("h-10 w-10 text-muted-foreground", isDragActive && "text-primary")} />
                    <div>
                        <p className="font-medium">
                            {isDragActive ? "Drop files here" : "Drag & drop files here"}
                        </p>
                        <p className="text-sm text-muted-foreground">or click to browse</p>
                    </div>
                    <p className="text-xs text-muted-foreground">
                        Max {maxFiles} files • Max {(maxSize / 1024 / 1024).toFixed(0)}MB per file
                    </p>
                </div>
            </Card>

            {/* Selected Files */}
            {selectedFiles.length > 0 && (
                <div className="space-y-2">
                    <p className="text-sm font-medium">Selected Files ({selectedFiles.length})</p>
                    <div className="space-y-2">
                        {selectedFiles.map((file, index) => (
                            <Card key={`${file.name}-${index}`} className="flex items-center gap-3 p-3">
                                {/* Icon */}
                                <div className="flex-shrink-0">
                                    {isImage(file) ? (
                                        <ImageIcon className="h-8 w-8 text-blue-500" />
                                    ) : (
                                        <FileIcon className="h-8 w-8 text-muted-foreground" />
                                    )}
                                </div>

                                {/* File Info */}
                                <div className="flex-1 min-w-0">
                                    <p className="truncate text-sm font-medium">{file.name}</p>
                                    <p className="text-xs text-muted-foreground">
                                        {(file.size / 1024).toFixed(1)} KB • {file.type || "Unknown type"}
                                    </p>
                                </div>

                                {/* Remove Button */}
                                <Button
                                    size="icon"
                                    variant="ghost"
                                    className="flex-shrink-0"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        removeFile(index);
                                    }}
                                >
                                    <XIcon className="h-4 w-4" />
                                </Button>
                            </Card>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

