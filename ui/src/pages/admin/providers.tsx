import { useEffect, useState } from 'react';
import { AdminLayout } from '@/layouts/admin-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { api } from '@/lib/api-client';
import { toast } from '@/lib/toast';
import { Plus, Search, Edit, Trash2, Power, PowerOff } from 'lucide-react';

interface Provider {
  id: string;
  provider_id: string;
  display_name: string;
  description?: string;
  disabled: boolean;
  is_active: boolean;
  created_at: string;
}

interface ProvidersResponse {
  data: Provider[];
  total: number;
}

export function AdminProviders() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    fetchProviders();
  }, [searchQuery]);

  const fetchProviders = async () => {
    try {
      setLoading(true);
      const params: Record<string, string> = {};
      if (searchQuery) {
        params.search = searchQuery;
      }
      const data = await api.get<ProvidersResponse>('/v1/admin/providers', params);
      setProviders(data.data);
    } catch (error) {
      toast.error('Failed to fetch providers', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleEnable = async (providerId: string) => {
    try {
      await api.post(`/v1/admin/providers/${providerId}/enable`);
      toast.success('Provider enabled');
      fetchProviders();
    } catch (error) {
      toast.error('Failed to enable provider');
    }
  };

  const handleDisable = async (providerId: string) => {
    try {
      await api.post(`/v1/admin/providers/${providerId}/disable`);
      toast.success('Provider disabled');
      fetchProviders();
    } catch (error) {
      toast.error('Failed to disable provider');
    }
  };

  const handleDelete = async (providerId: string) => {
    if (!confirm('Are you sure you want to delete this provider?')) return;
    try {
      await api.delete(`/v1/admin/providers/${providerId}`);
      toast.success('Provider deleted');
      fetchProviders();
    } catch (error) {
      toast.error('Failed to delete provider');
    }
  };

  return (
    <AdminLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">AI Providers</h1>
            <p className="text-muted-foreground mt-2">
              Manage AI provider configurations
            </p>
          </div>
          <Button>
            <Plus className="mr-2 h-4 w-4" />
            Create Provider
          </Button>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>Provider List</CardTitle>
                <CardDescription>
                  {providers.length} providers
                </CardDescription>
              </div>
              <div className="relative">
                <Search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search providers..."
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
            ) : providers.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No providers found
              </div>
            ) : (
              <div className="space-y-2">
                {providers.map((provider) => (
                  <div
                    key={provider.id}
                    className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <p className="font-medium">{provider.display_name}</p>
                        {provider.disabled && (
                          <Badge variant="destructive">Disabled</Badge>
                        )}
                        {!provider.is_active && (
                          <Badge variant="secondary">Inactive</Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground">{provider.provider_id}</p>
                      {provider.description && (
                        <p className="text-sm text-muted-foreground mt-1">{provider.description}</p>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      {provider.disabled ? (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleEnable(provider.id)}
                        >
                          <Power className="h-4 w-4" />
                        </Button>
                      ) : (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDisable(provider.id)}
                        >
                          <PowerOff className="h-4 w-4" />
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
                        onClick={() => handleDelete(provider.id)}
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

