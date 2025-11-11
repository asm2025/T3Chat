import { useAuth } from '@/lib/auth-context';
import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { ChatPlaceholder } from '@/components/chat/ChatPlaceholder';
import { MessageInput } from '@/components/chat/MessageInput';
import { ModeToggle } from '@/components/mode-toggle';

interface HomeProps {
  onSignInClick?: () => void;
}

export function Home({ onSignInClick }: HomeProps = {}) {
  const { user, userProfile } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    // Redirect authenticated users to chat
    if (user && !user.isAnonymous) {
      navigate('/chat');
    }
  }, [user, navigate]);

  const displayName = userProfile?.display_name || user?.displayName || user?.email?.split('@')[0];

  const handleSendMessage = (content: string) => {
    // For now, just navigate to chat. This can be enhanced later to create a chat with the message
    navigate('/chat');
  };

  const handlePromptClick = (prompt: string) => {
    // For now, just navigate to chat. This can be enhanced later
    navigate('/chat');
  };

  return (
    <div className="flex h-screen flex-col border-l border-border bg-background">
      {/* Mode Toggle at Top Right */}
      <div className="flex justify-end p-4">
        <ModeToggle />
      </div>

      {/* Main Content Area */}
      <div className="flex-1 overflow-hidden px-6">
        <ChatPlaceholder userName={displayName} onPromptClick={handlePromptClick} />
      </div>

      {/* Chat Input at Bottom */}
      <div className="p-6">
        <MessageInput onSend={handleSendMessage} disabled={false} />
      </div>
    </div>
  );
}
