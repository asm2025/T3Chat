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

export async function validateFile(file: File): Promise<boolean> {
  // Basic magic number check for common types
  const arr = new Uint8Array(await file.slice(0, 4).arrayBuffer());
  let header = "";
  for (let i = 0; i < arr.length; i++) {
    header += arr[i].toString(16).padStart(2, '0');
  }

  const type = file.type;
  // JPEG
  if (type === "image/jpeg" || type === "image/jpg") {
    return header.startsWith("ffd8");
  }
  // PNG
  if (type === "image/png") {
    return header.startsWith("89504e47");
  }
  // GIF
  if (type === "image/gif") {
    return header.startsWith("47494638");
  }
  // WebP
  if (type === "image/webp") {
    // WebP is RIFF...WEBP
    // first 4 bytes "RIFF" (52 49 46 46)
    return header.startsWith("52494646");
  }

  // Allow others for now if unknown
  return true;
}
