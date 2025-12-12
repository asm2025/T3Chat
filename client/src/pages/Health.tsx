import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { RefreshCw, CheckCircle2, XCircle, AlertCircle } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

interface HealthStatus {
  status: string;
  timestamp: string;
  database?: {
    status: string;
    response_time_ms?: number;
  };
  api?: {
    status: string;
    version?: string;
  };
}

export function Health() {
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastChecked, setLastChecked] = useState<Date | null>(null);

  const fetchHealth = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE_URL}/api/v1/health`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      const data = await response.json();
      setHealth(data);
      setLastChecked(new Date());
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch health status');
      setHealth(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchHealth();
    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchHealth, 30000);
    return () => clearInterval(interval);
  }, []);

  const getStatusBadge = (status: string) => {
    const statusLower = status.toLowerCase();
    if (statusLower === 'healthy' || statusLower === 'ok' || statusLower === 'up') {
      return (
        <Badge variant="default" className="bg-green-500">
          <CheckCircle2 className="mr-1 h-3 w-3" />
          {status}
        </Badge>
      );
    } else if (statusLower === 'degraded' || statusLower === 'warning') {
      return (
        <Badge variant="default" className="bg-yellow-500">
          <AlertCircle className="mr-1 h-3 w-3" />
          {status}
        </Badge>
      );
    } else {
      return (
        <Badge variant="destructive">
          <XCircle className="mr-1 h-3 w-3" />
          {status}
        </Badge>
      );
    }
  };

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-16 max-w-4xl">
        <div className="space-y-6">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-4xl font-bold">System Health</h1>
              <p className="text-muted-foreground mt-2">
                Monitor the status of T3Chat services and infrastructure
              </p>
            </div>
            <Button onClick={fetchHealth} disabled={loading} variant="outline">
              <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>

          {/* Error State */}
          {error && (
            <Card className="border-destructive">
              <CardHeader>
                <CardTitle className="text-destructive">Connection Error</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">{error}</p>
                <p className="text-sm text-muted-foreground mt-2">
                  Make sure the API server is running at {API_BASE_URL}
                </p>
              </CardContent>
            </Card>
          )}

          {/* Loading State */}
          {loading && !health && (
            <Card>
              <CardHeader>
                <Skeleton className="h-6 w-32" />
              </CardHeader>
              <CardContent className="space-y-4">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-3/4" />
              </CardContent>
            </Card>
          )}

          {/* Health Status */}
          {health && (
            <>
              {/* Overall Status */}
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle>Overall Status</CardTitle>
                    {getStatusBadge(health.status)}
                  </div>
                  {lastChecked && (
                    <CardDescription>
                      Last checked: {lastChecked.toLocaleTimeString()}
                    </CardDescription>
                  )}
                </CardHeader>
                <CardContent>
                  <p className="text-sm text-muted-foreground">
                    Timestamp: {new Date(health.timestamp).toLocaleString()}
                  </p>
                </CardContent>
              </Card>

              {/* Database Status */}
              {health.database && (
                <Card>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle>Database</CardTitle>
                      {getStatusBadge(health.database.status)}
                    </div>
                  </CardHeader>
                  <CardContent>
                    {health.database.response_time_ms !== undefined && (
                      <p className="text-sm text-muted-foreground">
                        Response time: {health.database.response_time_ms}ms
                      </p>
                    )}
                  </CardContent>
                </Card>
              )}

              {/* API Status */}
              {health.api && (
                <Card>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle>API</CardTitle>
                      {getStatusBadge(health.api.status)}
                    </div>
                  </CardHeader>
                  <CardContent>
                    {health.api.version && (
                      <p className="text-sm text-muted-foreground">
                        Version: {health.api.version}
                      </p>
                    )}
                  </CardContent>
                </Card>
              )}
            </>
          )}

          {/* Info Card */}
          <Card>
            <CardHeader>
              <CardTitle>About Health Checks</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">
                This page displays the real-time health status of the T3Chat API and its dependencies.
                The status is automatically refreshed every 30 seconds, or you can manually refresh
                using the button above.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

