import { useAuth } from "@/lib/auth-context";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ChatPlaceholder } from "@/components/chat/ChatPlaceholder";
import { MessageInput } from "@/components/chat/MessageInput";
import { ModeToggle } from "@/components/mode-toggle";

interface HomeProps {
    onSignInClick?: () => void;
}

export function Home({ onSignInClick }: HomeProps = {}) {
    const { user, userProfile } = useAuth();
    const navigate = useNavigate();

    useEffect(() => {
        // Redirect authenticated users to chat
        if (user && !user.isAnonymous) {
            navigate("/chat");
        }
    }, [user, navigate]);

    const displayName = userProfile?.display_name || user?.displayName || user?.email?.split("@")[0];

    const handleSendMessage = (content: string) => {
        // For now, just navigate to chat. This can be enhanced later to create a chat with the message
        navigate("/chat");
    };

    const handlePromptClick = (prompt: string) => {
        // For now, just navigate to chat. This can be enhanced later
        navigate("/chat");
    };

    return (
        <div className="flex h-screen flex-col bg-background">
            {/* Mode Toggle at Top Right */}
            <div className="flex justify-end p-4 shrink-0">
                <ModeToggle />
            </div>
            {/* Main Content Area */}
            <div className="flex flex-1 flex-col overflow-hidden">
                <div className="flex-1 overflow-y-auto px-6 py-4">
                    <div className="mx-auto w-full max-w-4xl">
                        <ChatPlaceholder userName={displayName} onPromptClick={handlePromptClick} />
                    </div>
                </div>

                {/* Chat Input at Bottom - Pinned with no margins/padding */}
                <div className="w-full shrink-0 pb-1">
                    <MessageInput onSend={handleSendMessage} disabled={false} />
                </div>
            </div>
        </div>
    );
}
