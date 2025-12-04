import { useParams } from "react-router-dom";
import { ChatView } from "@/components/chat/ChatView";
import { ChatList } from "@/components/chat/ChatList";
import { MasterLayout } from "@/components/master-layout";

export function Chat() {
    const { conversationId } = useParams<{ conversationId?: string }>();

    return (
        <MasterLayout contentClassName="h-full">
            <div className="h-full">
                <ChatView chatId={conversationId ?? null} />
            </div>
        </MasterLayout>
    );
}
