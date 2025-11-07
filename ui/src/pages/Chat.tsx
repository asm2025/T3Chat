import { useParams } from 'react-router-dom';
import { ChatView } from '@/components/chat/ChatView';
import { ChatList } from '@/components/chat/ChatList';

export function Chat() {
  const { chatId } = useParams<{ chatId?: string }>();
  
  return (
    <div className="flex h-[calc(100vh-3rem)]">
      <div className="w-64 border-r">
        <ChatList />
      </div>
      <div className="flex-1">
        <ChatView chatId={chatId || null} />
      </div>
    </div>
  );
}

