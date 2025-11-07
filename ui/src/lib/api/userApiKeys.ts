import { fetchWithAuth } from '../serverComm';
import type { UserApiKey, CreateUserApiKeyRequest } from '@/types/api';

export async function listUserApiKeys(): Promise<UserApiKey[]> {
  const response = await fetchWithAuth('/api/v1/user-api-keys');
  return response.json();
}

export async function createUserApiKey(data: CreateUserApiKeyRequest): Promise<UserApiKey> {
  const response = await fetchWithAuth('/api/v1/user-api-keys', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function deleteUserApiKey(id: string): Promise<void> {
  await fetchWithAuth(`/api/v1/user-api-keys/${id}`, {
    method: 'DELETE',
  });
}

