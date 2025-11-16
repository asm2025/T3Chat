import { useState, useEffect, useRef } from "react";
import { AuthProvider, useAuth } from "@/lib/auth-context";
import { ThemeProvider } from "@/components/theme-provider";
import { LoginForm } from "@/components/login-form";
import { Settings } from "@/pages/Settings";
import { Chat } from "@/pages/Chat";
import { Profile } from "@/pages/Profile";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar";
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "@/components/ui/resizable";
import { Sidebar } from "@/components/sidebar";
import { MobileWarningBanner } from "@/components/mobile-warning-banner";
import { FloatingToolbar } from "@/components/floating-toolbar";
import { useIsMobile } from "@/hooks/use-mobile";

const SIDEBAR_WIDTH_STORAGE_KEY = "t3chat-sidebar-width";
const DEFAULT_SIDEBAR_WIDTH = 20; // 20% of viewport width
const MIN_SIDEBAR_WIDTH = 5; // 5% of viewport width
const MAX_SIDEBAR_WIDTH = 95; // 95% of viewport width

function AppContent() {
    const { user, loading, profileLoading } = useAuth();
    const [showLoginForAnonymous, setShowLoginForAnonymous] = useState(false);
    const [sidebarOpen, setSidebarOpen] = useState(true);
    const [sidebarWidth, setSidebarWidth] = useState(DEFAULT_SIDEBAR_WIDTH);
    const isMobile = useIsMobile();
    const saveTimeoutRef = useRef<number | null>(null);

    // Load sidebar width from localStorage on mount
    useEffect(() => {
        try {
            const savedWidth = localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY);
            if (savedWidth) {
                const widthValue = parseFloat(savedWidth);
                if (!isNaN(widthValue) && widthValue >= MIN_SIDEBAR_WIDTH && widthValue <= MAX_SIDEBAR_WIDTH) {
                    setSidebarWidth(widthValue);
                }
            }
        } catch (error) {
            console.error("Failed to load sidebar width from localStorage:", error);
        }
    }, []);

    // Save sidebar width to localStorage (debounced)
    const handleSidebarResize = (sizes: number[]) => {
        // Only update width when sidebar is open and we have the expected number of panels
        // When sidebar is collapsed, sizes.length will be 1 (only main panel)
        // When sidebar is open, sizes.length will be 2 (sidebar + main panel)
        if (!sidebarOpen || sizes.length < 2) {
            return;
        }

        const newWidth = sizes[0];
        // Validate the width is within bounds
        if (newWidth >= MIN_SIDEBAR_WIDTH && newWidth <= MAX_SIDEBAR_WIDTH && newWidth !== sidebarWidth) {
            setSidebarWidth(newWidth);

            // Debounce localStorage saves
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
            saveTimeoutRef.current = window.setTimeout(() => {
                try {
                    localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, newWidth.toString());
                } catch (error) {
                    console.error("Failed to save sidebar width to localStorage:", error);
                }
            }, 150);
        }
    };

    // Cleanup timeout on unmount
    useEffect(() => {
        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
        };
    }, []);

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
        <SidebarProvider open={sidebarOpen} onOpenChange={setSidebarOpen}>
            <div className="flex min-h-screen w-full bg-background">
                <MobileWarningBanner />
                {shouldShowLogin ? (
                    <main className="flex flex-1 flex-col items-center justify-center px-4 py-12 sm:px-6">
                        <LoginForm />
                    </main>
                ) : (
                    <>
                        {/* Mobile: Use ShadCN Sidebar with Sheet */}
                        {isMobile ? (
                            <>
                                {!sidebarOpen && (
                                    <div className="fixed left-2 top-2 z-50 md:hidden">
                                        <SidebarTrigger />
                                    </div>
                                )}
                                <Sidebar variant="sidebar" collapsible="offcanvas" onSignInClick={handleSignInClick} />
                                <SidebarInset className="h-screen">
                                    <Routes>
                                        <Route path="/:chatId?" element={<Chat />} />
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
                                </SidebarInset>
                            </>
                        ) : (
                            // Desktop: Use ResizablePanelGroup with ShadCN Sidebar
                            <ResizablePanelGroup key={sidebarOpen ? "open" : "closed"} direction="horizontal" className="min-h-screen" onLayout={handleSidebarResize}>
                                {sidebarOpen && (
                                    <>
                                        <ResizablePanel
                                            id="sidebar-panel"
                                            defaultSize={Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, sidebarWidth))}
                                            minSize={MIN_SIDEBAR_WIDTH}
                                            maxSize={MAX_SIDEBAR_WIDTH}
                                            className="hidden md:flex flex-shrink-0 overflow-hidden">
                                            <Sidebar variant="sidebar" collapsible="none" className="h-full w-full flex flex-col" style={{ width: "100%", minWidth: 0 }} onSignInClick={handleSignInClick} />
                                        </ResizablePanel>
                                        <ResizableHandle withHandle className="hidden md:flex w-1 bg-transparent hover:bg-border transition-colors cursor-col-resize" />
                                    </>
                                )}
                                <ResizablePanel
                                    id="main-panel"
                                    defaultSize={sidebarOpen ? Math.max(5, Math.min(95, 100 - Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, sidebarWidth)))) : 100}
                                    minSize={5}
                                    maxSize={sidebarOpen ? 95 : 100}
                                    className="flex-1">
                                    <SidebarInset className="h-screen overflow-hidden">
                                        <FloatingToolbar />
                                        <Routes>
                                            <Route path="/:chatId?" element={<Chat />} />
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
                                    </SidebarInset>
                                </ResizablePanel>
                            </ResizablePanelGroup>
                        )}
                    </>
                )}
            </div>
        </SidebarProvider>
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
