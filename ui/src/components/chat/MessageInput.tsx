import React, { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ArrowUp, Paperclip, Search, ChevronDown, ChevronLeft, ChevronUp } from "lucide-react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import type { AIModel } from "@/types/model";
import { fetchJson } from "@/lib/fetch-interceptor";

const API_BASE_URL = (import.meta as any).env?.VITE_API_URL || "";

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
    const [fetchedModels, setFetchedModels] = useState<AIModel[] | null>(null); // active
    const [fetchedAllModels, setFetchedAllModels] = useState<AIModel[] | null>(null); // all
    const [showAllModels, setShowAllModels] = useState(false);
    const [searchQuery, setSearchQuery] = useState("");
    const [loadingModels, setLoadingModels] = useState(false);
    const [allModelsError, setAllModelsError] = useState<string | null>(null);

    // Fetch active models from server if no models provided via props
    useEffect(() => {
        if (models && models.length > 0) {
            setFetchedModels(models);
            return;
        }
        let isMounted = true;
        setLoadingModels(true);
        fetchJson<AIModel[]>(`${API_BASE_URL}/api/v1/models`)
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

    // Fetch all models lazily when toggled to "Show all"
    useEffect(() => {
        if (!showAllModels) return;
        // If we've already attempted to load (success or failure), don't refetch repeatedly
        if (fetchedAllModels !== null) return;
        let isMounted = true;
        setLoadingModels(true);
        setAllModelsError(null);
        fetchJson<AIModel[]>(`${API_BASE_URL}/api/v1/models/all`)
            .then((data) => {
                if (isMounted) setFetchedAllModels(data);
            })
            .catch((err) => {
                if (!isMounted) return;
                console.error("Failed to load all models", err);
                setFetchedAllModels([]);
                setAllModelsError(err instanceof Error ? err.message : "Failed to load all models");
            })
            .finally(() => {
                if (isMounted) setLoadingModels(false);
            });
        return () => {
            isMounted = false;
        };
    }, [showAllModels, fetchedAllModels]);

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

    // Source models: prefer props, else fetched
    const sourceModels = (models && models.length > 0 ? models : fetchedModels) ?? [];
    // Filter enabled models (active only) for dropdown
    const enabledModels = sourceModels.filter((model) => model.is_active);
    // Determine list to show based on toggle
    const allModels = (fetchedAllModels ?? enabledModels) as AIModel[];

    const listToRender = (showAllModels ? allModels : enabledModels).filter((m: AIModel) => m.display_name.toLowerCase().includes(searchQuery.toLowerCase()));

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
                                            aria-label={selectedModel ? `Select model. Current model: ${selectedModel.display_name}` : enabledModels.length === 0 ? "No models activated" : "Select model"}
                                            aria-haspopup="listbox">
                                            <div className="min-w-0 flex-1 text-left text-sm font-medium">
                                                <div className="truncate">{selectedModel?.display_name || (enabledModels.length === 0 ? "No models activated" : "Select model")}</div>
                                            </div>
                                            <ChevronDown className="right-0 size-4" />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent
                                        align="start"
                                        className={cn("bg-popover text-popover-foreground z-50 shadow-md", "relative overflow-hidden rounded-lg border-none p-0", "max-w-[calc(100vw-2rem)] sm:w-[420px]")}
                                        id="model-selector-listbox"
                                        aria-label="Model selector"
                                        tabIndex={-1}
                                        data-orientation="vertical">
                                        {/* Upgrade/promo section */}
                                        <div className="text-card-foreground rounded-xl bg-popover border-0 px-3 py-2.5 shadow-none">
                                            <div className="flex flex-col border-reflect bg-popover space-y-3 rounded-md p-5">
                                                <h3 className="text-xl font-semibold">Unlock all models + higher limits</h3>
                                                <div className="flex items-end justify-between">
                                                    <div className="flex items-baseline gap-1">
                                                        <span className="text-3xl font-bold text-pink-500">$8</span>
                                                        <span className="text-primary dark:text-primary-foreground">/month</span>
                                                    </div>
                                                    <Button className="focus-visible:ring-ring inline-flex items-center justify-center gap-2 text-sm whitespace-nowrap transition-colors focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 border-reflect button-reflect text-background rounded-lg bg-foreground p-2 font-semibold shadow-sm hover:bg-foreground/90 active:bg-foreground disabled:hover:bg-foreground/50 disabled:active:bg-foreground/50 h-9 px-4 py-2">
                                                        Upgrade now
                                                    </Button>
                                                </div>
                                            </div>
                                        </div>

                                        {/* Search input */}
                                        <div className="bg-popover sticky top-0 rounded-t-lg px-3.5 pt-0.5 sm:inset-x-0">
                                            <div className="flex items-center">
                                                <Search className="text-muted-foreground/75 mr-3 ml-px h-4 w-4" />
                                                <input
                                                    role="searchbox"
                                                    aria-label="Search models"
                                                    autoComplete="off"
                                                    placeholder="Search models..."
                                                    className="text-foreground placeholder-muted-foreground/50 w-full bg-transparent py-2 text-sm placeholder:select-none focus:outline-hidden"
                                                    value={searchQuery}
                                                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearchQuery(e.target.value)}
                                                />
                                            </div>
                                            <div className="border-chat-border border-b px-3"></div>
                                        </div>

                                        {/* Model list(s) */}
                                        <div className="max-h-[360px] overflow-y-auto px-1.5 scroll-shadow" data-shadow="true">
                                            {loadingModels && <div className="p-3 text-sm text-muted-foreground">Loading models…</div>}
                                            {!loadingModels && allModelsError && showAllModels && <div className="p-3 text-sm text-red-500">Failed to load all models: {allModelsError}</div>}
                                            {!loadingModels && listToRender.length === 0 && !allModelsError && (
                                                <div className="p-3 text-sm text-muted-foreground">{!showAllModels && enabledModels.length === 0 && searchQuery.trim().length === 0 ? "No models activated." : "No models found."}</div>
                                            )}

                                            {/* Active (favorites) list - simple list */}
                                            {!showAllModels &&
                                                listToRender.map((model: AIModel) => (
                                                    <DropdownMenuItem
                                                        key={model.id}
                                                        onClick={() => onModelChange?.(model)}
                                                        className={cn(
                                                            "focus:bg-accent/30 hover:bg-accent/30 focus:text-accent-foreground hover:text-accent-foreground relative cursor-default rounded-sm text-sm outline-hidden transition-colors select-none group flex flex-col items-start gap-1 p-0",
                                                            selectedModel?.id === model.id && "bg-accent",
                                                        )}
                                                        id={`model-option-${model.id}`}
                                                        aria-selected={selectedModel?.id === model.id}
                                                        tabIndex={-1}
                                                        aria-label={model.display_name}
                                                        data-orientation="vertical">
                                                        <div className="flex w-full items-center justify-between p-3">
                                                            <div className="text-muted-foreground flex items-center gap-2 pr-2 font-medium transition-colors">
                                                                <span className="w-fit">{model.display_name}</span>
                                                            </div>
                                                            <div className="flex items-center gap-2" />
                                                        </div>
                                                    </DropdownMenuItem>
                                                ))}

                                            {/* Show all list - denser layout */}
                                            {showAllModels && (
                                                <div className="flex flex-col gap-1 p-1">
                                                    {listToRender.map((model: AIModel) => (
                                                        <DropdownMenuItem
                                                            key={model.id}
                                                            onClick={() => onModelChange?.(model)}
                                                            className={cn("hover:bg-accent/30 focus:bg-accent/30 rounded-md px-2 py-2 text-sm", selectedModel?.id === model.id && "bg-accent")}>
                                                            <div className="flex w-full items-center justify-between">
                                                                <div className="flex min-w-0 items-center gap-2">
                                                                    <span className="truncate">{model.display_name}</span>
                                                                    {!model.is_active && <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">inactive</span>}
                                                                </div>
                                                                <div className="text-xs text-muted-foreground">{model.provider}</div>
                                                            </div>
                                                        </DropdownMenuItem>
                                                    ))}
                                                </div>
                                            )}
                                        </div>

                                        {/* Bottom controls */}
                                        <div className="bg-popover sticky bottom-0 flex items-center justify-between rounded-b-lg pt-1.5 pr-2.5 pb-1 pl-1 sm:inset-x-0">
                                            <div className="border-chat-border absolute inset-x-3 top-0 border-b"></div>
                                            <Button
                                                variant="ghost"
                                                className="focus-visible:ring-ring justify-center rounded-md font-medium whitespace-nowrap transition-colors focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 hover:bg-muted/40 hover:text-foreground h-9 px-4 py-2 text-muted-foreground flex items-center gap-2 pl-2 text-sm"
                                                onClick={() => setShowAllModels((prev: boolean) => !prev)}>
                                                {showAllModels ? <ChevronLeft className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />}
                                                <span>{showAllModels ? "Favorites" : "Show all"}</span>
                                                <div className="h-2 w-2 rounded-full bg-pink-500"></div>
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                className="focus-visible:ring-ring inline-flex items-center justify-center font-medium whitespace-nowrap transition-colors focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 hover:bg-muted/40 hover:text-foreground h-8 rounded-md text-xs text-muted-foreground relative gap-2 px-2"
                                                aria-label="Filter models"
                                                type="button">
                                                <Search className="h-4 w-4" />
                                            </Button>
                                        </div>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </div>

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
