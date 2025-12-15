import { useEffect, useRef, useState } from "react";
import { AlertTriangle, Info, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useConfig } from "@/stores/appStore";
import type { StartupNotice } from "@/types/config";

const CONFIG_WARNING_DISMISS_KEY = "t3chat-config-warning-dismissed";

type DismissedNoticeMap = Record<string, true>;

const noticeIdentifier = (notice: StartupNotice) => `${notice.kind}:${notice.message}`.toLowerCase();

const readDismissed = (): DismissedNoticeMap => {
    if (typeof window === "undefined") {
        return {};
    }
    try {
        const raw = localStorage.getItem(CONFIG_WARNING_DISMISS_KEY);
        if (!raw) {
            return {};
        }
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
            return parsed.reduce<DismissedNoticeMap>((acc, value) => {
                if (typeof value === "string" && value) {
                    acc[value] = true;
                }
                return acc;
            }, {});
        }
    } catch (error) {
        console.error("Failed to read config warning dismiss state:", error);
    }
    return {};
};

const persistDismissed = (map: DismissedNoticeMap) => {
    if (typeof window === "undefined") {
        return;
    }
    try {
        const ids = Object.keys(map);
        localStorage.setItem(CONFIG_WARNING_DISMISS_KEY, JSON.stringify(ids));
    } catch (error) {
        console.error("Failed to persist config warning dismiss state:", error);
    }
};

const pickNotice = (notices: StartupNotice[] | undefined) => {
    if (!notices?.length) {
        return null;
    }
    return notices.find((notice) => notice.kind === "warning") ?? notices[0];
};

export function ConfigWarningBanner() {
    const { config, fetchStartupConfig } = useConfig();
    const [dismissed, setDismissed] = useState<DismissedNoticeMap>(() => readDismissed());
    const bannerRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (!config) {
            void fetchStartupConfig();
        }
    }, [config, fetchStartupConfig]);

    const notice = pickNotice(config?.notices);
    const noticeId = notice ? noticeIdentifier(notice) : null;
    const isVisible = Boolean(notice && noticeId && !dismissed[noticeId]);

    useEffect(() => {
        if (!isVisible) {
            document.documentElement.style.removeProperty("--t3chat-top-banner-offset");
            return;
        }

        const updateOffset = () => {
            if (!bannerRef.current) {
                return;
            }
            const height = bannerRef.current.getBoundingClientRect().height;
            document.documentElement.style.setProperty("--t3chat-top-banner-offset", `${height}px`);
        };

        updateOffset();

        const observer = typeof ResizeObserver !== "undefined" ? new ResizeObserver(updateOffset) : null;
        if (observer && bannerRef.current) {
            observer.observe(bannerRef.current);
        }
        window.addEventListener("resize", updateOffset);

        return () => {
            observer?.disconnect();
            window.removeEventListener("resize", updateOffset);
            document.documentElement.style.removeProperty("--t3chat-top-banner-offset");
        };
    }, [isVisible]);

    if (!isVisible || !notice || !noticeId) {
        return null;
    }

    const handleDismiss = () => {
        setDismissed((prev) => {
            const next: DismissedNoticeMap = { ...prev, [noticeId]: true };
            persistDismissed(next);
            return next;
        });
    };

    const Icon = notice.kind === "warning" ? AlertTriangle : Info;

    return (
        <div
            ref={bannerRef}
            className="fixed left-0 right-0 z-[11000] border-b border-amber-500/40 bg-amber-500/20 p-2 text-sm text-amber-900 backdrop-blur-sm dark:border-amber-400/30 dark:bg-amber-500/10 dark:text-amber-100"
            style={{ top: "env(safe-area-inset-top, 0px)" }}
            role="status"
            aria-live="polite">
            <div className="mx-auto flex w-full max-w-5xl items-center justify-center gap-3 px-6 text-center">
                <Icon className="h-4 w-4 shrink-0" aria-hidden="true" />
                <span className="flex-1">{notice.message}</span>
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleDismiss}
                    className="ml-2 h-8 w-8 border border-transparent text-amber-900 hover:bg-amber-500/30 focus-visible:ring-1 focus-visible:ring-amber-500 dark:text-amber-50 dark:hover:bg-amber-400/25"
                    aria-label="Dismiss configuration notice">
                    <X className="h-4 w-4" aria-hidden="true" />
                </Button>
            </div>
        </div>
    );
}
