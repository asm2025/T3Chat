import { useChats } from '@/hooks/useChats';
import { useNavigate } from 'react-router-dom';

export function ChatList() {
  const { chats, loading } = useChats();
  const navigate = useNavigate();

  if (loading) return <div className="p-4">Loading chats...</div>;

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto">
        {chats.length === 0 ? (
          <div className="p-4 text-sm text-muted-foreground text-center">
            No chats yet
          </div>
        ) : (
          chats.map(chat => (
            <div
              key={chat.id}
              onClick={() => navigate(`/chat/${chat.id}`)}
              className="p-4 border-b cursor-pointer hover:bg-accent"
            >
              <div className="font-medium">{chat.title}</div>
              <div className="text-sm text-muted-foreground">
                {new Date(chat.updated_at).toLocaleDateString()}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

