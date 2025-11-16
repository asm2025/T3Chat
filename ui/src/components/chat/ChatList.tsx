import { useMemo, useState } from 'react';
import { useChats } from '@/hooks/useChats';
import { useNavigate, useParams } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Plus, Search } from 'lucide-react';
import * as api from '@/lib/serverComm';

export function ChatList() {
  const { chats, loading, refresh } = useChats();
  const navigate = useNavigate();
  const { chatId } = useParams<{ chatId?: string }>();
  const [query, setQuery] = useState('');

  const filteredChats = useMemo(() => {
    if (!query.trim()) {
      return chats;
    }
    return chats.filter(chat =>
      chat.title.toLowerCase().includes(query.toLowerCase())
    );
  }, [chats, query]);

  const handleNewChat = async () => {
    try {
      const newChat = await api.createChat({
        model_provider: 'openai',
        model_id: 'gpt-3.5-turbo',
      });
      await refresh();
      navigate(`/${newChat.id}`);
    } catch (error) {
      console.error('Failed to create chat', error);
    }
  };

  if (loading) return <div className="p-4">Loading chats...</div>;

  return (
    <aside className="flex h-full flex-col gap-4 bg-card p-4">
      <div className="space-y-3 rounded-xl border border-border bg-background p-4 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">Workspace</p>
            <p className="text-sm font-medium text-foreground">Recent Chats</p>
          </div>
          <Button
            size="icon"
            variant="outline"
            onClick={handleNewChat}
            className="size-9 rounded-full border-border bg-background shadow-sm"
          >
            <Plus className="h-4 w-4" />
            <span className="sr-only">Start a new chat</span>
          </Button>
        </div>
        <div className="relative">
          <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search conversations"
            className="h-10 rounded-full border border-border bg-background pl-10 text-sm shadow-sm"
          />
        </div>
      </div>
      <div className="flex-1 overflow-y-auto rounded-xl border border-border bg-background p-2.5 shadow-sm">
        {filteredChats.length === 0 ? (
          <div className="flex h-full items-center justify-center rounded-lg border border-dashed border-border px-4 py-6 text-center text-xs text-muted-foreground">
            {query ? 'No conversations match your search.' : 'Start a new chat to begin your first conversation.'}
          </div>
        ) : (
          <div className="space-y-2">
            {filteredChats.map(chat => {
              const active = chat.id === chatId;
              return (
                <button
                  key={chat.id}
                  onClick={() => navigate(`/${chat.id}`)}
                  className={`w-full rounded-xl border px-3.5 py-2.5 text-left transition ${
                    active
                      ? 'border-foreground/20 bg-foreground text-background shadow-sm'
                      : 'border-transparent bg-card text-foreground hover:border-border hover:bg-background'
                  }`}
                >
                  <div className="flex items-center justify-between text-sm font-medium">
                    <span className="truncate">{chat.title}</span>
                    <span className={`text-xs ${active ? 'text-background/60' : 'text-muted-foreground'}`}>
                      {new Date(chat.updated_at).toLocaleDateString()}
                    </span>
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

