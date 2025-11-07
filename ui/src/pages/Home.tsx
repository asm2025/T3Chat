import { useAuth } from '@/lib/auth-context';
import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { MessageSquare, ArrowRight } from 'lucide-react';

interface HomeProps {
  onSignInClick?: () => void;
}

export function Home({ onSignInClick }: HomeProps = {}) {
  const { user } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    // Redirect authenticated users to chat
    if (user && !user.isAnonymous) {
      navigate('/chat');
    }
  }, [user, navigate]);

  // Show welcome page for non-authenticated users
  if (!user || user.isAnonymous) {
    return (
      <div className="container mx-auto p-6 max-w-4xl">
        <div className="flex flex-col items-center justify-center min-h-[calc(100vh-12rem)] space-y-8">
          <div className="text-center space-y-4">
            <div className="flex justify-center mb-4">
              <div className="rounded-full bg-primary/10 p-6">
                <MessageSquare className="h-16 w-16 text-primary" />
              </div>
            </div>
            <h1 className="text-4xl font-bold tracking-tight">Welcome to T3Chat</h1>
            <p className="text-xl text-muted-foreground max-w-2xl">
              Your intelligent chat companion powered by AI. Start conversations, get answers, and explore the power of artificial intelligence.
            </p>
          </div>

          <Card className="w-full max-w-2xl">
            <CardHeader>
              <CardTitle>Get Started</CardTitle>
              <CardDescription>
                Sign in to start chatting with AI models and explore the features
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <h3 className="font-semibold">AI-Powered Conversations</h3>
                    <p className="text-sm text-muted-foreground">
                      Chat with state-of-the-art AI models and get intelligent responses to your questions.
                    </p>
                  </div>
                  <div className="space-y-2">
                    <h3 className="font-semibold">Multiple Models</h3>
                    <p className="text-sm text-muted-foreground">
                      Choose from a variety of AI models to suit your needs and preferences.
                    </p>
                  </div>
                </div>
                {onSignInClick && (
                  <Button onClick={onSignInClick} className="w-full" size="lg">
                    Sign In to Get Started
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  // This should not be reached due to redirect, but just in case
  return null;
} 