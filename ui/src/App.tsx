import { useState, useEffect, useRef, useCallback, useMemo } from "react";
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
import { PanelRight } from "lucide-react";

const SIDEBAR_WIDTH_STORAGE_KEY = "t3chat-sidebar-width";
// Sidebar width constraints as percentages of viewport width
const DEFAULT_SIDEBAR_WIDTH_PERCENT = 20; // 20% of viewport width
const MIN_SIDEBAR_WIDTH_PERCENT = 5; // 5% of viewport width
const MAX_SIDEBAR_WIDTH_PERCENT = 95; // 95% of viewport width

// Helper function to get viewport width
const getViewportWidth = () => {
    return window.innerWidth || document.documentElement.clientWidth;
};

function AppContent() {
    const { user, loading, profileLoading } = useAuth();
    const [showLoginForAnonymous, setShowLoginForAnonymous] = useState(false);
    const [sidebarOpen, setSidebarOpen] = useState(true);
    // Store width as percentage, but convert to pixels for rendering
    const [sidebarWidthPercent, setSidebarWidthPercent] = useState(DEFAULT_SIDEBAR_WIDTH_PERCENT);
    const [viewportWidth, setViewportWidth] = useState(getViewportWidth());
    const sidebarRef = useRef<HTMLDivElement>(null);
    const isResizingRef = useRef(false);
    const animationFrameRef = useRef<number | null>(null);
    const saveTimeoutRef = useRef<number | null>(null);
    // Initialize ref with default pixel width (will be updated by useMemo)
    const currentWidthPxRef = useRef((getViewportWidth() * DEFAULT_SIDEBAR_WIDTH_PERCENT) / 100);

    // Calculate current pixel width from percentage (memoized for performance)
    const sidebarWidthPx = useMemo(() => {
        return (viewportWidth * sidebarWidthPercent) / 100;
    }, [viewportWidth, sidebarWidthPercent]);

    // Update pixel ref when pixel width changes
    useEffect(() => {
        currentWidthPxRef.current = sidebarWidthPx;
    }, [sidebarWidthPx]);

    // Load sidebar width from localStorage on mount
    useEffect(() => {
        try {
            const savedWidth = localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY);
            if (savedWidth) {
                const widthValue = parseFloat(savedWidth);
                if (!isNaN(widthValue)) {
                    // If value is > 100, it's likely old pixel value - convert to percentage
                    let widthPercent: number;
                    if (widthValue > 100) {
                        // Old pixel format - convert to percentage based on current viewport
                        const currentViewportWidth = getViewportWidth();
                        widthPercent = (widthValue / currentViewportWidth) * 100;
                    } else {
                        // New percentage format
                        widthPercent = widthValue;
                    }

                    // Clamp to valid range
                    const clampedPercent = Math.max(MIN_SIDEBAR_WIDTH_PERCENT, Math.min(MAX_SIDEBAR_WIDTH_PERCENT, widthPercent));

                    setSidebarWidthPercent(clampedPercent);
                }
            }
        } catch (error) {
            console.error("Failed to load sidebar width from localStorage:", error);
        }
    }, []);

    // Handle window resize to update viewport width and pixel width
    useEffect(() => {
        const handleResize = () => {
            const newViewportWidth = getViewportWidth();
            setViewportWidth(newViewportWidth);

            // Update the DOM directly if sidebar is open and we're not currently resizing
            if (sidebarRef.current && !isResizingRef.current) {
                const newWidthPx = (newViewportWidth * sidebarWidthPercent) / 100;
                sidebarRef.current.style.width = `${newWidthPx}px`;
                sidebarRef.current.style.minWidth = `${newWidthPx}px`;
                sidebarRef.current.style.maxWidth = `${newWidthPx}px`;
            }
        };

        // Throttle resize events for better performance
        let resizeTimeout: number | null = null;
        const throttledHandleResize = () => {
            if (resizeTimeout) {
                cancelAnimationFrame(resizeTimeout);
            }
            resizeTimeout = requestAnimationFrame(handleResize);
        };

        window.addEventListener("resize", throttledHandleResize);
        return () => {
            window.removeEventListener("resize", throttledHandleResize);
            if (resizeTimeout) {
                cancelAnimationFrame(resizeTimeout);
            }
        };
    }, [sidebarWidthPercent]);

    // Save sidebar width percentage to localStorage (debounced)
    const saveSidebarWidth = useCallback((widthPercent: number) => {
        if (saveTimeoutRef.current) {
            clearTimeout(saveTimeoutRef.current);
        }
        saveTimeoutRef.current = window.setTimeout(() => {
            try {
                // Store as percentage for better cross-device compatibility
                localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, widthPercent.toString());
            } catch (error) {
                console.error("Failed to save sidebar width to localStorage:", error);
            }
        }, 150);
    }, []);

    // Cleanup timeouts on unmount
    useEffect(() => {
        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
            if (animationFrameRef.current) {
                cancelAnimationFrame(animationFrameRef.current);
            }
        };
    }, []);

    // Reset login form state when user upgrades from anonymous to authenticated
    useEffect(() => {
        if (user && !user.isAnonymous) {
            setShowLoginForAnonymous(false);
        }
    }, [user]);

    // Listen for sidebar toggle events from within the sidebar
    useEffect(() => {
        const handleToggle = () => setSidebarOpen((prev) => !prev);
        window.addEventListener("toggleSidebar", handleToggle);
        return () => window.removeEventListener("toggleSidebar", handleToggle);
    }, []);

    // Handle resize start - defined as useCallback to maintain hook order
    const handleResizeStart = useCallback(
        (e: React.MouseEvent) => {
            e.preventDefault();
            e.stopPropagation();

            if (!sidebarRef.current) return;

            isResizingRef.current = true;
            const startX = e.clientX;
            const startWidthPx = currentWidthPxRef.current;
            const currentViewportWidth = getViewportWidth();

            // Calculate min/max pixel widths based on current viewport
            const minWidthPx = (currentViewportWidth * MIN_SIDEBAR_WIDTH_PERCENT) / 100;
            const maxWidthPx = (currentViewportWidth * MAX_SIDEBAR_WIDTH_PERCENT) / 100;

            // Prevent text selection during resize
            document.body.style.userSelect = "none";
            document.body.style.cursor = "col-resize";

            const updateWidth = (clientX: number) => {
                if (!sidebarRef.current || !isResizingRef.current) return;

                const diff = clientX - startX;
                // Clamp to percentage-based min/max
                const newWidthPx = Math.max(minWidthPx, Math.min(maxWidthPx, startWidthPx + diff));

                // Direct DOM manipulation for smoother updates (bypasses React re-render during drag)
                // This gives us 60fps smooth dragging without React re-renders
                sidebarRef.current.style.width = `${newWidthPx}px`;
                sidebarRef.current.style.minWidth = `${newWidthPx}px`;
                sidebarRef.current.style.maxWidth = `${newWidthPx}px`;

                // Update ref immediately for next frame calculation
                currentWidthPxRef.current = newWidthPx;
            };

            const handleMouseMove = (event: MouseEvent) => {
                if (!isResizingRef.current) return;

                // Use requestAnimationFrame for smooth, throttled updates
                if (animationFrameRef.current !== null) {
                    cancelAnimationFrame(animationFrameRef.current);
                }

                animationFrameRef.current = requestAnimationFrame(() => {
                    updateWidth(event.clientX);
                });
            };

            const handleMouseUp = () => {
                if (!isResizingRef.current) return;

                const finalWidthPx = currentWidthPxRef.current;
                isResizingRef.current = false;

                // Convert final pixel width to percentage based on current viewport
                const currentViewportWidth = getViewportWidth();
                const finalWidthPercent = (finalWidthPx / currentViewportWidth) * 100;

                // Update React state only after drag is complete (prevents re-renders during drag)
                setSidebarWidthPercent(finalWidthPercent);
                saveSidebarWidth(finalWidthPercent);

                // Restore body styles
                document.body.style.userSelect = "";
                document.body.style.cursor = "";

                // Clean up animation frame
                if (animationFrameRef.current !== null) {
                    cancelAnimationFrame(animationFrameRef.current);
                    animationFrameRef.current = null;
                }

                // Remove event listeners
                document.removeEventListener("mousemove", handleMouseMove);
                document.removeEventListener("mouseup", handleMouseUp);
            };

            // Add event listeners
            // Use capture phase for mouseup to catch it even if mouse leaves the window
            document.addEventListener("mousemove", handleMouseMove, { passive: true });
            document.addEventListener("mouseup", handleMouseUp, true);
        },
        [saveSidebarWidth],
    );

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
                    {/* Sidebar Toggle Button */}
                    {!sidebarOpen && (
                        <div className="top-safe-offset-2 pointer-events-auto fixed left-2 z-50 flex flex-row gap-0.5 p-1">
                            <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setSidebarOpen(!sidebarOpen)}
                                className="h-8 w-8 rounded-md bg-white dark:bg-background border border-gray-300 dark:border-border shadow-sm hover:bg-gray-50 dark:hover:bg-accent">
                                <PanelRight className="h-4 w-4" />
                                <span className="sr-only">Toggle Sidebar</span>
                            </Button>
                        </div>
                    )}

                    {/* Sidebar */}
                    {sidebarOpen && (
                        <div
                            ref={sidebarRef}
                            className="flex border-r border-gray-300 dark:border-border bg-white dark:bg-background relative select-none"
                            style={{
                                width: `${sidebarWidthPx}px`,
                                minWidth: `${sidebarWidthPx}px`,
                                maxWidth: `${sidebarWidthPx}px`,
                            }}>
                            <div className="flex-1 overflow-hidden">
                                <RedesignedSidebar onSignInClick={handleSignInClick} />
                            </div>

                            {/* Resizer */}
                            <div data-resizer className="absolute top-0 right-0 h-full w-2 cursor-col-resize bg-transparent hover:bg-gray-300 dark:hover:bg-border active:bg-gray-400 dark:active:bg-border z-10" onMouseDown={handleResizeStart} />
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
