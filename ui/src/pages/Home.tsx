import { useAuth } from "@/lib/auth-context";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ChatPlaceholder } from "@/components/chat/ChatPlaceholder";
import { MessageInput } from "@/components/chat/MessageInput";
import { MasterLayout } from "@/components/MasterLayout";

interface HomeProps {
    onSignInClick?: () => void;
}

export function Home({ onSignInClick: _onSignInClick }: HomeProps) {
    const { user, userProfile } = useAuth();
    const navigate = useNavigate();

    useEffect(() => {
        // Redirect authenticated users to chat
        if (user && !user.isAnonymous) {
            navigate("/chat");
        }
    }, [user, navigate]);

    const displayName = userProfile?.display_name || user?.displayName || user?.email?.split("@")[0];

    const handleSendMessage = (_content: string) => {
        // For now, just navigate to chat. This can be enhanced later to create a chat with the message
        navigate("/chat");
    };

    const handlePromptClick = (_prompt: string) => {
        // For now, just navigate to chat. This can be enhanced later
        navigate("/chat");
    };

    return (
        <MasterLayout bottomSection={<MessageInput onSend={handleSendMessage} disabled={false} />}>
            <div className="mx-auto w-full max-w-4xl">
                <ChatPlaceholder userName={displayName} onPromptClick={handlePromptClick} />
            </div>
        </MasterLayout>
    );
}
