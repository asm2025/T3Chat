import type { AIModel } from '@/types/model';

export function ModelInfo({ model }: { model: AIModel }) {
  return (
    <div className="text-sm text-muted-foreground">
      {model.description && <p>{model.description}</p>}
      <div className="mt-2 space-y-1">
        <div>Context Window: {model.context_window.toLocaleString()} tokens</div>
        {model.cost_per_token && (
          <div>Cost: ${model.cost_per_token} per token</div>
        )}
        <div className="flex gap-2 mt-2">
          {model.supports_streaming && (
            <span className="px-2 py-1 bg-primary/10 rounded text-xs">Streaming</span>
          )}
          {model.supports_images && (
            <span className="px-2 py-1 bg-primary/10 rounded text-xs">Images</span>
          )}
          {model.supports_functions && (
            <span className="px-2 py-1 bg-primary/10 rounded text-xs">Functions</span>
          )}
        </div>
      </div>
    </div>
  );
}

