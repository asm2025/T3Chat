import { Moon, Sun, Monitor, Settings2 } from "lucide-react";
import { useTheme } from "next-themes";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { useSidebar } from "@/components/ui/sidebar";

export function ModeToggle() {
    const { open: sidebarOpen, isMobile } = useSidebar();
    const isCollapsed = !sidebarOpen && !isMobile;
    const { theme, setTheme } = useTheme();
    const [mounted, setMounted] = useState(false);
    const [open, setOpen] = useState(false);

    // Prevent hydration mismatch
    useEffect(() => {
        setMounted(true);
    }, []);

    if (!mounted) {
        return (
            <Button variant="ghost" size="icon" className="h-8 w-8">
                <Settings2 className="h-4 w-4" />
                <span className="sr-only">Theme settings</span>
            </Button>
        );
    }

    // Get the current theme, defaulting to "system" if not set
    // next-themes returns undefined initially, then the stored value
    // The storageKey="t3chat-theme" in ThemeProvider handles persistence automatically
    const currentTheme = theme || "system";

    const getActiveIndex = () => {
        if (currentTheme === "light") return 0;
        if (currentTheme === "system") return 1;
        return 2; // dark
    };

    const activeIndex = getActiveIndex();

    return (
        <DropdownMenu open={open} onOpenChange={setOpen}>
            <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className={cn("h-8 w-8 relative z-10", !isCollapsed && "sm:bg-gradient-noise-top")}>
                    <Settings2 className="h-4 w-4" />
                    <span className="sr-only">Theme settings</span>
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="min-w-[8rem] outline-chat-border/20! outline-1! outline-solid! dark:outline-white/5!">
                <DropdownMenuGroup>
                    <DropdownMenuItem className="flex flex-row items-center gap-2 px-2 py-1.5 text-sm cursor-default" onSelect={(e) => e.preventDefault()}>
                        <span>Theme</span>
                        <div className="border-border bg-background/80 relative flex flex-row items-center gap-0.5 rounded-full border backdrop-blur-sm ml-auto">
                            <div
                                className={cn("hover:bg-muted/40 cursor-pointer rounded-full px-2.5 py-1 transition", activeIndex === 0 && "text-foreground")}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setTheme("light");
                                    setOpen(false);
                                }}>
                                <Sun className={cn("size-4 transition-transform duration-200", activeIndex === 0 ? "rotate-0" : "rotate-90")} />
                            </div>
                            <div
                                className={cn("hover:bg-muted/40 cursor-pointer rounded-full px-2.5 py-1 transition", activeIndex === 1 && "text-foreground")}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setTheme("system");
                                    setOpen(false);
                                }}>
                                <Monitor className="size-4" />
                            </div>
                            <div
                                className={cn("hover:bg-muted/40 cursor-pointer rounded-full px-2.5 py-1 transition", activeIndex === 2 && "text-foreground")}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setTheme("dark");
                                    setOpen(false);
                                }}>
                                <Moon className={cn("size-4 transition-transform duration-200", activeIndex === 2 ? "rotate-0" : "rotate-90")} />
                            </div>
                            <div
                                className={cn(
                                    "absolute inset-y-0 -z-10 w-9 transform-gpu rounded-full border backdrop-blur-md transition-transform duration-200",
                                    // Use pink/magenta for light mode to match T3Chat design, primary for dark mode
                                    "bg-pink-200/70 dark:bg-primary/40 border-pink-300/40 dark:border-primary-foreground/20",
                                    activeIndex === 0 && "translate-x-0",
                                    activeIndex === 1 && "translate-x-[2.375rem]",
                                    activeIndex === 2 && "translate-x-[4.75rem]",
                                )}
                            />
                        </div>
                    </DropdownMenuItem>
                </DropdownMenuGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
