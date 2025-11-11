import { Settings, MessageSquare } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { Sidebar, SidebarContent, SidebarFooter, SidebarMenu, SidebarMenuItem, SidebarMenuButton, SidebarGroup, SidebarGroupContent } from "@/components/ui/sidebar";

export function AppSidebar() {
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path;

    return (
        <Sidebar collapsible="icon" className="sticky top-16 z-40 h-[calc(100vh-4rem)] bg-transparent">
            <SidebarContent className="mx-3 mt-4 overflow-y-auto rounded-xl border border-border bg-card p-2 shadow-sm">
                <SidebarGroup>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton
                                    tooltip="Chat"
                                    isActive={location.pathname.startsWith("/chat")}
                                    asChild
                                    className="rounded-xl text-sm font-medium"
                                >
                                    <Link to="/chat">
                                        <MessageSquare className="h-4 w-4" />
                                        <span>Chat</span>
                                    </Link>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>

            <SidebarFooter className="mx-3 mb-4 rounded-xl border border-border bg-card p-2 shadow-sm">
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton
                            tooltip="Settings"
                            isActive={isActive("/settings")}
                            asChild
                            className="rounded-xl text-sm font-medium"
                        >
                            <Link to="/settings">
                                <Settings className="h-4 w-4" />
                                <span>Settings</span>
                            </Link>
                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarFooter>
        </Sidebar>
    );
}
