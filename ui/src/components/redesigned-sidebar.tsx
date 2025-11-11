import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { useAuth } from "@/lib/auth-context";
import { signOut } from "firebase/auth";
import { auth } from "@/lib/firebase";
import { useNavigate, useLocation } from "react-router-dom";
import { 
  Plus, 
  Search, 
  User, 
  History, 
  Brain, 
  Key, 
  Paperclip, 
  Settings, 
  LogOut,
  MessageSquare
} from "lucide-react";
import { useState } from "react";

interface RedesignedSidebarProps {
  onSignInClick?: () => void;
}

export function RedesignedSidebar({ onSignInClick }: RedesignedSidebarProps = {}) {
  const { user, logout, userProfile } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [searchQuery, setSearchQuery] = useState("");

  const handleLogout = () => {
    logout();
    signOut(auth);
  };

  const isAnonymous = user?.isAnonymous ?? false;
  const displayName = userProfile?.display_name || user?.displayName || user?.email || 'User';
  const userInitials = displayName
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);

  const handleNewChat = () => {
    navigate('/chat');
  };

  const menuItems = [
    { icon: User, label: "Account", onClick: () => navigate('/profile') },
    { icon: History, label: "History & Sync", onClick: () => navigate('/history') },
    { icon: Brain, label: "Models", onClick: () => navigate('/models') },
    { icon: Key, label: "API Keys", onClick: () => navigate('/api-keys') },
    { icon: Paperclip, label: "Attachments", onClick: () => navigate('/attachments') },
    { icon: Settings, label: "Settings", onClick: () => navigate('/settings') },
  ];

  return (
    <aside className="flex h-screen w-64 flex-col bg-background">
      {/* App Logo and Name */}
      <div className="px-4 py-6">
        <button
          type="button"
          onClick={() => navigate('/')}
          className="flex flex-col text-left leading-tight"
        >
          <span className="text-xl font-semibold tracking-tight">T3.chat</span>
          <span className="text-[11px] uppercase tracking-[0.16em] text-muted-foreground">
            Conversations reimagined
          </span>
        </button>
      </div>

      {/* New Chat Button */}
      <div className="px-4 pb-3">
        <Button
          onClick={handleNewChat}
          className="w-full rounded-lg"
          size="default"
        >
          <Plus className="mr-2 h-4 w-4" />
          New Chat
        </Button>
      </div>

      {/* Search Chat History */}
      <div className="px-4 pb-3">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search your threads..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      {/* Chat History List */}
      <div className="flex-1 overflow-y-auto px-4">
        <div className="space-y-1">
          {/* Placeholder for chat history items */}
          {/* This will be populated with actual chat history */}
          <div className="flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted cursor-pointer">
            <MessageSquare className="h-4 w-4" />
            <span className="flex-1 truncate">Chat history will appear here</span>
          </div>
        </div>
      </div>

      {/* User Section at Bottom */}
      <div className="border-t border-border px-4 py-3">
        {user && !isAnonymous ? (
          <Popover>
            <PopoverTrigger asChild>
              <Button
                variant="ghost"
                className="w-full justify-start gap-2 px-2 hover:bg-muted"
              >
                <Avatar className="h-8 w-8 border border-border bg-muted">
                  <AvatarFallback className="text-xs font-medium">
                    {userInitials}
                  </AvatarFallback>
                </Avatar>
                <span className="flex-1 truncate text-left text-sm">
                  {displayName}
                </span>
              </Button>
            </PopoverTrigger>
            <PopoverContent align="start" className="w-64 p-2">
              <div className="space-y-1">
                {menuItems.map((item) => (
                  <Button
                    key={item.label}
                    variant="ghost"
                    className="w-full justify-start gap-2 px-3 py-2 text-sm font-normal"
                    onClick={item.onClick}
                  >
                    <item.icon className="h-4 w-4" />
                    {item.label}
                  </Button>
                ))}
                <div className="my-1 h-px bg-border" />
                <Button
                  variant="ghost"
                  className="w-full justify-start gap-2 px-3 py-2 text-sm font-normal text-destructive hover:bg-destructive/10 hover:text-destructive"
                  onClick={handleLogout}
                >
                  <LogOut className="h-4 w-4" />
                  Logout
                </Button>
              </div>
            </PopoverContent>
          </Popover>
        ) : (
          <Button
            variant="outline"
            className="w-full"
            onClick={onSignInClick}
          >
            Login
          </Button>
        )}
      </div>
    </aside>
  );
}

