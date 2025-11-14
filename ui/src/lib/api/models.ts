import { fetchWithAuth, safeJsonParse } from '../serverComm';
import type { AIModel } from '@/types/model';

export async function listModels(): Promise<AIModel[]> {
  const response = await fetchWithAuth('/api/v1/models');
  return safeJsonParse(response);
}

export async function getModel(id: string): Promise<AIModel> {
  const response = await fetchWithAuth(`/api/v1/models/${id}`);
  return safeJsonParse(response);
}

