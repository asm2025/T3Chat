import { Settings, MessageSquare } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { Sidebar, SidebarContent, SidebarFooter, SidebarMenu, SidebarMenuItem, SidebarMenuButton, SidebarGroup, SidebarGroupLabel, SidebarGroupContent } from "@/components/ui/sidebar";

export function AppSidebar() {
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path;

    return (
        <Sidebar collapsible="icon" className="sticky top-12 h-[calc(100vh-3rem)] z-40">
            <SidebarContent className="overflow-y-auto">
                <SidebarGroup>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            <SidebarMenuItem>
                                <SidebarMenuButton tooltip="Chat" isActive={location.pathname.startsWith("/chat")} asChild>
                                    <Link to="/chat">
                                        <MessageSquare className="w-4 h-4" />
                                        <span>Chat</span>
                                    </Link>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>

            <SidebarFooter>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton tooltip="Settings" isActive={isActive("/settings")} asChild>
                            <Link to="/settings">
                                <Settings className="w-4 h-4" />
                                <span>Settings</span>
                            </Link>
                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarFooter>
        </Sidebar>
    );
}
