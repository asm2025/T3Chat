import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Extracts a meaningful error message from an unknown error value.
 * Tries multiple strategies to get the actual error message instead of generic fallbacks.
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  // Try to extract message from error objects
  if (error && typeof error === "object") {
    const errObj = error as Record<string, unknown>;
    if (typeof errObj.message === "string" && errObj.message) {
      return errObj.message;
    }
    if (typeof errObj.error === "string" && errObj.error) {
      return errObj.error;
    }
    // Try to stringify for debugging
    try {
      const stringified = String(error);
      if (stringified !== "[object Object]") {
        return stringified;
      }
    } catch {
      // Ignore stringification errors
    }
  }
  // Last resort: try JSON stringify
  try {
    return JSON.stringify(error);
  } catch {
    // If all else fails, return a generic message
    return "An error occurred";
  }
}