import { useParams } from "react-router-dom";
import { ChatView } from "@/components/chat/ChatView";
import { MasterLayout } from "@/components/MasterLayout";

export function Chat() {
    const { conversationId } = useParams<{ conversationId?: string }>();

    return (
        <MasterLayout>
            <ChatView conversationId={conversationId ?? null} />
        </MasterLayout>
    );
}
