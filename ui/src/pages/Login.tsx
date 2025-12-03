import { LoginForm } from "@/components/login-form";
import { MasterLayout } from "@/components/MasterLayout";

export function Login() {
  return (
    <MasterLayout contentClassName="px-0">
      <div className="flex min-h-[calc(100vh-10rem)] items-center justify-center px-4 py-8">
        <div className="w-full max-w-xl space-y-6 text-center">
          <div className="space-y-2">
            <p className="text-sm uppercase tracking-[0.3em] text-muted-foreground">
              Welcome back
            </p>
            <h1 className="text-3xl font-semibold">Sign in to T3Chat</h1>
            <p className="text-muted-foreground">
              Access your chats, presets, and admin tools from a single place.
            </p>
          </div>
          <LoginForm />
        </div>
      </div>
    </MasterLayout>
  );
}

