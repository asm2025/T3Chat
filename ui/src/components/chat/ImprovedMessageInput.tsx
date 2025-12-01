import React, { useEffect, useState, useImperativeHandle, forwardRef } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ArrowUp, Paperclip, Settings, ChevronDown, Image, Link, Upload } from "lucide-react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger, DropdownMenuSub, DropdownMenuSubTrigger, DropdownMenuSubContent, DropdownMenuSeparator } from "@/components/ui/dropdown-menu";
import { cn, getErrorMessage } from "@/lib/utils";
import { toast } from "@/lib/toast";
import type { AIModel } from "@/types/model";
import type { AiProvider } from "@/types/chat";
import { t3ChatClient } from "@/lib/t3-chat-client";

interface MessageInputProps {
    onSend: (content: string) => void;
    disabled?: boolean;
    models?: AIModel[];
    selectedModel?: AIModel | null;
    onModelChange?: (model: AIModel) => void;
    onSettingsClick?: () => void;
    onFileAttach?: (source: 'device' | 'url' | 'cloud') => void;
}

export interface MessageInputRef {
    setContent: (content: string) => void;
    clearContent: () => void;
}

// Group models by provider
const groupModelsByProvider = (models: AIModel[]): Record<string, AIModel[]> => {
    return models.reduce((acc, model) => {
        const provider = model.provider;
        if (!acc[provider]) {
            acc[provider] = [];
        }
        acc[provider].push(model);
        return acc;
    }, {} as Record<string, AIModel[]>);
};

// Provider display names
const providerNames: Record<string, string> = {
    openai: 'OpenAI',
    anthropic: 'Anthropic',
    google: 'Google',
    meta: 'Meta',
    deepseek: 'DeepSeek',
    mistral: 'Mistral',
    xai: 'xAI',
    cohere: 'Cohere',
    alibaba: 'Alibaba',
    ollama: 'Ollama'
};

export const ImprovedMessageInput = forwardRef<MessageInputRef, MessageInputProps>(
    ({ onSend, disabled, models = [], selectedModel, onModelChange, onSettingsClick, onFileAttach }, ref) => {
        const [content, setContent] = useState("");
        const [fetchedModels, setFetchedModels] = useState<AIModel[] | null>(null);
        const [loadingModels, setLoadingModels] = useState(false);

        // Expose methods via ref
        useImperativeHandle(ref, () => ({
            setContent: (newContent: string) => {
                setContent(newContent);
            },
            clearContent: () => {
                setContent("");
            },
        }));

        // Fetch models from server if no models provided via props
        useEffect(() => {
            if (models && models.length > 0) {
                setFetchedModels(models);
                return;
            }
            let isMounted = true;
            setLoadingModels(true);
            t3ChatClient
                .listModels()
                .then((data) => {
                    if (isMounted) setFetchedModels(data);
                })
                .catch(() => {
                    if (isMounted) setFetchedModels([]);
                })
                .finally(() => {
                    if (isMounted) setLoadingModels(false);
                });
            return () => {
                isMounted = false;
            };
        }, [models]);

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

        // Source models: prefer props, else fetched
        const sourceModels = (models && models.length > 0 ? models : fetchedModels) ?? [];
        // Filter enabled models (active only)
        const enabledModels = sourceModels.filter((model) => model.is_active);
        // Group by provider
        const groupedModels = groupModelsByProvider(enabledModels);
        const providers = Object.keys(groupedModels).sort();

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
                    {/* Text Area */}
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
                            className="text-foreground placeholder:text-secondary-foreground/60 w-full min-w-0 resize-none bg-transparent text-base leading-6 outline-none disabled:opacity-0 border-0 p-0 focus-visible:ring-0 shadow-none min-h-[60px] max-h-[120px]"
                            rows={2}
                        />
                        <div id="chat-input-description" className="sr-only">
                            Press Enter to send, Shift + Enter for new line
                        </div>
                    </div>

                    {/* Bottom Controls Row */}
                    <div className="mt-1 -mb-px flex w-full min-w-0 flex-row items-center justify-between gap-2">
                        {/* Left side - Model selector and attachment button */}
                        <div className="flex min-w-0 flex-1 items-center gap-2">
                            {/* Model Selector */}
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        className={cn(
                                            "h-7 text-xs gap-1 px-2 rounded-md",
                                            "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
                                            "focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                                        )}
                                        aria-label={selectedModel ? `Select model. Current: ${selectedModel.display_name}` : "Select model"}>
                                        <span className="truncate max-w-[150px]">
                                            {selectedModel?.display_name || (enabledModels.length === 0 ? "No models" : "Select model")}
                                        </span>
                                        <ChevronDown className="h-3 w-3 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="start" className="w-56">
                                    {loadingModels ? (
                                        <div className="p-2 text-sm text-muted-foreground">Loading models...</div>
                                    ) : providers.length === 0 ? (
                                        <div className="p-2 text-sm text-muted-foreground">No models available</div>
                                    ) : (
                                        providers.map((provider) => (
                                            <DropdownMenuSub key={provider}>
                                                <DropdownMenuSubTrigger>
                                                    <span>{providerNames[provider] || provider}</span>
                                                </DropdownMenuSubTrigger>
                                                <DropdownMenuSubContent>
                                                    {groupedModels[provider]?.map((model) => (
                                                        <DropdownMenuItem
                                                            key={model.id}
                                                            onClick={() => onModelChange?.(model)}
                                                            className={cn(
                                                                selectedModel?.id === model.id && "bg-accent"
                                                            )}>
                                                            <span className="truncate">{model.display_name}</span>
                                                        </DropdownMenuItem>
                                                    ))}
                                                </DropdownMenuSubContent>
                                            </DropdownMenuSub>
                                        ))
                                    )}
                                </DropdownMenuContent>
                            </DropdownMenu>

                            {/* Attachment Button with menu */}
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        disabled={disabled}
                                        className={cn(
                                            "h-7 w-7 p-0 rounded-md",
                                            "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
                                            "focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                                        )}
                                        aria-label="Attach file">
                                        <Paperclip className="h-4 w-4" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="start">
                                    <DropdownMenuItem onClick={() => onFileAttach?.('device')}>
                                        <Upload className="mr-2 h-4 w-4" />
                                        <span>Upload from device</span>
                                    </DropdownMenuItem>
                                    <DropdownMenuItem onClick={() => onFileAttach?.('url')}>
                                        <Link className="mr-2 h-4 w-4" />
                                        <span>From URL</span>
                                    </DropdownMenuItem>
                                    <DropdownMenuItem onClick={() => onFileAttach?.('cloud')}>
                                        <Image className="mr-2 h-4 w-4" />
                                        <span>From cloud storage</span>
                                    </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                        </div>

                        {/* Right side - Settings and Send button */}
                        <div className="flex items-center gap-2">
                            {/* Settings Button (Cog Icon) */}
                            {onSettingsClick && (
                                <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    onClick={onSettingsClick}
                                    className={cn(
                                        "h-7 w-7 p-0 rounded-md",
                                        "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
                                        "focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                                    )}
                                    aria-label="Settings">
                                    <Settings className="h-4 w-4" />
                                </Button>
                            )}

                            {/* Send Button */}
                            <Button
                                type="submit"
                                disabled={disabled || !content.trim()}
                                size="sm"
                                className={cn(
                                    "h-8 w-8 p-0 rounded-lg",
                                    "bg-foreground hover:bg-foreground/90 active:bg-foreground",
                                    "disabled:hover:bg-foreground/50 disabled:active:bg-foreground/50",
                                    "text-background border-reflect button-reflect",
                                )}
                                aria-label={content.trim() ? "Send message" : "Message requires text"}>
                                <ArrowUp className="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                </form>
            </div>
        );
    }
);

ImprovedMessageInput.displayName = "ImprovedMessageInput";

