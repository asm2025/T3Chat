import { useState, useEffect, useRef } from "react";
import { ChatView } from "@/components/chat/ChatView";
import { MasterLayout } from "@/components/master-layout";
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar";
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "@/components/ui/resizable";
import { Sidebar } from "@/components/sidebar";
import { MobileWarningBanner } from "@/components/mobile-warning-banner";
import { FloatingToolbar } from "@/components/floating-toolbar";
import { useIsMobile } from "@/hooks/useMobile";

const SIDEBAR_WIDTH_STORAGE_KEY = "t3chat-sidebar-width";
const DEFAULT_SIDEBAR_WIDTH = 20;
const MIN_SIDEBAR_WIDTH = 5;
const MAX_SIDEBAR_WIDTH = 95;

export function Chat() {
    const [sidebarOpen, setSidebarOpen] = useState(true);
    const [sidebarWidth, setSidebarWidth] = useState(DEFAULT_SIDEBAR_WIDTH);
    const isMobile = useIsMobile();
    const saveTimeoutRef = useRef<number | null>(null);

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

    const handleSidebarResize = (sizes: number[]) => {
        if (!sidebarOpen || sizes.length < 2) {
            return;
        }

        const newWidth = sizes[0];
        if (newWidth >= MIN_SIDEBAR_WIDTH && newWidth <= MAX_SIDEBAR_WIDTH && newWidth !== sidebarWidth) {
            setSidebarWidth(newWidth);

            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
            saveTimeoutRef.current = window.setTimeout(() => {
                try {
                    localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, newWidth.toString());
                } catch (error) {
                    console.error("Failed to save sidebar width from localStorage:", error);
                }
            }, 150);
        }
    };

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
                {isMobile ? (
                    <>
                        {!sidebarOpen && (
                            <div className="fixed left-2 top-2 z-50 md:hidden">
                                <SidebarTrigger />
                            </div>
                        )}
                        <Sidebar variant="sidebar" collapsible="offcanvas" />
                        <SidebarInset className="h-screen">
                            <MasterLayout contentClassName="h-full">
                                <div className="h-full">
                                    <ChatView />
                                </div>
                            </MasterLayout>
                        </SidebarInset>
                    </>
                ) : (
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
                                <MasterLayout contentClassName="h-full">
                                    <div className="h-full">
                                        <ChatView />
                                    </div>
                                </MasterLayout>
                            </SidebarInset>
                        </ResizablePanel>
                    </ResizablePanelGroup>
                )}
            </div>
        </SidebarProvider>
    );
}
