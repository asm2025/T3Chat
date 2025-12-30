import { useParams } from "react-router-dom";
import { ChatView } from "@/components/chat/ChatView";
import { MasterLayout } from "@/components/master-layout";

export function Chat() {
    const { chatId } = useParams<{ chatId?: string }>();

    return (
        <MasterLayout contentClassName="h-full">
            <div className="h-full">
                <ChatView chatId={chatId ?? null} />
            </div>
        </MasterLayout>
    );
}
