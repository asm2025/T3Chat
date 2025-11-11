import { useState, useEffect } from "react";
import { AuthProvider, useAuth } from "@/lib/auth-context";
import { ThemeProvider } from "@/components/theme-provider";
import { LoginForm } from "@/components/login-form";
import { RedesignedSidebar } from "@/components/redesigned-sidebar";
import { Home } from "@/pages/Home";
import { Settings } from "@/pages/Settings";
import { Chat } from "@/pages/Chat";
import { Profile } from "@/pages/Profile";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";

function AppContent() {
    const { user, loading, profileLoading } = useAuth();
    const [showLoginForAnonymous, setShowLoginForAnonymous] = useState(false);

    // Reset login form state when user upgrades from anonymous to authenticated
    useEffect(() => {
        if (user && !user.isAnonymous) {
            setShowLoginForAnonymous(false);
        }
    }, [user]);

    // Show loading while authentication or profile is loading
    if (loading || profileLoading) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
        );
    }

    // Determine if login form should be shown
    const allowAnonymous = import.meta.env.VITE_ALLOW_ANONYMOUS_USERS !== "false";

    let shouldShowLogin: boolean;

    if (allowAnonymous) {
        // Anonymous users are allowed - only show login if there's no user at all
        // OR if anonymous user clicked "Sign In" to upgrade
        shouldShowLogin = !user || (user.isAnonymous && showLoginForAnonymous);
    } else {
        // Anonymous users NOT allowed - show login if no user OR if user is anonymous
        // (force authentication with real credentials)
        shouldShowLogin = !user || user.isAnonymous;
    }

    const handleSignInClick = () => {
        setShowLoginForAnonymous(true);
    };

    return (
        <div className="flex min-h-screen w-full bg-background">
            {shouldShowLogin ? (
                <main className="flex flex-1 flex-col items-center justify-center px-4 py-12 sm:px-6">
                    <LoginForm />
                </main>
            ) : (
                <>
                    <RedesignedSidebar onSignInClick={handleSignInClick} />
                    <main className="flex-1 overflow-hidden">
                        <Routes>
                            <Route path="/" element={<Home onSignInClick={handleSignInClick} />} />
                            <Route path="/chat/:chatId?" element={<Chat />} />
                            <Route path="/profile" element={<Profile />} />
                            <Route path="/settings" element={<Settings />} />
                            <Route path="/history" element={<div className="h-screen overflow-y-auto border-l border-border bg-background p-6"><h1 className="text-2xl font-semibold">History & Sync</h1></div>} />
                            <Route path="/models" element={<div className="h-screen overflow-y-auto border-l border-border bg-background p-6"><h1 className="text-2xl font-semibold">Models</h1></div>} />
                            <Route path="/api-keys" element={<div className="h-screen overflow-y-auto border-l border-border bg-background p-6"><h1 className="text-2xl font-semibold">API Keys</h1></div>} />
                            <Route path="/attachments" element={<div className="h-screen overflow-y-auto border-l border-border bg-background p-6"><h1 className="text-2xl font-semibold">Attachments</h1></div>} />
                        </Routes>
                    </main>
                </>
            )}
        </div>
    );
}

function App() {
    return (
        <AuthProvider>
            <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange storageKey="t3chat-theme">
                <Router>
                    <AppContent />
                </Router>
            </ThemeProvider>
        </AuthProvider>
    );
}

export default App;
