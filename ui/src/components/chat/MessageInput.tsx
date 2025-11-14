import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ArrowUp, Paperclip, Search, ChevronDown } from "lucide-react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
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
        <div className="border-reflect pointer-events-none min-w-0 overflow-hidden rounded-t-[20px] bg-chat-input-background p-2 pb-0 backdrop-blur-lg">
            <form
                onSubmit={handleSubmit}
                className={cn(
                    "text-secondary-foreground pb-safe-offset-3 dark:bg-secondary/4.5 dark:outline-chat-background/40",
                    "pointer-events-auto relative flex w-full min-w-0 flex-col items-stretch gap-2",
                    "rounded-t-xl border border-b-0 border-white/70 bg-chat-input-background px-3 pt-3",
                    "outline-8 outline-solid max-sm:pb-6 sm:max-w-3xl dark:border-[hsl(0,100%,83%)]/4",
                    "shadow-[rgba(0,0,0,0.1)_0px_80px_50px_0px,rgba(0,0,0,0.07)_0px_50px_30px_0px,rgba(0,0,0,0.06)_0px_30px_15px_0px,rgba(0,0,0,0.04)_0px_15px_8px,rgba(0,0,0,0.04)_0px_6px_4px,rgba(0,0,0,0.02)_0px_2px_2px]",
                )}
                id="chat-input-form">
                <div className="flex min-w-0 grow flex-row items-start">
                    <Textarea
                        value={content}
                        onChange={(e) => setContent(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Type your message here..."
                        disabled={disabled}
                        id="chat-input"
                        aria-label="Message input"
                        aria-describedby="chat-input-description"
                        autoComplete="off"
                        className="text-foreground placeholder:text-secondary-foreground/60 w-full min-w-0 resize-none bg-transparent text-base leading-6 outline-none disabled:opacity-0 border-0 p-0 focus-visible:ring-0 shadow-none"
                        style={{ height: "48px" }}
                    />
                    <div id="chat-input-description" className="sr-only">
                        Press Enter to send, Shift + Enter for new line
                    </div>
                </div>
                <div className="mt-2 -mb-px flex w-full min-w-0 flex-row-reverse justify-between">
                    {/* Send Button */}
                    <div className="-mt-0.5 -mr-0.5 flex shrink-0 items-center justify-center gap-2" aria-label="Message actions">
                        <Button
                            type="submit"
                            disabled={disabled || !content.trim()}
                            className={cn(
                                "size-9 relative rounded-lg p-2 font-semibold shadow-sm",
                                "bg-foreground hover:bg-foreground/90 active:bg-foreground",
                                "disabled:hover:bg-foreground/50 disabled:active:bg-foreground/50",
                                "text-background border-reflect button-reflect",
                            )}
                            aria-label={content.trim() ? "Send message" : "Message requires text"}>
                            <ArrowUp className="size-5" />
                        </Button>
                    </div>

                    {/* Left side buttons */}
                    <div className="flex min-w-0 flex-1 items-center gap-2 pr-2">
                        <div className="flex min-w-0 items-center gap-2">
                            {/* Model Selector */}
                            {enabledModels.length > 0 && (
                                <div className="min-w-0 flex-1">
                                    <DropdownMenu>
                                        <DropdownMenuTrigger asChild>
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                className={cn(
                                                    "h-8 rounded-md text-xs relative flex max-w-[128px] min-w-0 items-center gap-1 px-1 py-1.5",
                                                    "sm:gap-2 sm:px-2 md:max-w-none",
                                                    "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
                                                    "disabled:hover:text-foreground/50 disabled:hover:bg-transparent",
                                                    "focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                                                )}
                                                aria-label={selectedModel ? `Select model. Current model: ${selectedModel.display_name}` : "Select model"}
                                                aria-haspopup="listbox">
                                                <div className="min-w-0 flex-1 text-left text-sm font-medium">
                                                    <div className="truncate">{selectedModel?.display_name || "Select model"}</div>
                                                </div>
                                                <ChevronDown className="right-0 size-4" />
                                            </Button>
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent align="start" className="w-[200px]">
                                            {enabledModels.map((model) => (
                                                <DropdownMenuItem
                                                    key={model.id}
                                                    onClick={() => {
                                                        if (onModelChange) {
                                                            onModelChange(model);
                                                        }
                                                    }}
                                                    className={cn(selectedModel?.id === model.id && "bg-accent")}>
                                                    {model.display_name}
                                                </DropdownMenuItem>
                                            ))}
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>
                            )}

                            {/* Web Search Toggle */}
                            {onWebSearchToggle && (
                                <div className="shrink-0">
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        onClick={() => onWebSearchToggle(!webSearchEnabled)}
                                        className={cn(
                                            "text-xs h-8 gap-2 rounded-full border border-solid px-2 sm:px-2.5",
                                            "border-secondary-foreground/10 text-muted-foreground py-1.5",
                                            "hover:bg-muted/40 hover:text-foreground",
                                            "disabled:hover:text-foreground/50 disabled:hover:bg-transparent",
                                            "focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                                        )}
                                        aria-label={webSearchEnabled ? "Web search enabled" : "Web search disabled"}>
                                        <Search className="h-4 w-4" />
                                        <span className="hidden md:block">Search</span>
                                    </Button>
                                </div>
                            )}

                            {/* File Attachment Button */}
                            <div className="shrink-0">
                                <Button
                                    type="button"
                                    variant="ghost"
                                    onClick={handleFileAttach}
                                    disabled={disabled}
                                    className={cn(
                                        "text-xs h-8 gap-2 rounded-full border border-solid px-2 sm:px-2.5",
                                        "border-secondary-foreground/10 text-muted-foreground py-1.5",
                                        "hover:bg-muted/40 hover:text-foreground",
                                        "disabled:hover:text-foreground/50 disabled:hover:bg-transparent",
                                        "focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                                    )}
                                    aria-label="Attach file">
                                    <Paperclip className="size-4" />
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </form>
        </div>
    );
}
