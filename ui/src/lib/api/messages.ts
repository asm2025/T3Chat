import { fetchWithAuth, safeJsonParse } from '../serverComm';
import type { Message } from '@/types/chat';
import type { CreateMessageRequest } from '@/types/api';

export async function getMessages(chatId: string): Promise<Message[]> {
  const response = await fetchWithAuth(`/api/v1/chats/${chatId}/messages`);
  return safeJsonParse(response);
}

export async function createMessage(chatId: string, data: CreateMessageRequest): Promise<Message> {
  const response = await fetchWithAuth(`/api/v1/chats/${chatId}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return safeJsonParse(response);
}

