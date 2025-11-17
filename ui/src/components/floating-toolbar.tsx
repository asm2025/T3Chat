import { Button } from "@/components/ui/button";
import { useSidebar } from "@/components/ui/sidebar";
import { PanelLeft, Search, Plus } from "lucide-react";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/dialog";
import { useState } from "react";
import { cn } from "@/lib/utils";
import { useNavigate } from "react-router-dom";
import { Clock } from "lucide-react";
import { useChat } from "@/stores/appStore";

export function FloatingToolbar() {
    const { open, toggleSidebar, isMobile } = useSidebar();
    const [searchOpen, setSearchOpen] = useState(false);
    const [searchQuery, setSearchQuery] = useState("");
    const navigate = useNavigate();
    const { clearChat } = useChat();

    // Only show when sidebar is collapsed (and not on mobile where it's always offcanvas)
    if (open || isMobile) {
        return null;
    }

    const handleNewChat = () => {
        clearChat();
        navigate("/");
    };

    return (
        <div className="top-safe-offset-2 pointer-events-auto fixed left-2 z-50 flex flex-row gap-0.5 p-1">
            {/* Backdrop blur background */}
            <div
                className={cn(
                    "pointer-events-none absolute inset-0 right-auto -z-10 rounded-md backdrop-blur-sm transition-[background-color,width] delay-0 duration-250",
                    "bg-sidebar/50 w-[6.75rem] delay-125 duration-125",
                    "max-sm:bg-sidebar/50 max-sm:w-[6.75rem] max-sm:delay-125 max-sm:duration-125"
                )}
            />

            {/* Sidebar Toggle Button */}
            <Button
                data-sidebar="trigger"
                variant="ghost"
                size="icon"
                className="size-9 text-muted-foreground z-10 h-8 w-8 hover:bg-muted/40 hover:text-foreground"
                onClick={toggleSidebar}>
                <PanelLeft className="h-4 w-4" />
                <span className="sr-only">Toggle Sidebar</span>
            </Button>

            {/* Search Button */}
            <Dialog open={searchOpen} onOpenChange={setSearchOpen}>
                <DialogTrigger asChild>
                    <Button
                        variant="ghost"
                        size="icon"
                        className="text-muted-foreground size-8 translate-x-0 opacity-100 transition-[transform,opacity] delay-150 duration-250 hover:bg-muted/40 hover:text-foreground">
                        <Search className="h-4 w-4" />
                        <span className="sr-only">Search</span>
                    </Button>
                </DialogTrigger>
                <DialogContent
                    className="z-50 grid gap-4 rounded-lg border-none bg-transparent p-0 shadow-none outline-hidden duration-0 [&>button]:hidden !left-1/2 !top-1/3 !-translate-x-1/2 !-translate-y-1/2"
                    style={{ width: "100%", maxWidth: "36rem" }}>
                    <div className="bg-popover text-secondary-foreground outline-chat-border/20 pointer-events-auto flex h-fit w-full flex-col gap-1 rounded-xl p-3.5 pt-2.5 shadow-2xl outline-1 backdrop-blur-md outline-solid max-sm:mx-4 dark:outline-white/5">
                        <div className="relative">
                            <div className="bg-popover w-full rounded-t-lg">
                                <div className="mr-px flex items-center justify-start gap-1 pb-2">
                                    <div className="text-muted-foreground/75 flex items-center shrink-0">
                                        <Search className="ml-px size-4" />
                                        <span className="ml-[3px] size-4 skew-x-30 opacity-20 select-none flex items-center">/</span>
                                        <Plus className="mr-3 size-4" />
                                    </div>
                                    <textarea
                                        id="search-input"
                                        name="search"
                                        className="placeholder:text-muted-foreground/50 flex-1 resize-none bg-transparent text-sm placeholder:select-none focus:ring-0 focus:outline-hidden focus-visible:ring-0 focus-visible:outline-hidden leading-none"
                                        role="searchbox"
                                        aria-label="Search threads and messages"
                                        placeholder="Search or press Enter to start new chat..."
                                        value={searchQuery}
                                        onChange={(e) => setSearchQuery(e.target.value)}
                                        onKeyDown={(e) => {
                                            if (e.key === "Enter" && !e.shiftKey) {
                                                e.preventDefault();
                                                if (searchQuery.trim()) {
                                                    handleNewChat();
                                                    setSearchOpen(false);
                                                }
                                            }
                                        }}
                                        style={{ height: "20px", lineHeight: "20px", paddingTop: 0, paddingBottom: 0 }}
                                    />
                                </div>
                                <div className="border-chat-border border-b px-3" />
                            </div>
                        </div>
                        <div className="mt-2.5 max-h-[50vh] space-y-2 overflow-y-auto">
                            <div className="flex flex-col gap-1">
                                <div className="text-color-heading flex w-full items-center justify-start gap-1.5 pl-[3px] text-sm">
                                    <Clock className="size-3" />
                                    Recent Chats
                                </div>
                                <ul className="text-muted-foreground flex flex-col gap-0 text-sm">
                                    {/* Placeholder for recent chats */}
                                </ul>
                            </div>
                        </div>
                    </div>
                </DialogContent>
            </Dialog>

            {/* New Chat Button (disabled) */}
            <a
                className={cn(
                    "focus-visible:ring-ring inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50",
                    "[&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 hover:bg-muted/40 hover:text-foreground disabled:hover:text-foreground/50 disabled:hover:bg-transparent",
                    "text-muted-foreground size-8 translate-x-0 transition-[transform,opacity] delay-150 duration-150",
                    "pointer-events-none opacity-25"
                )}
                aria-disabled="true"
                tabIndex={-1}
                onClick={(e) => e.preventDefault()}
                href="/"
                data-discover="true">
                <Plus className="h-4 w-4" />
                <span className="sr-only">New Thread</span>
            </a>
        </div>
    );
}

