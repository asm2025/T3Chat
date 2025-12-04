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
        const defaultModel = models[0];
        try {
            const newChat = await t3ChatClient.createChat(
                defaultModel
                    ? {
                          model_provider: defaultModel.provider,
                          model_id: defaultModel.model_id,
                      }
                    : {
                          model_provider: "openai",
                          model_id: "gpt-3.5-turbo",
                      },
            );
            await refresh();
            navigate(`/chat/${newChat.id}`);
        } catch (err) {
            toast.error("Failed to create chat", { description: getErrorMessage(err) });
        }
    };

    if (loading) {
        return <div className="p-4 text-sm text-muted-foreground">Loading chats…</div>;
    }

    return (
        <aside className="flex h-full flex-col gap-4 rounded-xl border border-border bg-card p-4 shadow-sm">
            <div className="space-y-3 rounded-xl border border-border bg-background p-4 shadow-sm">
                <div className="flex items-center justify-between">
                    <div>
                        <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Workspace</p>
                        <p className="text-sm font-medium text-foreground">Your Chats</p>
                    </div>
                    <Button size="icon" variant="outline" onClick={handleNewChat} className="h-9 w-9 rounded-full border-border">
                        <Plus className="h-4 w-4" />
                        <span className="sr-only">Start a new chat</span>
                    </Button>
                </div>
                <div className="relative">
                    <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search conversations" className="h-10 rounded-full border border-border bg-background pl-10 text-sm shadow-sm" />
                </div>
            </div>
            <div className="flex-1 overflow-y-auto rounded-xl border border-border bg-background p-2.5 shadow-sm">
                {filteredChats.length === 0 ? (
                    <div className="flex h-full items-center justify-center rounded-lg border border-dashed border-border px-4 py-6 text-center text-xs text-muted-foreground">
                        {query ? "No conversations match your search." : "Start a new chat to begin your first conversation."}
                    </div>
                ) : (
                    <div className="space-y-2">
                        {filteredChats.map((chat) => {
                            const active = chat.id === conversationId;
                            return (
                                <button
                                    key={chat.id}
                                    onClick={() => navigate(`/chat/${chat.id}`)}
                                    className={`w-full rounded-xl border px-3.5 py-2.5 text-left transition ${
                                        active ? "border-foreground/20 bg-foreground text-background shadow-sm" : "border-transparent bg-card text-foreground hover:border-border hover:bg-background"
                                    }`}>
                                    <div className="flex items-center justify-between text-sm font-medium">
                                        <span className="truncate">{chat.title}</span>
                                        <span className={`text-xs ${active ? "text-background/70" : "text-muted-foreground"}`}>{new Date(chat.updated_at).toLocaleDateString()}</span>
                                    </div>
                                </button>
                            );
                        })}
                    </div>
                )}
            </div>
        </aside>
    );
}
