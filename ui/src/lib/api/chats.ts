import { fetchWithAuth, safeJsonParse } from '../serverComm';
import type { Chat, ChatWithMessages } from '@/types/chat';
import type { CreateChatRequest } from '@/types/api';

export async function listChats(page = 1, pageSize = 20): Promise<{ data: Chat[]; total: number }> {
  const response = await fetchWithAuth(`/api/v1/chats?page=${page}&page_size=${pageSize}`);
  return safeJsonParse(response);
}

export async function getChat(id: string): Promise<ChatWithMessages> {
  const response = await fetchWithAuth(`/api/v1/chats/${id}`);
  return safeJsonParse(response);
}

export async function createChat(data: CreateChatRequest): Promise<Chat> {
  const response = await fetchWithAuth('/api/v1/chats', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return safeJsonParse(response);
}

export async function updateChat(id: string, data: { title?: string }): Promise<Chat> {
  const response = await fetchWithAuth(`/api/v1/chats/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return safeJsonParse(response);
}

export async function deleteChat(id: string): Promise<void> {
  await fetchWithAuth(`/api/v1/chats/${id}`, {
    method: 'DELETE',
  });
}

