import { toast as sonnerToast, type ExternalToast } from "sonner";

export type ToastType = "success" | "error" | "warning" | "info" | "loading";

export type ToastOptions = ExternalToast;

/**
 * Toast notification client for displaying notifications throughout the app.
 * Provides a simple API for showing different types of toast notifications.
 */
export const toast = {
    /**
     * Show a success toast notification
     */
    success: (message: string, options?: ToastOptions) => {
        return sonnerToast.success(message, options);
    },

    /**
     * Show an error toast notification
     */
    error: (message: string, options?: ToastOptions) => {
        return sonnerToast.error(message, options);
    },

    /**
     * Show a warning toast notification
     */
    warning: (message: string, options?: ToastOptions) => {
        return sonnerToast.warning(message, options);
    },

    /**
     * Show an info toast notification
     */
    info: (message: string, options?: ToastOptions) => {
        return sonnerToast.info(message, options);
    },

    /**
     * Show a loading toast notification
     * Returns a toast ID that can be used to update or dismiss the toast
     */
    loading: (message: string, options?: ToastOptions) => {
        return sonnerToast.loading(message, options);
    },

    /**
     * Show a promise toast notification
     * Automatically shows loading, then success or error based on promise result
     */
    promise: <T>(
        promise: Promise<T>,
        messages: {
            loading: string;
            success: string | ((data: T) => string);
            error: string | ((error: unknown) => string);
        },
        options?: Omit<ToastOptions, "description">,
    ) => {
        return sonnerToast.promise(promise, {
            loading: messages.loading,
            success: messages.success,
            error: messages.error,
            ...options,
        });
    },

    /**
     * Dismiss a toast by its ID
     */
    dismiss: (toastId?: string | number) => {
        sonnerToast.dismiss(toastId);
    },

    /**
     * Dismiss all toasts
     */
    dismissAll: () => {
        sonnerToast.dismiss();
    },
};
