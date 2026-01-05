import { useState, useEffect } from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useIsMobile } from "@/hooks/useMobile";

const MOBILE_WARNING_DISMISSED_KEY = "t3chat-mobile-warning-dismissed";

export function MobileWarningBanner() {
    const [isDismissed, setIsDismissed] = useState(false);
    const isMobile = useIsMobile();

    // Load dismissed state from localStorage
    useEffect(() => {
        try {
            const dismissed = localStorage.getItem(MOBILE_WARNING_DISMISSED_KEY);
            if (dismissed === "true") {
                setIsDismissed(true);
            }
        } catch (error) {
            console.error("Failed to load mobile warning dismissed state:", error);
        }
    }, []);

    // Save dismissed state to localStorage
    const handleDismiss = () => {
        try {
            localStorage.setItem(MOBILE_WARNING_DISMISSED_KEY, "true");
            setIsDismissed(true);
        } catch (error) {
            console.error("Failed to save mobile warning dismissed state:", error);
        }
    };

    // Only show on mobile and if not dismissed
    if (!isMobile || isDismissed) {
        return null;
    }

    return (
        <div
            className="fixed top-0 right-0 left-0 z-[10000] bg-yellow-500/20 p-3 text-sm backdrop-blur-sm border-b border-yellow-500/30 md:hidden"
            style={{
                top: "calc(env(safe-area-inset-top, 0px) + var(--t3chat-top-banner-offset, 0px))",
            }}
            role="status"
            aria-live="polite">
            <div className="flex items-center gap-3 px-4">
                <span className="flex-1 text-left">We do NOT support mobile yet. Use with caution.</span>
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleDismiss}
                    className="h-8 w-8 shrink-0 hover:bg-yellow-500/30 focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 bg-yellow-500/20"
                    aria-label="Close mobile warning">
                    <X className="h-4 w-4" />
                </Button>
            </div>
        </div>
    );
}
