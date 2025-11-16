import { ReactNode } from "react";
import { ModeToggle } from "@/components/mode-toggle";
import { useSidebar } from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";

interface MasterLayoutProps {
    children: ReactNode;
    footer?: ReactNode;
}

export function MasterLayout({ children, footer }: MasterLayoutProps) {
    const { open: sidebarOpen, isMobile } = useSidebar();
    // Only apply collapsed styles on desktop when sidebar is closed
    const isCollapsed = !sidebarOpen && !isMobile;

    return (
        <div className="flex h-screen flex-col bg-background relative">
            <div
                className={cn(
                    "bg-chat-background ease-snappy absolute top-0 bottom-0 left-0 right-0 overflow-hidden bg-fixed pb-[140px] transition-all select-none print:hidden",
                    isCollapsed ? "!translate-y-0 !rounded-none border-none max-sm:border-none" : "border-chat-border border-t border-l max-sm:border-none sm:translate-y-3.5 sm:rounded-tl-xl",
                )}>
                <div className={cn("bg-noise ease-snappy absolute inset-0 -top-3.5 bg-fixed [background-position:right_bottom] transition-transform", isCollapsed && "translate-y-3.5")}></div>
            </div>

            {/* Top decorative border */}
            <div
                className={cn(
                    "bg-gradient-noise-top/80 ease-snappy blur-fallback:bg-gradient-noise-top absolute inset-x-3 top-0 z-10 box-content overflow-hidden backdrop-blur-md transition-[transform,border] max-sm:hidden sm:h-3.5 print:hidden",
                    isCollapsed ? "-translate-y-[15px] border-transparent" : "border-chat-border border-b",
                )}
                aria-hidden="true">
                <div className="from-gradient-noise-top blur-fallback:hidden absolute top-0 left-0 h-full w-8 bg-gradient-to-r to-transparent"></div>
                <div className="from-gradient-noise-top blur-fallback:hidden absolute top-0 right-24 h-full w-8 bg-gradient-to-l to-transparent"></div>
                <div className="bg-gradient-noise-top blur-fallback:hidden absolute top-0 right-0 h-full w-24"></div>
            </div>

            {/* Main content area */}
            <div className="absolute top-0 bottom-0 w-full print:static print:h-auto print:overflow-visible">
                {/* Fixed top right decorative element (above chat-scroll-container) */}
                {!isCollapsed && (
                    <div className="fixed top-0 right-0 max-sm:hidden">
                        <div className="ease-snappy group pointer-events-none absolute top-3.5 z-10 -mb-8 h-32 w-full origin-top transition-all" style={{ boxShadow: "10px -10px 8px 2px var(--gradient-noise-top)" }}>
                            <svg
                                className="absolute h-9 origin-top-left skew-x-30 overflow-visible ease-snappy -right-[4rem] transform-gpu transition-transform duration-300 translate-x-1"
                                version="1.1"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 128 32">
                                <line stroke="var(--gradient-noise-top)" strokeWidth="2px" shapeRendering="optimizeQuality" vectorEffect="non-scaling-stroke" strokeLinecap="round" strokeMiterlimit="10" x1="1" y1="0" x2="128" y2="0"></line>
                                <path
                                    stroke="var(--chat-border)"
                                    className="translate-y-[0.5px]"
                                    fill="var(--gradient-noise-top)"
                                    shapeRendering="optimizeQuality"
                                    strokeWidth="1px"
                                    strokeLinecap="round"
                                    strokeMiterlimit="10"
                                    vectorEffect="non-scaling-stroke"
                                    d="M0,0c5.9,0,10.7,4.8,10.7,10.7v10.7c0,5.9,4.8,10.7,10.7,10.7H128V0"></path>
                            </svg>
                        </div>
                    </div>
                )}

                {footer && (
                    <div className="pointer-events-none absolute bottom-0 z-10 w-full px-2">
                        <div className="relative mx-auto flex w-full max-w-3xl flex-col text-center">
                            <div className="pointer-events-auto">{footer}</div>
                        </div>
                    </div>
                )}

                <div
                    id="chat-scroll-container"
                    className="absolute inset-0 overflow-y-auto pt-8 sm:pt-3.5 print:visible print:static print:inset-auto print:block print:h-auto print:scroll-pb-0! print:overflow-visible print:pt-2 print:pb-0!"
                    style={{ paddingBottom: "144px", scrollbarGutter: "stable both-edges", scrollPaddingBottom: "112px" }}>
                    {/* Decorative curve inside scroll container */}
                    {!isCollapsed && (
                        <div className="pointer-events-none fixed top-0 right-0 z-20 h-20 w-40 print:invisible max-sm:hidden" style={{ clipPath: "inset(0px 12px 0px 0px)" }}>
                            <div className="ease-snappy group pointer-events-none absolute top-3.5 z-10 -mb-8 h-32 w-full origin-top transition-all" style={{ boxShadow: "10px -10px 8px 2px var(--gradient-noise-top)" }}>
                                <svg
                                    className="absolute h-9 origin-top-left skew-x-30 overflow-visible ease-snappy -right-[4rem] transform-gpu transition-transform duration-300 translate-x-1"
                                    version="1.1"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 0 128 32">
                                    <line stroke="var(--gradient-noise-top)" strokeWidth="2px" shapeRendering="optimizeQuality" vectorEffect="non-scaling-stroke" strokeLinecap="round" strokeMiterlimit="10" x1="1" y1="0" x2="128" y2="0"></line>
                                    <path
                                        stroke="var(--chat-border)"
                                        className="translate-y-[0.5px]"
                                        fill="var(--gradient-noise-top)"
                                        shapeRendering="optimizeQuality"
                                        strokeWidth="1px"
                                        strokeLinecap="round"
                                        strokeMiterlimit="10"
                                        vectorEffect="non-scaling-stroke"
                                        d="M0,0c5.9,0,10.7,4.8,10.7,10.7v10.7c0,5.9,4.8,10.7,10.7,10.7H128V0"></path>
                                </svg>
                            </div>
                        </div>
                    )}

                    {/* Mode Toggle inside scroll container */}
                    <div className="top-safe-offset-2 fixed z-20 print:invisible" style={{ right: "var(--firefox-scrollbar, 0.4rem)" }}>
                        <div className="pointer-events-none absolute inset-0 left-auto -z-10 w-0 rounded-md bg-transparent backdrop-blur-sm transition-[background-color,width] delay-0 duration-250 max-sm:bg-sidebar/50 max-sm:delay-125 max-sm:duration-125 max-sm:w-10"></div>
                        <div className={cn("text-muted-foreground flex flex-row items-center gap-0.5 rounded-md p-1 transition-all", isCollapsed ? "rounded-bl-xl" : "sm:bg-gradient-noise-top")}>
                            <ModeToggle />
                        </div>
                    </div>

                    <div role="log" aria-live="polite" className="pt-safe-offset-10 mx-auto flex w-full max-w-3xl flex-col space-y-12 px-4 pb-10 print:space-y-0 print:pt-0">
                        {children}
                    </div>
                </div>
            </div>
        </div>
    );
}
