import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { api } from '@/lib/api-client';
import { toast } from '@/lib/toast';
import { Search, Brain, Zap, Image, Code } from 'lucide-react';
import { MasterLayout } from '@/components/MasterLayout';

interface AIModel {
  id: string;
  model_id: string;
  display_name: string;
  description?: string;
  provider_id: string;
  context_window: number;
  max_output_tokens?: number;
  supports_streaming: boolean;
  supports_images: boolean;
  supports_functions: boolean;
  supports_vision: boolean;
  is_paid: boolean;
}

export function Models() {
  const [models, setModels] = useState<AIModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedProvider, setSelectedProvider] = useState<string>('all');

  useEffect(() => {
    fetchModels();
  }, []);

  const fetchModels = async () => {
    try {
      setLoading(true);
      const data = await api.get<AIModel[]>('/v1/models');
      setModels(data);
    } catch (error) {
      toast.error('Failed to fetch models', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  // Get unique providers
  const providers = Array.from(new Set(models.map(m => m.provider_id)));

  // Filter models
  const filteredModels = models.filter(model => {
    const matchesSearch = 
      model.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      model.model_id.toLowerCase().includes(searchQuery.toLowerCase()) ||
      model.description?.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesProvider = selectedProvider === 'all' || model.provider_id === selectedProvider;
    
    return matchesSearch && matchesProvider;
  });

  return (
    <MasterLayout contentClassName="px-0">
      <div className="container mx-auto px-6 max-w-6xl">
        <div className="space-y-6">
          <div>
            <h1 className="text-3xl font-bold">AI Models</h1>
            <p className="text-muted-foreground mt-2">
              Browse and explore available AI models
            </p>
          </div>

          {/* Filters */}
          <Card>
            <CardContent className="pt-6">
              <div className="flex flex-col md:flex-row gap-4">
                <div className="relative flex-1">
                  <Search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    placeholder="Search models..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-8"
                  />
                </div>
                <select
                  value={selectedProvider}
                  onChange={(e) => setSelectedProvider(e.target.value)}
                  className="px-3 py-2 border rounded-md bg-background"
                >
                  <option value="all">All Providers</option>
                  {providers.map(provider => (
                    <option key={provider} value={provider}>
                      {provider.charAt(0).toUpperCase() + provider.slice(1)}
                    </option>
                  ))}
                </select>
              </div>
            </CardContent>
          </Card>

          {/* Models List */}
          {loading ? (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {[1, 2, 3, 4, 5, 6].map((i) => (
                <Card key={i}>
                  <CardHeader>
                    <Skeleton className="h-6 w-32" />
                  </CardHeader>
                  <CardContent>
                    <Skeleton className="h-4 w-full mb-2" />
                    <Skeleton className="h-4 w-3/4" />
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : filteredModels.length === 0 ? (
            <Card>
              <CardContent className="pt-6">
                <div className="text-center py-8 text-muted-foreground">
                  {searchQuery || selectedProvider !== 'all' 
                    ? 'No models found matching your filters'
                    : 'No models available'}
                </div>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {filteredModels.map((model) => (
                <Card key={model.id} className="hover:shadow-lg transition-shadow">
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <CardTitle className="text-lg">{model.display_name}</CardTitle>
                        <CardDescription className="mt-1 font-mono text-xs">
                          {model.model_id}
                        </CardDescription>
                      </div>
                      {model.is_paid && (
                        <Badge variant="secondary">Paid</Badge>
                      )}
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {model.description && (
                      <p className="text-sm text-muted-foreground line-clamp-2">
                        {model.description}
                      </p>
                    )}
                    
                    <div className="flex flex-wrap gap-2">
                      {model.supports_streaming && (
                        <Badge variant="outline" className="text-xs">
                          <Zap className="mr-1 h-3 w-3" />
                          Streaming
                        </Badge>
                      )}
                      {model.supports_images && (
                        <Badge variant="outline" className="text-xs">
                          <Image className="mr-1 h-3 w-3" />
                          Images
                        </Badge>
                      )}
                      {model.supports_functions && (
                        <Badge variant="outline" className="text-xs">
                          <Code className="mr-1 h-3 w-3" />
                          Functions
                        </Badge>
                      )}
                      {model.supports_vision && (
                        <Badge variant="outline" className="text-xs">
                          <Brain className="mr-1 h-3 w-3" />
                          Vision
                        </Badge>
                      )}
                    </div>

                    <div className="text-xs text-muted-foreground space-y-1">
                      <div className="flex justify-between">
                        <span>Provider:</span>
                        <span className="font-medium">{model.provider_id}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Context Window:</span>
                        <span className="font-medium">{model.context_window.toLocaleString()}</span>
                      </div>
                      {model.max_output_tokens && (
                        <div className="flex justify-between">
                          <span>Max Output:</span>
                          <span className="font-medium">{model.max_output_tokens.toLocaleString()}</span>
                        </div>
                      )}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>
    </MasterLayout>
  );
}

