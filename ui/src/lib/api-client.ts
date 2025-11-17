import { getAuth } from "firebase/auth";
import { app } from "./firebase";

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

export interface ApiResponse<T> {
    ok: boolean;
    status: number;
    body: T | null;
}

export interface ApiClientError extends Error {
    status?: number;
    code?: string;
    user_id?: string;
    isNetworkError?: boolean;
}

interface RequestOptions {
    method?: string;
    headers?: Record<string, string>;
    body?: unknown;
    query?: Record<string, string | number | boolean>;
}

/**
 * Single API client class using fetch with auth token injection and error handling.
 * Inspired by Miguel Grinberg's React Mega-Tutorial API client pattern.
 */
export class ApiClient {
    private baseUrl: string;

    constructor() {
        this.baseUrl = `${API_BASE_URL}/api`;
    }

    private async getAuthToken(): Promise<string | null> {
        const auth = getAuth(app);
        const user = auth.currentUser;
        if (!user) return null;
        return user.getIdToken();
    }

    private async request<T>(options: RequestOptions & { url: string }): Promise<ApiResponse<T>> {
        // Build query string
        let query = "";
        if (options.query) {
            const params = new URLSearchParams();
            for (const [key, value] of Object.entries(options.query)) {
                params.append(key, String(value));
            }
            query = params.toString();
            if (query) {
                query = "?" + query;
            }
        }

        // Get auth token
        const token = await this.getAuthToken();

        // Prepare headers
        const headers: Record<string, string> = {
            "Content-Type": "application/json",
            ...options.headers,
        };

        if (token) {
            headers["Authorization"] = `Bearer ${token}`;
        }

        // Prepare body
        let body: string | null = null;
        if (options.body !== undefined) {
            body = JSON.stringify(options.body);
        }

        let response: Response;
        try {
            response = await fetch(this.baseUrl + options.url + query, {
                method: options.method || "GET",
                headers,
                body,
                credentials: "include",
            });
        } catch (error) {
            // Network error (server unresponsive, CORS, etc.)
            const apiError = new Error("The server is unresponsive") as ApiClientError;
            apiError.name = "ApiClientError";
            apiError.status = 500;
            apiError.isNetworkError = true;
            throw apiError;
        }

        // Parse response body
        let responseBody: T | null = null;
        const contentType = response.headers.get("content-type") || "";

        if (response.status !== 204 && response.status !== 304) {
            if (contentType.includes("application/json")) {
                try {
                    responseBody = await response.json();
                } catch (parseError) {
                    // If JSON parsing fails, throw error
                    const apiError = new Error("Failed to parse JSON response") as ApiClientError;
                    apiError.name = "ApiClientError";
                    apiError.status = response.status;
                    throw apiError;
                }
            } else {
                // Non-JSON response - read as text for error messages
                const text = await response.text().catch(() => "");
                if (!response.ok) {
                    const apiError = new Error(text || response.statusText || `HTTP ${response.status}`) as ApiClientError;
                    apiError.name = "ApiClientError";
                    apiError.status = response.status;
                    if (text.trim().startsWith("<!DOCTYPE") || text.trim().startsWith("<html")) {
                        apiError.message = `Server returned HTML instead of JSON. This usually means:\n1. The API server is not running at ${API_BASE_URL}\n2. The endpoint does not exist (404 page)\n3. There's a server error (500 page)\nResponse status: ${response.status} ${response.statusText}`;
                    }
                    throw apiError;
                }
                // For non-JSON successful responses, return text as body
                responseBody = text as unknown as T;
            }
        }

        // Handle error responses
        if (!response.ok) {
            const apiError = new Error(response.statusText || `HTTP ${response.status}`) as ApiClientError;
            apiError.name = "ApiClientError";
            apiError.status = response.status;

            // Try to extract error details from JSON response
            if (responseBody && typeof responseBody === "object") {
                const errorData = responseBody as Record<string, unknown>;
                if (typeof errorData.message === "string") {
                    apiError.message = errorData.message;
                } else if (typeof errorData.error === "string") {
                    apiError.message = errorData.error;
                }
                if (typeof errorData.code === "string") {
                    apiError.code = errorData.code;
                }
                if (typeof errorData.user_id === "string") {
                    apiError.user_id = errorData.user_id;
                }
            }

            // Special handling for 401
            if (response.status === 401) {
                apiError.message = apiError.message || "Authentication required";
            }

            throw apiError;
        }

        return {
            ok: response.ok,
            status: response.status,
            body: responseBody,
        };
    }

    /**
     * GET request - fetch a single resource
     */
    async get<T>(url: string, query?: Record<string, string | number | boolean>): Promise<T> {
        const response = await this.request<T>({ url, method: "GET", query });
        return response.body as T;
    }

    /**
     * GET request - fetch a list of resources (with pagination support)
     */
    async list<T>(url: string, query?: Record<string, string | number | boolean>): Promise<T> {
        const response = await this.request<T>({ url, method: "GET", query });
        return response.body as T;
    }

    /**
     * POST request - create a new resource
     */
    async post<T>(url: string, body?: unknown, options?: Omit<RequestOptions, "method" | "body">): Promise<T> {
        const response = await this.request<T>({ url, method: "POST", body, ...options });
        return response.body as T;
    }

    /**
     * PUT request - update an existing resource
     */
    async update<T>(url: string, body?: unknown, options?: Omit<RequestOptions, "method" | "body">): Promise<T> {
        const response = await this.request<T>({ url, method: "PUT", body, ...options });
        return response.body as T;
    }

    /**
     * DELETE request - delete a resource
     */
    async delete(url: string, options?: Omit<RequestOptions, "method">): Promise<void> {
        await this.request({ url, method: "DELETE", ...options });
    }

    /**
     * Streaming request - for endpoints that return Server-Sent Events or streaming data
     * Returns the raw Response object for manual handling
     */
    async stream(url: string, body?: unknown): Promise<Response> {
        const token = await this.getAuthToken();
        const headers: Record<string, string> = {
            "Content-Type": "application/json",
        };

        if (token) {
            headers["Authorization"] = `Bearer ${token}`;
        }

        const response = await fetch(this.baseUrl + url, {
            method: "POST",
            headers,
            body: body ? JSON.stringify(body) : undefined,
            credentials: "include",
        });

        if (!response.ok) {
            const contentType = response.headers.get("content-type") || "";
            let message = response.statusText || `HTTP ${response.status}`;
            if (contentType.includes("application/json")) {
                try {
                    const data = await response.json();
                    if (typeof data === "object" && data && ("message" in data || "error" in data)) {
                        message = String((data as any).message ?? (data as any).error);
                    }
                } catch {}
            }
            const apiError = new Error(message) as ApiClientError;
            apiError.name = "ApiClientError";
            apiError.status = response.status;
            throw apiError;
        }

        return response;
    }
}

// Export a singleton instance
export const api = new ApiClient();

