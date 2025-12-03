import { useEffect, useState, useCallback } from 'react';
import { AdminLayout } from '@/layouts/admin-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { api } from '@/lib/api-client';
import { toast } from '@/lib/toast';
import { Plus, Search, Edit, Trash2, Power, PowerOff, AlertTriangle } from 'lucide-react';

interface Model {
  id: string;
  model_id: string;
  display_name: string;
  description?: string;
  provider_id: string;
  disabled: boolean;
  is_active: boolean;
  deprecated_at?: string;
  created_at: string;
}

interface ModelsResponse {
  data: Model[];
  total: number;
}

export function AdminModels() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');

  const getErrorMessage = (err: unknown) =>
    err instanceof Error ? err.message : 'Unknown error';

  const fetchModels = useCallback(async () => {
    try {
      setLoading(true);
      const params: Record<string, string> = {};
      if (searchQuery) {
        params.search = searchQuery;
      }
      const data = await api.get<ModelsResponse>('/v1/admin/models', params);
      setModels(data.data);
    } catch (error) {
      toast.error('Failed to fetch models', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  }, [searchQuery]);

  useEffect(() => {
    void fetchModels();
  }, [fetchModels]);

  const handleEnable = async (modelId: string) => {
    try {
      await api.post(`/v1/admin/models/${modelId}/enable`);
      toast.success('Model enabled');
      fetchModels();
    } catch (err) {
      toast.error('Failed to enable model', {
        description: getErrorMessage(err),
      });
    }
  };

  const handleDisable = async (modelId: string) => {
    try {
      await api.post(`/v1/admin/models/${modelId}/disable`);
      toast.success('Model disabled');
      fetchModels();
    } catch (err) {
      toast.error('Failed to disable model', {
        description: getErrorMessage(err),
      });
    }
  };

  const handleDeprecate = async (modelId: string) => {
    if (!confirm('Are you sure you want to deprecate this model?')) return;
    try {
      await api.post(`/v1/admin/models/${modelId}/deprecate`);
      toast.success('Model deprecated');
      fetchModels();
    } catch (err) {
      toast.error('Failed to deprecate model', {
        description: getErrorMessage(err),
      });
    }
  };

  const handleDelete = async (modelId: string) => {
    if (!confirm('Are you sure you want to delete this model?')) return;
    try {
      await api.delete(`/v1/admin/models/${modelId}`);
      toast.success('Model deleted');
      fetchModels();
    } catch (err) {
      toast.error('Failed to delete model', {
        description: getErrorMessage(err),
      });
    }
  };

  return (
    <AdminLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">AI Models</h1>
            <p className="text-muted-foreground mt-2">
              Manage AI model configurations
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline">
              Scan Providers
            </Button>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Create Model
            </Button>
          </div>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>Model List</CardTitle>
                <CardDescription>
                  {models.length} models
                </CardDescription>
              </div>
              <div className="relative">
                <Search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search models..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-8 w-64"
                />
              </div>
            </div>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-2">
                {[1, 2, 3, 4, 5].map((i) => (
                  <Skeleton key={i} className="h-16 w-full" />
                ))}
              </div>
            ) : models.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No models found
              </div>
            ) : (
              <div className="space-y-2">
                {models.map((model) => (
                  <div
                    key={model.id}
                    className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <p className="font-medium">{model.display_name}</p>
                        {model.disabled && (
                          <Badge variant="destructive">Disabled</Badge>
                        )}
                        {!model.is_active && (
                          <Badge variant="secondary">Inactive</Badge>
                        )}
                        {model.deprecated_at && (
                          <Badge variant="outline">
                            <AlertTriangle className="mr-1 h-3 w-3" />
                            Deprecated
                          </Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground">{model.model_id}</p>
                      <p className="text-xs text-muted-foreground">Provider: {model.provider_id}</p>
                      {model.description && (
                        <p className="text-sm text-muted-foreground mt-1">{model.description}</p>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      {model.disabled ? (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleEnable(model.id)}
                        >
                          <Power className="h-4 w-4" />
                        </Button>
                      ) : (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDisable(model.id)}
                        >
                          <PowerOff className="h-4 w-4" />
                        </Button>
                      )}
                      {!model.deprecated_at && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDeprecate(model.id)}
                        >
                          <AlertTriangle className="h-4 w-4" />
                        </Button>
                      )}
                      <Button
                        variant="outline"
                        size="sm"
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => handleDelete(model.id)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </AdminLayout>
  );
}

