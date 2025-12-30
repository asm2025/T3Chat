import { useState, useEffect, useRef } from "react";
import { ThemeProvider } from "@/components/theme-provider";
import { AuthProvider } from "@/lib/auth-context";
import { useAuth } from "@/lib/use-auth";
import { ProtectedRoute } from "@/components/protected-route";
import { AdminRoute } from "@/components/admin-route";
import { Settings } from "@/pages/Settings";
import { Chat } from "@/pages/Chat";
import { Profile } from "@/pages/Profile";
import { Models } from "@/pages/Models";
import { Home } from "@/pages/Home";
import { About } from "@/pages/About";
import { Health } from "@/pages/Health";
import { AdminDashboard } from "@/pages/admin/dashboard";
import { AdminUsers } from "@/pages/admin/users";
import { AdminProviders } from "@/pages/admin/providers";
import { AdminModels } from "@/pages/admin/models";
import { AuthCallback } from "@/pages/AuthCallback";
import { Login } from "@/pages/Login";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar";
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "@/components/ui/resizable";
import { Sidebar } from "@/components/sidebar";
import { MobileWarningBanner } from "@/components/mobile-warning-banner";
import { FloatingToolbar } from "@/components/floating-toolbar";
import { useIsMobile } from "@/hooks/useMobile";
import { Toaster } from "@/components/ui/sonner";
import { ConfigWarningBanner } from "@/components/config-warning-banner";

const SIDEBAR_WIDTH_STORAGE_KEY = "t3chat-sidebar-width";
const DEFAULT_SIDEBAR_WIDTH = 20; // 20% of viewport width
const MIN_SIDEBAR_WIDTH = 5; // 5% of viewport width
const MAX_SIDEBAR_WIDTH = 95; // 95% of viewport width

function AuthenticatedLayout() {
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

    return (
        <SidebarProvider open={sidebarOpen} onOpenChange={setSidebarOpen}>
            <div className="flex flex-col min-h-screen w-full justify-between bg-background dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors">
                <MobileWarningBanner />
                {/* Mobile: Use ShadCN Sidebar with Sheet */}
                {isMobile ? (
                    <>
                        {!sidebarOpen && (
                            <div className="fixed left-2 top-2 z-50 md:hidden">
                                <SidebarTrigger />
                            </div>
                        )}
                        <Sidebar variant="sidebar" collapsible="offcanvas" />
                        <SidebarInset className="h-screen">
                            <Routes>
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
                                <Route path="/models" element={<Models />} />
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
                                    className="hidden md:flex shrink-0 overflow-hidden">
                                    <Sidebar variant="sidebar" collapsible="none" className="h-full w-full flex flex-col" style={{ width: "100%", minWidth: 0 }} />
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
                                    <Route path="/models" element={<Models />} />
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
            </div>
        </SidebarProvider>
    );
}

function App() {
    const { loading } = useAuth();

    return (
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange storageKey="t3chat-theme">
            <Router>
                <ConfigWarningBanner />
                {loading ? (
                    <div className="flex items-center justify-center min-h-screen">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                    </div>
                ) : (
                    <Routes>
                        {/* Public routes */}
                        <Route path="/" element={<Home />} />
                        <Route path="/about" element={<About />} />
                        <Route path="/health" element={<Health />} />
                        <Route path="/login" element={<Login />} />
                        <Route path="/auth/callback" element={<AuthCallback />} />

                        {/* Admin routes */}
                        <Route
                            path="/admin"
                            element={
                                <AdminRoute>
                                    <AdminDashboard />
                                </AdminRoute>
                            }
                        />
                        <Route
                            path="/admin/users"
                            element={
                                <AdminRoute>
                                    <AdminUsers />
                                </AdminRoute>
                            }
                        />
                        <Route
                            path="/admin/providers"
                            element={
                                <AdminRoute>
                                    <AdminProviders />
                                </AdminRoute>
                            }
                        />
                        <Route
                            path="/admin/models"
                            element={
                                <AdminRoute>
                                    <AdminModels />
                                </AdminRoute>
                            }
                        />

                        {/* Protected routes */}
                        <Route
                            path="/*"
                            element={
                                <ProtectedRoute>
                                    <AuthenticatedLayout />
                                </ProtectedRoute>
                            }
                        />
                    </Routes>
                )}
                <Toaster />
            </Router>
        </ThemeProvider>
    );
}

function AppWithAuth() {
    return (
        <AuthProvider>
            <App />
        </AuthProvider>
    );
}

export default AppWithAuth;
