import { useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Plus, Search } from "lucide-react";
import { useChats } from "@/hooks/useChats";
import { useModels } from "@/hooks/useModels";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";

export function ChatList() {
    const { chats, loading, refresh } = useChats();
    const { models } = useModels();
    const navigate = useNavigate();
    const { conversationId } = useParams<{ conversationId?: string }>();
    const [query, setQuery] = useState("");

    const filteredChats = useMemo(() => {
        if (!query.trim()) {
            return chats;
        }
        return chats.filter((chat) => chat.title.toLowerCase().includes(query.toLowerCase()));
    }, [chats, query]);

    const handleNewChat = async () => {
        // Logic to simply navigate to the root chat page which shows "New Chat" view
        // The actual creation happens when sending a message.
        // Or if we want to force a blank state:
        navigate("/chat");
    };

    if (loading) {
        return <div className="p-4 text-sm text-muted-foreground">Loading chats…</div>;
    }

    return (
        <div className="flex h-full flex-col gap-2 py-2">
            <div className="px-3">
                <Button onClick={handleNewChat} className="w-full justify-start gap-2" size="default">
                    <Plus className="h-4 w-4" />
                    <span>New Chat</span>
                </Button>
            </div>

            <div className="px-3">
                <div className="relative">
                    <Search className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                        value={query}
                        onChange={(event) => setQuery(event.target.value)}
                        placeholder="Search threads..."
                        className="h-9 rounded-md bg-muted/50 pl-9 text-sm shadow-none focus-visible:ring-1"
                    />
                </div>
            </div>

            <div className="flex-1 overflow-y-auto px-2">
                {filteredChats.length === 0 ? (
                    <div className="px-2 py-8 text-center text-xs text-muted-foreground">
                        {query ? "No conversations match your search." : "No chats yet."}
                    </div>
                ) : (
                    <div className="space-y-1">
                        {filteredChats.map((chat) => {
                            const active = chat.id === conversationId;
                            return (
                                <button
                                    key={chat.id}
                                    onClick={() => navigate(`/chat/${chat.id}`)}
                                    className={`flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors ${
                                        active ? "bg-accent text-accent-foreground font-medium" : "text-muted-foreground hover:bg-muted hover:text-foreground"
                                    }`}>
                                    <div className="flex-1 truncate">{chat.title || "Untitled Chat"}</div>
                                </button>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
}
