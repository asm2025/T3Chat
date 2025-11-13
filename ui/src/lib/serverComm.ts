import { getAuth } from "firebase/auth";
import { app } from "./firebase";

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3010";

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

export async function fetchWithAuth(endpoint: string, options: RequestInit = {}): Promise<Response> {
    const token = await getAuthToken();
    const headers = new Headers(options.headers);

    if (token) {
        headers.set("Authorization", `Bearer ${token}`);
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        ...options,
        headers,
    });

    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: response.statusText }));

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
    return response.json();
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
    return response.json();
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
