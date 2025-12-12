import type { AiProvider } from './chat';

export interface AIModel {
  id: string;
  provider: AiProvider;
  model_id: string;
  display_name: string;
  description?: string;
  context_window: number;
  supports_streaming: boolean;
  supports_images: boolean;
  supports_functions: boolean;
  cost_per_token?: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

