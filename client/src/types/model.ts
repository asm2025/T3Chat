import type { AiProvider } from "./conversation";

export interface AIModel {
    id: string;
    provider: AiProvider;
    modelId: string;
    displayName: string;
    description?: string;
    contextWindow: number;
    supportsStreaming: boolean;
    supportsImages: boolean;
    supportsFunctions: boolean;
    costPerToken?: number;
    isActive: boolean;
    createdAt: string;
    updatedAt: string;
}
