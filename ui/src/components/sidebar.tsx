import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useAuth } from "@/lib/auth-context";
import { signOut } from "firebase/auth";
import { auth } from "@/lib/firebase";
import { useNavigate } from "react-router-dom";
import { Search, User, History, Brain, Key, Paperclip, Settings, LogOut, PanelLeft } from "lucide-react";
import { useState } from "react";
import { Sidebar as ShadcnSidebar, SidebarHeader, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarMenu, SidebarInput, SidebarTrigger, useSidebar } from "@/components/ui/sidebar";
import { useChat } from "@/stores/appStore";

interface AppSidebarProps {
    onSignInClick?: () => void;
    variant?: "sidebar" | "floating" | "inset";
    collapsible?: "offcanvas" | "icon" | "none";
    className?: string;
    style?: React.CSSProperties;
}

export function Sidebar({ onSignInClick, variant = "sidebar", collapsible = "offcanvas", className, style }: AppSidebarProps) {
    const { user, logout, userProfile } = useAuth();
    const navigate = useNavigate();
    const [searchQuery, setSearchQuery] = useState("");
    const { open, toggleSidebar, isMobile } = useSidebar();
    const { clearChat } = useChat();

    const handleLogout = () => {
        logout();
        signOut(auth);
    };

    const isAnonymous = user?.isAnonymous ?? false;
    const displayName = userProfile?.display_name || user?.displayName || user?.email || "User";
    const userInitials = displayName
        .split(" ")
        .map((n) => n[0])
        .join("")
        .toUpperCase()
        .slice(0, 2);

    const handleNewChat = () => {
        clearChat();
        navigate("/");
    };

    const menuItems = [
        { icon: User, label: "Account", onClick: () => navigate("/profile") },
        { icon: History, label: "History & Sync", onClick: () => navigate("/history") },
        { icon: Brain, label: "Models", onClick: () => navigate("/models") },
        { icon: Key, label: "API Keys", onClick: () => navigate("/api-keys") },
        { icon: Paperclip, label: "Attachments", onClick: () => navigate("/attachments") },
        { icon: Settings, label: "Settings", onClick: () => navigate("/settings") },
    ];

    return (
        <ShadcnSidebar variant={variant} collapsible={collapsible} className={className} style={style}>
            <SidebarHeader>
                <div className="flex items-center gap-2">
                    <SidebarTrigger className="md:hidden" />
                    {!isMobile && open && (
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8 text-muted-foreground hover:bg-muted/40 hover:text-foreground"
                            onClick={toggleSidebar}>
                            <PanelLeft className="h-4 w-4" />
                            <span className="sr-only">Toggle Sidebar</span>
                        </Button>
                    )}
                    <button type="button" onClick={() => navigate("/")} className="flex flex-col flex-1 text-center leading-tight">
                        <span className="block text-xl font-semibold tracking-tight">T3.chat</span>
                    </button>
                </div>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupContent>
                        {/* New Chat Button */}
                        <Button onClick={handleNewChat} className="bg-foreground text-background hover:bg-foreground/90 w-full rounded-lg" size="default">
                            New Chat
                        </Button>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupContent>
                        {/* Search Chat History */}
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground pointer-events-none" />
                            <SidebarInput type="text" placeholder="Search your threads..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
                        </div>
                    </SidebarGroupContent>
                </SidebarGroup>
                <SidebarGroup>
                    <SidebarGroupContent>
                        <SidebarMenu>
                            {/* Placeholder for chat history items */}
                            {/* This will be populated with actual chat history */}
                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>
            </SidebarContent>
            <SidebarFooter>
                {user && !isAnonymous ? (
                    <Popover>
                        <PopoverTrigger asChild>
                            <Button variant="ghost" className="w-full justify-start gap-2 px-2 hover:bg-muted">
                                <Avatar className="h-8 w-8 border border-border bg-muted">
                                    <AvatarFallback className="text-xs font-medium">{userInitials}</AvatarFallback>
                                </Avatar>
                                <span className="flex-1 truncate text-left text-sm">{displayName}</span>
                            </Button>
                        </PopoverTrigger>
                        <PopoverContent align="start" className="w-64 p-2">
                            <div className="space-y-1">
                                {menuItems.map((item) => (
                                    <Button key={item.label} variant="ghost" className="w-full justify-start gap-2 px-3 py-2 text-sm font-normal" onClick={item.onClick}>
                                        <item.icon className="h-4 w-4" />
                                        {item.label}
                                    </Button>
                                ))}
                                <div className="my-1 h-px bg-border" />
                                <Button variant="ghost" className="w-full justify-start gap-2 px-3 py-2 text-sm font-normal text-destructive hover:bg-destructive/10 hover:text-destructive" onClick={handleLogout}>
                                    <LogOut className="h-4 w-4" />
                                    Logout
                                </Button>
                            </div>
                        </PopoverContent>
                    </Popover>
                ) : (
                    <Button variant="outline" className="w-full" onClick={onSignInClick}>
                        Login
                    </Button>
                )}
            </SidebarFooter>
        </ShadcnSidebar>
    );
}
