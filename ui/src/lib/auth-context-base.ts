import { createContext } from "react";

export interface User {
    id: string;
    email: string;
    name?: string;
    roles?: string[];
    email_verified?: boolean;
    avatar_url?: string;
}

export interface AuthContextValue {
    user: User | null;
    loading: boolean;
    isAuthenticated: boolean;
    isAdmin: boolean;
    login: () => void;
    logout: () => void;
}

export const AuthContext = createContext<AuthContextValue | undefined>(undefined);

