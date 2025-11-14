import { getAuth } from "firebase/auth";
import { app } from "./firebase";

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

// Functional error type instead of class
interface APIError extends Error {
    status: number;
    code?: string;
    user_id?: string;
}

function createAPIError(status: number, message: string, code?: string, user_id?: string): APIError {
    const error = new Error(message) as APIError;
    error.name = "APIError";
    error.status = status;
    error.code = code;
    error.user_id = user_id;
    return error;
}

async function getAuthToken(): Promise<string | null> {
    const auth = getAuth(app);
    const user = auth.currentUser;
    if (!user) {
        return null;
    }
    return user.getIdToken();
}

// Helper function to safely parse JSON from a response
export async function safeJsonParse(response: Response): Promise<any> {
    const contentType = response.headers.get("content-type");

    // Clone the response so we can read it multiple times if needed (for error messages)
    const clonedResponse = response.clone();

    // Check if response is JSON
    if (!contentType || !contentType.includes("application/json")) {
        // If not JSON, read as text to provide better error message
        const text = await clonedResponse.text();

        // If it looks like HTML (common for error pages), provide helpful message
        if (text.trim().startsWith("<!DOCTYPE") || text.trim().startsWith("<html")) {
            throw new Error(
                `Server returned HTML instead of JSON. This usually means:\n` +
                    `1. The API server is not running at ${API_BASE_URL}\n` +
                    `2. The endpoint does not exist (404 page)\n` +
                    `3. There's a server error (500 page)\n` +
                    `Response status: ${response.status} ${response.statusText}`,
            );
        }

        // Otherwise, throw with the actual text
        throw new Error(`Expected JSON but received ${contentType || "unknown content type"}. Response: ${text.substring(0, 200)}`);
    }

    // Content-Type suggests JSON, try to parse it
    try {
        return await response.json();
    } catch (error) {
        // If JSON parsing fails, read the text from the clone to see what we actually got
        // (We can't read from response again since it's already consumed)
        const text = await clonedResponse.text();
        throw new Error(`Failed to parse JSON response: ${error instanceof Error ? error.message : String(error)}\n` + `Response content (first 200 chars): ${text.substring(0, 200)}`);
    }
}

export async function fetchWithAuth(endpoint: string, options: RequestInit = {}): Promise<Response> {
    const token = await getAuthToken();
    const headers = new Headers(options.headers);

    if (token) {
        headers.set("Authorization", `Bearer ${token}`);
    }

    let response: Response;
    try {
        response = await fetch(`${API_BASE_URL}${endpoint}`, {
            ...options,
            headers,
        });
    } catch (error) {
        // Network error (server not reachable, CORS, etc.)
        if (error instanceof TypeError && error.message.includes("fetch")) {
            throw new Error(`Failed to connect to API server at ${API_BASE_URL}${endpoint}.\n` + `Please ensure the API server is running and accessible.\n` + `Original error: ${error.message}`);
        }
        throw error;
    }

    if (!response.ok) {
        const errorData = await safeJsonParse(response).catch((parseError) => {
            // If parsing fails, return a basic error structure
            return {
                error: response.statusText || "Unknown error",
                message: parseError instanceof Error ? parseError.message : String(parseError),
            };
        });

        throw createAPIError(response.status, errorData.error || errorData.message || `API request failed: ${response.statusText}`, errorData.code, errorData.user_id);
    }

    return response;
}

// API endpoints
export async function getCurrentUser(): Promise<{
    id: string;
    email: string | null;
    display_name: string | null;
    image_url: string | null;
    created_at: string;
    updated_at: string;
}> {
    const response = await fetchWithAuth("/api/v1/me");
    return safeJsonParse(response);
}

export async function updateUser(data: { display_name?: string | null; image_url?: string | null }): Promise<{
    id: string;
    email: string | null;
    display_name: string | null;
    image_url: string | null;
    created_at: string;
    updated_at: string;
}> {
    const response = await fetchWithAuth("/api/v1/me", {
        method: "PUT",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    });
    return safeJsonParse(response);
}

// Example of how to add more API endpoints:
// export async function createChat(data: CreateChatData) {
//   const response = await fetchWithAuth('/api/v1/protected/chats', {
//     method: 'POST',
//     headers: {
//       'Content-Type': 'application/json',
//     },
//     body: JSON.stringify(data),
//   });
//   return response.json();
// }

export const api = {
    getCurrentUser,
    updateUser,
    // Add other API endpoints here
};

// Export API functions
export * from "./api/models";
export * from "./api/chats";
export * from "./api/messages";
export * from "./api/chat";
export * from "./api/userApiKeys";
