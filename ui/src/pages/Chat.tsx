import { useParams } from 'react-router-dom';
import { ImprovedChatView } from '@/components/chat/ImprovedChatView';
import { MasterLayout } from '@/components/MasterLayout';

export function Chat() {
    const { conversationId } = useParams<{ conversationId?: string }>();

    return (
        <MasterLayout>
            <ImprovedChatView conversationId={conversationId || null} />
        </MasterLayout>
    );
}
