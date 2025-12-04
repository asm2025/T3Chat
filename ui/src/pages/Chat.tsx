import { useParams } from "react-router-dom";
import { ChatView } from "@/components/chat/ChatView";
import { ChatList } from "@/components/chat/ChatList";
import { MasterLayout } from "@/components/master-layout";

export function Chat() {
    const { conversationId } = useParams<{ conversationId?: string }>();

    return (
        <MasterLayout>
            <div className="flex flex-col gap-4 lg:flex-row">
                <div className="lg:w-80">
                    <ChatList />
                </div>
                <div className="flex-1 min-h-[60vh]">
                    <ChatView chatId={conversationId ?? null} />
                </div>
            </div>
        </MasterLayout>
    );
}
