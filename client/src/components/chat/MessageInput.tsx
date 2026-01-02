import { useEffect, useRef, useState } from "react";
import { ArrowUp, Paperclip, Search } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { AIModel } from "@/types/model";

const TERMS_STORAGE_KEY = "t3chat.termsAcknowledged";

interface MessageInputProps {
    onSend: (content: string) => void;
    disabled?: boolean;
    models?: AIModel[];
    selectedModel?: AIModel | null;
    onModelSelect?: (model: AIModel) => void;
    modelSelectEnabled?: boolean;
    onContentChange?: (hasContent: boolean) => void;
    value?: string;
    onValueChange?: (value: string) => void;
}

export function MessageInput({ onSend, disabled, models = [], selectedModel, onModelSelect, modelSelectEnabled = true, onContentChange, value: controlledValue, onValueChange }: MessageInputProps) {
    const [internalContent, setInternalContent] = useState("");
    const isControlled = controlledValue !== undefined;
    const content = isControlled ? controlledValue : internalContent;
    const setContent = (newContent: string) => {
        if (isControlled) {
            onValueChange?.(newContent);
        } else {
            setInternalContent(newContent);
        }
    };
    const [hasAcceptedPolicies, setHasAcceptedPolicies] = useState(() => {
        if (typeof window === "undefined") {
            return false;
        }

        try {
            return localStorage.getItem(TERMS_STORAGE_KEY) === "true";
        } catch {
            return false;
        }
    });
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    useEffect(() => {
        if (typeof window === "undefined") {
            return;
        }
        try {
            if (localStorage.getItem(TERMS_STORAGE_KEY) === "true") {
                setHasAcceptedPolicies(true);
            }
        } catch {
            // Ignore storage read errors
        }
    }, []);

    useEffect(() => {
        const textarea = textareaRef.current;
        if (!textarea) {
            return;
        }
        textarea.style.height = "48px";
        const nextHeight = Math.min(textarea.scrollHeight, 240);
        textarea.style.height = `${nextHeight}px`;
    }, [content]);

    const sendMessage = () => {
        const trimmed = content.trim();
        if (!trimmed || disabled) {
            return;
        }

        onSend(trimmed);
        setContent("");

        if (textareaRef.current) {
            textareaRef.current.style.height = "48px";
        }
    };

    const handleSubmit = (event: React.FormEvent) => {
        event.preventDefault();
        sendMessage();
    };

    const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            sendMessage();
        }
    };

    const handleAcceptPolicies = () => {
        setHasAcceptedPolicies(true);
        if (typeof window === "undefined") {
            return;
        }

        try {
            localStorage.setItem(TERMS_STORAGE_KEY, "true");
        } catch {
            // Ignore storage write errors
        }
    };

    const isSubmitDisabled = disabled || !content.trim();

    return (
        <div className="pointer-events-none mx-4 shrink-0 sticky bottom-0 z-20">
            <div className="flex w-full max-w-3xl mx-auto flex-col text-center">
                {!hasAcceptedPolicies && (
                    <div className="pointer-events-auto mx-auto mb-2 w-full sm:w-auto">
                        <div className="border-secondary/40 bg-chat-background/70 text-secondary-foreground/80 flex flex-col items-center justify-center gap-3 rounded-t-md border p-4 text-sm backdrop-blur-md sm:flex-row sm:text-left">
                            <span>
                                Make sure you agree to our{" "}
                                <a href="/terms-of-service" className="text-foreground transition-colors hover:text-primary" target="_blank" rel="noreferrer">
                                    Terms
                                </a>{" "}
                                and our{" "}
                                <a href="/privacy-policy" className="text-foreground transition-colors hover:text-primary" target="_blank" rel="noreferrer">
                                    Privacy Policy
                                </a>
                            </span>
                            <button
                                type="button"
                                onClick={handleAcceptPolicies}
                                className="focus-visible:ring-ring inline-flex h-8 items-center justify-center gap-2 rounded-md bg-secondary/50 px-3 text-xs font-medium text-secondary-foreground transition-colors hover:bg-secondary focus-visible:ring-1 focus-visible:outline-none sm:ml-4">
                                I agree
                            </button>
                        </div>
                    </div>
                )}

                <div className="bg-chat-background pointer-events-auto border-reflect min-w-0 overflow-hidden rounded-t-[20px] bg-chat-input-background/80 p-2 pb-0">
                    <form
                        id="chat-input-form"
                        onSubmit={handleSubmit}
                        className="pointer-events-auto relative flex w-full min-w-0 flex-col items-stretch gap-2 rounded-t-xl border-t border-l border-r border-gray-300 dark:border-gray-600 bg-chat-input-background/90 px-3 pt-3 pb-safe-offset-3 text-secondary-foreground">
                        <div className="flex min-w-0 flex-row items-start">
                            <textarea
                                ref={textareaRef}
                                value={content}
                                onChange={(event) => {
                                    const newContent = event.target.value;
                                    setContent(newContent);
                                    onContentChange?.(newContent.trim().length > 0);
                                }}
                                onKeyDown={handleKeyDown}
                                placeholder="Type your message here..."
                                aria-label="Message input"
                                aria-describedby="chat-input-description"
                                autoComplete="off"
                                disabled={disabled}
                                className="text-foreground placeholder:text-secondary-foreground/60 w-full min-h-[48px] min-w-0 resize-none bg-transparent text-base leading-6 outline-none disabled:opacity-50"
                            />
                            <div id="chat-input-description" className="sr-only">
                                Press Enter to send, Shift + Enter for new line
                            </div>
                        </div>

                        <div className="mt-2 -mb-px flex w-full min-w-0 flex-row-reverse flex-wrap items-center justify-between gap-2">
                            <div className="-mt-0.5 -mr-0.5 flex shrink-0 items-center justify-center gap-2" aria-label="Message actions">
                                <button
                                    type="submit"
                                    disabled={isSubmitDisabled}
                                    aria-label={isSubmitDisabled ? "Message requires text" : "Send message"}
                                    className="focus-visible:ring-ring button-reflect inline-flex size-10 items-center justify-center rounded-lg bg-primary p-2 text-primary-foreground shadow-sm transition-colors focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 hover:bg-primary/90">
                                    <ArrowUp className="size-5" />
                                </button>
                            </div>

                            <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2 pr-2">
                                {modelSelectEnabled && (
                                    <div className="min-w-0 flex-1 sm:max-w-sm">
                                        <Select
                                            value={selectedModel?.id}
                                            onValueChange={(value) => {
                                                const model = models.find((m) => m.id === value);
                                                if (model && onModelSelect) onModelSelect(model);
                                            }}
                                            disabled={disabled}>
                                            <SelectTrigger className="focus-visible:ring-ring inline-flex min-w-0 w-full items-center justify-between gap-2 rounded-md px-2 py-1.5 text-left text-sm font-medium text-muted-foreground transition-colors hover:bg-muted/40 hover:text-foreground focus-visible:ring-1 focus-visible:outline-none h-auto border-none bg-transparent shadow-none [&>svg]:opacity-100">
                                                <SelectValue placeholder="Select a model" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {models.map((model) => (
                                                    <SelectItem key={model.id} value={model.id}>
                                                        {model.displayName}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                )}

                                <button
                                    type="button"
                                    disabled
                                    title="Web search is not available yet"
                                    className="focus-visible:ring-ring inline-flex items-center gap-2 rounded-full border border-secondary-foreground/10 px-3 py-1.5 text-xs text-muted-foreground transition-colors focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50">
                                    <Search className="size-4" aria-hidden="true" />
                                    <span className="hidden md:inline">Search</span>
                                </button>

                                <button
                                    type="button"
                                    disabled
                                    title="File attachments are coming soon"
                                    className="focus-visible:ring-ring inline-flex items-center gap-2 rounded-full border border-secondary-foreground/10 px-3 py-1.5 text-xs text-muted-foreground transition-colors focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50">
                                    <Paperclip className="size-4" aria-hidden="true" />
                                    <span className="sr-only">Attachments unavailable</span>
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    );
}
