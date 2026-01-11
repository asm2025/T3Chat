import { useState, useEffect, type SVGProps } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Card, CardContent } from "./ui/card";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Separator } from "./ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import { Badge } from "./ui/badge";
import { useAuth } from "@/lib/use-auth";
import { handleCallback, localLogin, isOidcEnabled } from "@/lib/auth";
import { toast } from "@/lib/toast";
import { Loader2, LogIn, UserPlus } from "lucide-react";

export function LoginForm() {
    const [isLoading, setIsLoading] = useState(false);
    const [oidcEnabled, setOidcEnabled] = useState(false);
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [searchParams] = useSearchParams();
    const navigate = useNavigate();
    const { login } = useAuth();

    // Check if OIDC is enabled on mount
    useEffect(() => {
        isOidcEnabled().then(setOidcEnabled);
    }, []);

    // Handle OIDC callback
    useEffect(() => {
        const token = searchParams.get("token");
        if (token) {
            setIsLoading(true);
            handleCallback(token)
                .then(() => {
                    navigate("/", { replace: true });
                    // Reload page to refresh auth state
                    window.location.reload();
                })
                .catch((error) => {
                    toast.error("Failed to complete login", {
                        description: error.message,
                    });
                })
                .finally(() => {
                    setIsLoading(false);
                });
        }
    }, [searchParams, navigate]);

    const handleOidcLogin = () => {
        setIsLoading(true);
        login();
    };

    const handleLocalLoginSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsLoading(true);

        try {
            await localLogin(username, password);
            navigate("/", { replace: true });
            // Reload page to refresh auth state
            window.location.reload();
        } catch (error) {
            toast.error("Login failed", {
                description: error instanceof Error ? error.message : "Invalid username or password",
            });
        } finally {
            setIsLoading(false);
        }
    };

    const hasOidc = oidcEnabled;

    return (
        <Card className="relative mx-auto w-full max-w-md overflow-hidden rounded-[32px] border border-border/60 bg-card/90 text-left shadow-2xl backdrop-blur-xl supports-[backdrop-filter]:bg-card/80">
            <div className="pointer-events-none absolute inset-0 opacity-70">
                <div className="absolute -top-20 right-[-15%] h-64 w-64 rounded-full bg-primary/25 blur-[120px]" />
                <div className="absolute bottom-[-25%] left-[-10%] h-72 w-72 rounded-full bg-muted/60 blur-[140px] dark:bg-muted/30" />
            </div>

            <CardContent className="relative space-y-8 p-6 sm:p-10">
                <div className="space-y-2">
                    <p className="text-xs font-semibold uppercase tracking-[0.35em] text-primary/70">Authentication</p>
                    <h2 className="text-2xl font-semibold leading-tight">Choose how you'd like to access your account.</h2>
                    <p className="text-sm text-muted-foreground">Sign in to continue with your existing workspace or request a new invite.</p>
                </div>

                <Tabs defaultValue="signin" className="w-full">
                    <TabsList className="w-full justify-between rounded-full bg-muted/70 p-1 text-sm text-muted-foreground">
                        <TabsTrigger
                            value="register"
                            className="w-full rounded-full px-3 py-2 text-xs font-medium uppercase tracking-wide text-muted-foreground/80 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm">
                            <span className="flex items-center justify-center gap-2 text-sm font-semibold normal-case tracking-normal">
                                <UserPlus className="h-4 w-4" />
                                Register
                            </span>
                        </TabsTrigger>
                        <TabsTrigger
                            value="signin"
                            className="w-full rounded-full px-3 py-2 text-xs font-medium uppercase tracking-wide text-muted-foreground/80 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm">
                            <span className="flex items-center justify-center gap-2 text-sm font-semibold normal-case tracking-normal">
                                <LogIn className="h-4 w-4" />
                                Sign In
                            </span>
                        </TabsTrigger>
                    </TabsList>

                    <TabsContent value="register" className="mt-6 space-y-4">
                        <div className="rounded-2xl border border-dashed border-border/80 bg-muted/30 p-4 text-sm text-muted-foreground">Self-serve registrations are coming soon. Request access and we'll reach out as soon as it's available.</div>
                        <Button variant="outline" asChild className="h-12 w-full rounded-2xl text-base font-semibold">
                            <a href="mailto:support@t3chat.com?subject=T3Chat%20Access%20Request">Request access</a>
                        </Button>
                    </TabsContent>

                    <TabsContent value="signin" className="mt-6 space-y-6">
                        <p className="text-center text-sm text-muted-foreground">Sign in to your existing account</p>

                        {hasOidc && (
                            <>
                                <Button type="button" variant="outline" className="h-12 w-full justify-center rounded-2xl border border-border/80 bg-background text-base font-semibold shadow-sm" onClick={handleOidcLogin} disabled={isLoading}>
                                    {isLoading ? (
                                        <>
                                            <Loader2 className="h-4 w-4 animate-spin" />
                                            Redirecting...
                                        </>
                                    ) : (
                                        <>
                                            <MicrosoftIcon className="h-5 w-5" />
                                            Sign in with Microsoft
                                        </>
                                    )}
                                </Button>

                                <div className="relative py-2">
                                    <Separator className="bg-border/80" />
                                    <Badge className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full border border-border/80 bg-card px-3 py-1 text-[0.6rem] font-semibold tracking-[0.35em] text-muted-foreground">
                                        OR WITH EMAIL
                                    </Badge>
                                </div>
                            </>
                        )}

                        <form onSubmit={handleLocalLoginSubmit} className="space-y-4">
                            <div className="space-y-1.5">
                                <Label htmlFor="username" className="text-sm font-medium">
                                    Email or username
                                </Label>
                                <Input
                                    id="username"
                                    type="text"
                                    placeholder="you@company.com or your username"
                                    value={username}
                                    onChange={(e) => setUsername(e.target.value)}
                                    required
                                    disabled={isLoading}
                                    autoComplete="username"
                                    className="h-12 rounded-2xl border border-border/70 bg-background/70 px-4 text-base shadow-inner focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20"
                                />
                            </div>

                            <div className="space-y-1.5">
                                <Label htmlFor="password" className="text-sm font-medium">
                                    Password
                                </Label>
                                <Input
                                    id="password"
                                    type="password"
                                    placeholder="Enter your password"
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    required
                                    disabled={isLoading}
                                    autoComplete="current-password"
                                    className="h-12 rounded-2xl border border-border/70 bg-background/70 px-4 text-base shadow-inner focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20"
                                />
                            </div>

                            <Button
                                type="submit"
                                className="h-12 w-full rounded-2xl text-base font-semibold shadow-lg transition hover:translate-y-[1px] hover:shadow-md !bg-foreground !text-background hover:!bg-foreground/90 dark:!bg-white dark:!text-black"
                                disabled={isLoading}>
                                {isLoading ? (
                                    <>
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                        Signing in...
                                    </>
                                ) : (
                                    "Sign in"
                                )}
                            </Button>
                        </form>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    );
}

function MicrosoftIcon(props: SVGProps<SVGSVGElement>) {
    return (
        <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
            <rect width="10.5" height="10.5" fill="#F35325" />
            <rect x="12.5" width="10.5" height="10.5" fill="#81BC06" />
            <rect y="12.5" width="10.5" height="10.5" fill="#05A6F0" />
            <rect x="12.5" y="12.5" width="10.5" height="10.5" fill="#FFBA08" />
        </svg>
    );
}
