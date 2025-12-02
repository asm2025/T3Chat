import { useEffect } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { handleCallback } from '@/lib/auth';
import { toast } from '@/lib/toast';
import { Card, CardContent } from '@/components/ui/card';
import { Loader2 } from 'lucide-react';

export function AuthCallback() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  useEffect(() => {
    const token = searchParams.get('token');
    if (token) {
      handleCallback(token)
        .then(() => {
          toast.success('Successfully logged in');
          navigate('/', { replace: true });
          // Reload page to refresh auth state
          window.location.reload();
        })
        .catch((error) => {
          toast.error('Failed to complete login', {
            description: error instanceof Error ? error.message : 'Unknown error',
          });
          navigate('/login', { replace: true });
        });
    } else {
      toast.error('No token provided');
      navigate('/login', { replace: true });
    }
  }, [searchParams, navigate]);

  return (
    <div className="flex items-center justify-center min-h-screen">
      <Card className="w-full max-w-md">
        <CardContent className="pt-6">
          <div className="flex flex-col items-center justify-center space-y-4">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">Completing login...</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

