import { useState, useEffect } from "react";
import { AuthProvider, useAuth } from "@/lib/auth-context";
import { ThemeProvider } from "@/components/theme-provider";
import { LoginForm } from "@/components/login-form";
import { RedesignedSidebar } from "@/components/sidebar";
import { Home } from "@/pages/Home";
import { Settings } from "@/pages/Settings";
import { Chat } from "@/pages/Chat";
import { Profile } from "@/pages/Profile";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { PanelLeft, PanelRight } from "lucide-react";

function AppContent() {
    const { user, loading, profileLoading } = useAuth();
    const [showLoginForAnonymous, setShowLoginForAnonymous] = useState(false);
    const [sidebarOpen, setSidebarOpen] = useState(true);
    const [sidebarWidth, setSidebarWidth] = useState(256); // 16rem = 256px

    // Reset login form state when user upgrades from anonymous to authenticated
    useEffect(() => {
        if (user && !user.isAnonymous) {
            setShowLoginForAnonymous(false);
        }
    }, [user]);

    // Listen for sidebar toggle events from within the sidebar
    useEffect(() => {
        const handleToggle = () => setSidebarOpen((prev) => !prev);
        window.addEventListener('toggleSidebar', handleToggle);
        return () => window.removeEventListener('toggleSidebar', handleToggle);
    }, []);

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

    const handleResizeStart = (e: React.MouseEvent) => {
        e.preventDefault();
        const startX = e.clientX;
        const startWidth = sidebarWidth;

        const handleMouseMove = (event: MouseEvent) => {
            const diff = event.clientX - startX;
            const newWidth = Math.max(200, Math.min(500, startWidth + diff));
            setSidebarWidth(newWidth);
        };

        const handleMouseUp = () => {
            document.removeEventListener("mousemove", handleMouseMove);
            document.removeEventListener("mouseup", handleMouseUp);
        };

        document.addEventListener("mousemove", handleMouseMove);
        document.addEventListener("mouseup", handleMouseUp);
    };

    return (
        <div className="flex min-h-screen w-full bg-background">
            {shouldShowLogin ? (
                <main className="flex flex-1 flex-col items-center justify-center px-4 py-12 sm:px-6">
                    <LoginForm />
                </main>
            ) : (
                <>
                    {/* Sidebar Toggle Button */}
                    {!sidebarOpen && (
                        <div className="top-safe-offset-2 pointer-events-auto fixed left-2 z-50 flex flex-row gap-0.5 p-1">
                            <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setSidebarOpen(!sidebarOpen)}
                                className="h-8 w-8 rounded-md bg-white dark:bg-background border border-gray-300 dark:border-border shadow-sm hover:bg-gray-50 dark:hover:bg-accent"
                            >
                                <PanelRight className="h-4 w-4" />
                                <span className="sr-only">Toggle Sidebar</span>
                            </Button>
                        </div>
                    )}

                    {/* Sidebar */}
                    {sidebarOpen && (
                        <div
                            className="flex transition-all duration-200 ease-in-out border-r border-gray-300 dark:border-border bg-white dark:bg-background relative"
                            style={{ width: `${sidebarWidth}px`, minWidth: `${sidebarWidth}px` }}
                        >
                            <div className="flex-1 overflow-hidden">
                                <RedesignedSidebar onSignInClick={handleSignInClick} />
                            </div>
                            
                            {/* Resizer */}
                            <div
                                className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize bg-transparent hover:bg-gray-300 dark:hover:bg-border transition-colors z-10"
                                onMouseDown={handleResizeStart}
                            />
                        </div>
                    )}

                    {/* Main Content */}
                    <main className="flex-1 overflow-hidden bg-background">
                        <Routes>
                            <Route path="/" element={<Home onSignInClick={handleSignInClick} />} />
                            <Route path="/chat/:chatId?" element={<Chat />} />
                            <Route path="/profile" element={<Profile />} />
                            <Route path="/settings" element={<Settings />} />
                            <Route
                                path="/history"
                                element={
                                    <div className="h-screen overflow-y-auto border-l border-gray-300 dark:border-border bg-white dark:bg-background p-6">
                                        <h1 className="text-2xl font-semibold">History & Sync</h1>
                                    </div>
                                }
                            />
                            <Route
                                path="/models"
                                element={
                                    <div className="h-screen overflow-y-auto border-l border-gray-300 dark:border-border bg-white dark:bg-background p-6">
                                        <h1 className="text-2xl font-semibold">Models</h1>
                                    </div>
                                }
                            />
                            <Route
                                path="/api-keys"
                                element={
                                    <div className="h-screen overflow-y-auto border-l border-gray-300 dark:border-border bg-white dark:bg-background p-6">
                                        <h1 className="text-2xl font-semibold">API Keys</h1>
                                    </div>
                                }
                            />
                            <Route
                                path="/attachments"
                                element={
                                    <div className="h-screen overflow-y-auto border-l border-gray-300 dark:border-border bg-white dark:bg-background p-6">
                                        <h1 className="text-2xl font-semibold">Attachments</h1>
                                    </div>
                                }
                            />
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
