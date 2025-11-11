import { Button } from "@/components/ui/button";
import { auth } from "@/lib/firebase";
import { signOut } from "firebase/auth";
import { Menu, User, LogOut } from "lucide-react";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { useAuth } from "@/lib/auth-context";
import { ModeToggle } from "@/components/mode-toggle";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { useNavigate } from "react-router-dom";

interface NavbarProps {
  onSignInClick?: () => void;
}

export function Navbar({ onSignInClick }: NavbarProps = {}) {
  const { user, logout, userProfile } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();  // Set isLoggedOut flag first
    signOut(auth);  // Then sign out from Firebase
  };

  const handleProfileClick = () => {
    navigate('/profile');
  };

  const isAnonymous = user?.isAnonymous ?? false;
  const displayName = userProfile?.display_name || user?.displayName || user?.email || 'User';
  const userInitials = displayName
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);

  return (
    <header className="sticky top-0 z-50 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/90">
      <div className="flex h-16 items-center gap-3 px-3 sm:px-6">
        <div className="flex items-center gap-3">
          <SidebarTrigger className="flex size-10 items-center justify-center rounded-full border border-border text-muted-foreground hover:text-foreground sm:size-9">
            <Menu className="h-5 w-5" />
          </SidebarTrigger>
          <button
            type="button"
            onClick={() => navigate('/')}
            className="flex flex-col text-left leading-tight"
          >
            <span className="text-base font-semibold tracking-tight sm:text-lg">T3.chat</span>
            <span className="text-[11px] uppercase tracking-[0.16em] text-muted-foreground">Conversations reimagined</span>
          </button>
        </div>
        <div className="ml-auto flex items-center gap-2 sm:gap-3">
          <ModeToggle />
          {user && !isAnonymous ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" className="flex items-center gap-2 rounded-full px-2.5 py-1.5 text-sm hover:bg-muted">
                  <span className="hidden text-sm sm:inline">{displayName}</span>
                  <Avatar className="h-8 w-8 border border-border bg-muted shadow-sm">
                    <AvatarFallback className="text-xs font-medium tracking-wide">
                      {userInitials}
                    </AvatarFallback>
                  </Avatar>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-52">
                <DropdownMenuItem onClick={handleProfileClick} className="cursor-pointer gap-2">
                  <User className="h-4 w-4" />
                  Profile
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  onClick={handleLogout}
                  variant="destructive"
                  className="cursor-pointer gap-2"
                >
                  <LogOut className="h-4 w-4" />
                  Sign Out
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <Button
              variant="outline"
              size="sm"
              onClick={onSignInClick}
              className="rounded-full border-border px-4 text-sm font-medium text-foreground shadow-sm hover:bg-muted"
            >
              Login
            </Button>
          )}
        </div>
      </div>
    </header>
  );
} 