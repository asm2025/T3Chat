import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

interface MessageInputProps {
    onSend: (content: string) => void;
    disabled?: boolean;
}

export function MessageInput({ onSend, disabled }: MessageInputProps) {
    const [content, setContent] = useState("");

    const sendMessage = () => {
        const trimmed = content.trim();
        if (!trimmed || disabled) {
            return;
        }
        onSend(trimmed);
            setContent("");
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

    return (
        <form onSubmit={handleSubmit} className="border-t border-border bg-card p-4">
            <div className="flex gap-2">
                            <Textarea
                                value={content}
                    onChange={(event) => setContent(event.target.value)}
                                onKeyDown={handleKeyDown}
                    placeholder="Type your message..."
                                disabled={disabled}
                    rows={3}
                    className="flex-1"
                />
                <Button type="submit" disabled={disabled || !content.trim()}>
                    Send
                                        </Button>
                        </div>
                    </form>
    );
}

