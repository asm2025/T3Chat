import { useEffect, useState } from "react";
import { AdminLayout } from "@/layouts/admin-layout";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api-client";

interface DashboardStats {
    total_users: number;
    active_users: number;
    total_providers: number;
    active_providers: number;
    total_models: number;
    active_models: number;
    total_chats: number;
    recent_activity?: Array<{
        type: string;
        description: string;
        timestamp: string;
    }>;
}

export function AdminDashboard() {
    const [stats, setStats] = useState<DashboardStats | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetchStats();
    }, []);

    const fetchStats = async () => {
        try {
            setLoading(true);
            setError(null);
            const data = await api.get<DashboardStats>("/admin/dashboard/stats");
            setStats(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to fetch dashboard stats");
        } finally {
            setLoading(false);
        }
    };

    if (loading) {
        return (
            <AdminLayout>
                <div className="space-y-6">
                    <div>
                        <Skeleton className="h-8 w-48 mb-2" />
                        <Skeleton className="h-4 w-96" />
                    </div>
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                        {[1, 2, 3, 4].map((i) => (
                            <Card key={i}>
                                <CardHeader>
                                    <Skeleton className="h-4 w-24" />
                                </CardHeader>
                                <CardContent>
                                    <Skeleton className="h-8 w-16" />
                                </CardContent>
                            </Card>
                        ))}
                    </div>
                </div>
            </AdminLayout>
        );
    }

    if (error) {
        return (
            <AdminLayout>
                <Card className="border-destructive">
                    <CardHeader>
                        <CardTitle className="text-destructive">Error</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-muted-foreground">{error}</p>
                    </CardContent>
                </Card>
            </AdminLayout>
        );
    }

    return (
        <AdminLayout>
            <div className="space-y-6">
                <div>
                    <h1 className="text-3xl font-bold">Admin Dashboard</h1>
                    <p className="text-muted-foreground mt-2">Overview of system statistics and activity</p>
                </div>

                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">Total Users</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{stats?.total_users ?? 0}</div>
                            <p className="text-xs text-muted-foreground">{stats?.active_users ?? 0} active</p>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">AI Providers</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{stats?.total_providers ?? 0}</div>
                            <p className="text-xs text-muted-foreground">{stats?.active_providers ?? 0} active</p>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">AI Models</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{stats?.total_models ?? 0}</div>
                            <p className="text-xs text-muted-foreground">{stats?.active_models ?? 0} active</p>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">Total Chats</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{stats?.total_chats ?? 0}</div>
                        </CardContent>
                    </Card>
                </div>

                {stats?.recent_activity && stats.recent_activity.length > 0 && (
                    <Card>
                        <CardHeader>
                            <CardTitle>Recent Activity</CardTitle>
                            <CardDescription>Latest system events</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-2">
                                {stats.recent_activity.map((activity, index) => (
                                    <div key={index} className="flex items-center justify-between py-2 border-b last:border-0">
                                        <div>
                                            <p className="text-sm font-medium">{activity.description}</p>
                                            <p className="text-xs text-muted-foreground">{activity.type}</p>
                                        </div>
                                        <p className="text-xs text-muted-foreground">{new Date(activity.timestamp).toLocaleString()}</p>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                )}
            </div>
        </AdminLayout>
    );
}
