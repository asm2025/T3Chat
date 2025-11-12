import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ArrowUp, Paperclip, Search } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { AIModel } from "@/types/model";

interface MessageInputProps {
    onSend: (content: string) => void;
    disabled?: boolean;
    models?: AIModel[];
    selectedModel?: AIModel | null;
    onModelChange?: (model: AIModel) => void;
    webSearchEnabled?: boolean;
    onWebSearchToggle?: (enabled: boolean) => void;
}

export function MessageInput({ onSend, disabled, models = [], selectedModel, onModelChange, webSearchEnabled = false, onWebSearchToggle }: MessageInputProps) {
    const [content, setContent] = useState("");

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (content.trim() && !disabled) {
            onSend(content);
            setContent("");
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            if (content.trim() && !disabled) {
                onSend(content);
                setContent("");
            }
        }
    };

    const handleFileAttach = () => {
        // TODO: Implement file attachment functionality
        console.log("File attachment clicked");
    };

    // Filter enabled models
    const enabledModels = models.filter((model) => model.is_active);

    return (
        <form onSubmit={handleSubmit} className="w-full px-6">
            <div className="mx-auto w-full max-w-4xl">
                <div className="border border-border rounded-lg bg-background p-4 shadow-sm">
                    <Textarea
                        value={content}
                        onChange={(e) => setContent(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Type your message here..."
                        disabled={disabled}
                        rows={4}
                        className="min-h-[100px] resize-none border-none bg-transparent text-sm leading-relaxed outline-none focus-visible:ring-0 p-0"
                    />
                    <div className="mt-3 flex items-center justify-between gap-2">
                        <div className="flex items-center gap-2">
                            {/* Model Selector */}
                            {enabledModels.length > 0 && (
                                <Select
                                    value={selectedModel?.id || ""}
                                    onValueChange={(value) => {
                                        const model = enabledModels.find((m) => m.id === value);
                                        if (model && onModelChange) {
                                            onModelChange(model);
                                        }
                                    }}>
                                    <SelectTrigger className="h-8 w-[140px] text-xs">
                                        <SelectValue placeholder="Select model" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {enabledModels.map((model) => (
                                            <SelectItem key={model.id} value={model.id}>
                                                {model.display_name}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            )}

                            {/* Web Search Toggle */}
                            {onWebSearchToggle && (
                                <div className="flex items-center gap-2 px-2">
                                    <Search className="h-4 w-4 text-muted-foreground" />
                                    <Switch checked={webSearchEnabled} onCheckedChange={onWebSearchToggle} className="h-5 w-9" />
                                </div>
                            )}

                            {/* File Attachment Button */}
                            <Button type="button" variant="ghost" size="icon" onClick={handleFileAttach} className="h-8 w-8" disabled={disabled}>
                                <Paperclip className="h-4 w-4" />
                                <span className="sr-only">Attach file</span>
                            </Button>
                        </div>

                        {/* Send Button */}
                        <Button type="submit" disabled={disabled || !content.trim()} className="h-8 w-8 rounded-md bg-foreground text-background shadow-sm hover:bg-foreground/90 p-0">
                            <ArrowUp className="h-4 w-4" />
                            <span className="sr-only">Send</span>
                        </Button>
                    </div>
                </div>
            </div>
        </form>
    );
}
