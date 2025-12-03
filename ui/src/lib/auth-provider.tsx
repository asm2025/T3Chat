import { ReactNode, useEffect, useState } from "react";
import { getCurrentUser, logout as authLogout, getToken, initiateLogin } from "./auth";
import { AuthContext, type User } from "./auth-context-base";

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const loadUser = async () => {
            const token = getToken();
            if (!token) {
                setLoading(false);
                return;
            }

            try {
                const currentUser = await getCurrentUser();
                setUser(currentUser);
            } catch (error) {
                console.error("Failed to load user:", error);
                setUser(null);
            } finally {
                setLoading(false);
            }
        };

        void loadUser();
    }, []);

    function login() {
        initiateLogin();
    }

    async function logout() {
        await authLogout();
        setUser(null);
    }

    const isAuthenticated = !!user;
    const isAdmin = user?.roles?.includes("admin") ?? false;

    return (
        <AuthContext.Provider
            value={{
                user,
                loading,
                isAuthenticated,
                isAdmin,
                login,
                logout,
            }}>
            {children}
        </AuthContext.Provider>
    );
}

