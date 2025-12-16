import { useParams } from "react-router-dom";
import { ChatView } from "@/components/chat/ChatView";
import { MasterLayout } from "@/components/master-layout";

export function Chat() {
    const { conversationId } = useParams<{ conversationId?: string }>();

    return (
        <MasterLayout contentClassName="h-full">
            <div className="h-full">
                <ChatView conversationId={conversationId ?? null} />
            </div>
        </MasterLayout>
    );
}
