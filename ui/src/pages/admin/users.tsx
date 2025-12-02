import { useEffect, useState } from 'react';
import { AdminLayout } from '@/layouts/admin-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { api } from '@/lib/api-client';
import { toast } from '@/lib/toast';
import { Plus, Search, Edit, Trash2, Lock, Unlock, UserCheck, UserX } from 'lucide-react';

interface User {
  id: string;
  email: string;
  name?: string;
  roles?: string[];
  disabled: boolean;
  locked_out: boolean;
  created_at: string;
}

interface UsersResponse {
  data: User[];
  total: number;
  page: number;
  page_size: number;
}

export function AdminUsers() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const pageSize = 20;

  useEffect(() => {
    fetchUsers();
  }, [page, searchQuery]);

  const fetchUsers = async () => {
    try {
      setLoading(true);
      const params: Record<string, string | number> = {
        page,
        limit: pageSize,
      };
      if (searchQuery) {
        params.search = searchQuery;
      }
      const data = await api.get<UsersResponse>('/v1/admin/users', params);
      setUsers(data.data);
      setTotal(data.total);
    } catch (error) {
      toast.error('Failed to fetch users', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleEnable = async (userId: string) => {
    try {
      await api.post(`/v1/admin/users/${userId}/enable`);
      toast.success('User enabled');
      fetchUsers();
    } catch (error) {
      toast.error('Failed to enable user');
    }
  };

  const handleDisable = async (userId: string) => {
    try {
      await api.post(`/v1/admin/users/${userId}/disable`);
      toast.success('User disabled');
      fetchUsers();
    } catch (error) {
      toast.error('Failed to disable user');
    }
  };

  const handleLock = async (userId: string) => {
    try {
      await api.post(`/v1/admin/users/${userId}/lock`, { duration_minutes: 60 });
      toast.success('User locked');
      fetchUsers();
    } catch (error) {
      toast.error('Failed to lock user');
    }
  };

  const handleUnlock = async (userId: string) => {
    try {
      await api.post(`/v1/admin/users/${userId}/unlock`);
      toast.success('User unlocked');
      fetchUsers();
    } catch (error) {
      toast.error('Failed to unlock user');
    }
  };

  const handleDelete = async (userId: string) => {
    if (!confirm('Are you sure you want to delete this user?')) return;
    try {
      await api.delete(`/v1/admin/users/${userId}`);
      toast.success('User deleted');
      fetchUsers();
    } catch (error) {
      toast.error('Failed to delete user');
    }
  };

  return (
    <AdminLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Users</h1>
            <p className="text-muted-foreground mt-2">
              Manage user accounts and permissions
            </p>
          </div>
          <Button>
            <Plus className="mr-2 h-4 w-4" />
            Create User
          </Button>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>User List</CardTitle>
                <CardDescription>
                  {total} total users
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <div className="relative">
                  <Search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    placeholder="Search users..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-8 w-64"
                  />
                </div>
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
            ) : users.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No users found
              </div>
            ) : (
              <>
                <div className="space-y-2">
                  {users.map((user) => (
                    <div
                      key={user.id}
                      className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50"
                    >
                      <div className="flex-1">
                        <div className="flex items-center gap-2">
                          <p className="font-medium">{user.name || user.email}</p>
                          {user.disabled && (
                            <Badge variant="destructive">Disabled</Badge>
                          )}
                          {user.locked_out && (
                            <Badge variant="secondary">Locked</Badge>
                          )}
                          {user.roles?.includes('admin') && (
                            <Badge>Admin</Badge>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground">{user.email}</p>
                        <p className="text-xs text-muted-foreground">
                          Created: {new Date(user.created_at).toLocaleDateString()}
                        </p>
                      </div>
                      <div className="flex items-center gap-2">
                        {user.disabled ? (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleEnable(user.id)}
                          >
                            <UserCheck className="h-4 w-4" />
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleDisable(user.id)}
                          >
                            <UserX className="h-4 w-4" />
                          </Button>
                        )}
                        {user.locked_out ? (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleUnlock(user.id)}
                          >
                            <Unlock className="h-4 w-4" />
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleLock(user.id)}
                          >
                            <Lock className="h-4 w-4" />
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
                          onClick={() => handleDelete(user.id)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
                {total > pageSize && (
                  <div className="flex items-center justify-between mt-4">
                    <p className="text-sm text-muted-foreground">
                      Page {page} of {Math.ceil(total / pageSize)}
                    </p>
                    <div className="flex gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setPage(p => Math.max(1, p - 1))}
                        disabled={page === 1}
                      >
                        Previous
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setPage(p => Math.min(Math.ceil(total / pageSize), p + 1))}
                        disabled={page >= Math.ceil(total / pageSize)}
                      >
                        Next
                      </Button>
                    </div>
                  </div>
                )}
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </AdminLayout>
  );
}

