import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import type { AIModel } from '@/types/model';

export function ModelSelector({
  models,
  selectedModel,
  onSelect,
}: {
  models: AIModel[];
  selectedModel: AIModel | null;
  onSelect: (model: AIModel) => void;
}) {
  return (
    <Select
      value={selectedModel?.id}
      onValueChange={(value) => {
        const model = models.find(m => m.id === value);
        if (model) onSelect(model);
      }}
    >
      <SelectTrigger className="w-full">
        <SelectValue placeholder="Select a model" />
      </SelectTrigger>
      <SelectContent>
        {models.map(model => (
          <SelectItem key={model.id} value={model.id}>
            {model.display_name} ({model.provider})
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

