import { fetchWithAuth } from '../serverComm';
import type { AIModel } from '@/types/model';

export async function listModels(): Promise<AIModel[]> {
  const response = await fetchWithAuth('/api/v1/models');
  return response.json();
}

export async function getModel(id: string): Promise<AIModel> {
  const response = await fetchWithAuth(`/api/v1/models/${id}`);
  return response.json();
}

