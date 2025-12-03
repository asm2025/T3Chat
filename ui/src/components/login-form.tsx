import { useState, useEffect } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Separator } from "./ui/separator";
import { useAuth } from "@/lib/use-auth";
import { handleCallback, localLogin, isOidcEnabled } from "@/lib/auth";
import { toast } from "@/lib/toast";
import { Loader2 } from "lucide-react";

export function LoginForm() {
    const [isLoading, setIsLoading] = useState(false);
    const [showOidcOption, setShowOidcOption] = useState(false);
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
                    toast.success("Successfully logged in");
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
            toast.success("Successfully logged in");
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

    return (
        <Card className="w-full mx-auto max-w-md rounded-2xl border border-border bg-card shadow-lg">
            <CardHeader>
                <CardTitle>Sign In</CardTitle>
                <CardDescription>{showOidcOption && oidcEnabled ? "Sign in with your username and password or use your OIDC provider authentication" : "Sign in with your username and password"}</CardDescription>
            </CardHeader>

            <CardContent className="space-y-6">
                {!showOidcOption ? (
                    <form onSubmit={handleLocalLoginSubmit} className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="username">Email</Label>
                            <Input id="username" type="text" placeholder="Enter your username" value={username} onChange={(e) => setUsername(e.target.value)} required disabled={isLoading} autoComplete="username" />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="password">Password</Label>
                            <Input id="password" type="password" placeholder="Enter your password" value={password} onChange={(e) => setPassword(e.target.value)} required disabled={isLoading} autoComplete="current-password" />
                        </div>

                        <Button type="submit" className="w-full" disabled={isLoading}>
                            {isLoading ? (
                                <>
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                    Logging in...
                                </>
                            ) : (
                                "Login"
                            )}
                        </Button>

                        {oidcEnabled && (
                            <>
                                <div className="relative">
                                    <div className="absolute inset-0 flex items-center">
                                        <Separator />
                                    </div>
                                    <div className="relative flex justify-center text-xs uppercase">
                                        <span className="bg-card px-2 text-muted-foreground">Or</span>
                                    </div>
                                </div>

                                <Button type="button" variant="outline" className="w-full" onClick={() => setShowOidcOption(true)} disabled={isLoading}>
                                    Login with OIDC
                                </Button>
                            </>
                        )}
                    </form>
                ) : (
                    <>
                        <div className="text-center space-y-2">
                            <p className="text-sm text-muted-foreground">Click the button below to sign in with your organization's identity provider.</p>
                        </div>

                        <Button type="button" className="w-full" onClick={handleOidcLogin} disabled={isLoading}>
                            {isLoading ? (
                                <>
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                    Redirecting...
                                </>
                            ) : (
                                "Login with OIDC"
                            )}
                        </Button>

                        <div className="relative">
                            <div className="absolute inset-0 flex items-center">
                                <Separator />
                            </div>
                            <div className="relative flex justify-center text-xs uppercase">
                                <span className="bg-card px-2 text-muted-foreground">Or</span>
                            </div>
                        </div>

                        <Button type="button" variant="outline" className="w-full" onClick={() => setShowOidcOption(false)} disabled={isLoading}>
                            Login with Username & Password
                        </Button>
                    </>
                )}
            </CardContent>
        </Card>
    );
}
