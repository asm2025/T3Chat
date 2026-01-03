const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export interface AuthToken {
  access_token: string;
  refresh_token?: string;
  expires_at: number;
}

export interface User {
  id: string;
  email: string;
  name?: string;
  roles?: string[];
  email_verified?: boolean;
  avatar_url?: string;
}

const TOKEN_KEY = 'auth_token';
const REFRESH_TOKEN_KEY = 'refresh_token';
const EXPIRES_AT_KEY = 'auth_token_expires_at';

/**
 * Check if OIDC authentication is enabled
 */
export async function isOidcEnabled(): Promise<boolean> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/auth/config`);
    if (!response.ok) return false;
    const data = await response.json();
    return data.oidc_enabled === true;
  } catch {
    return false;
  }
}

/**
 * Initiate OIDC login flow by redirecting to backend login endpoint
 */
export async function initiateLogin(): Promise<void> {
  window.location.href = `${API_BASE_URL}/api/auth/login`;
}

/**
 * Local login with username/email and password
 */
export async function localLogin(username: string, password: string): Promise<User> {
  const response = await fetch(`${API_BASE_URL}/api/auth/local/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    const errorMessage = errorData.message || errorData.error || 'Login failed';
    throw new Error(errorMessage);
  }

  const data = await response.json();
  
  // Store token in localStorage
  localStorage.setItem(TOKEN_KEY, data.token);

  // Use server-provided expiration if available (robustness fix)
  if (data.expires_at) {
    // Rust server returns seconds, convert to ms
    const expiresAt = data.expires_at * 1000;
    localStorage.setItem(EXPIRES_AT_KEY, expiresAt.toString());
  } else {
    // Try to decode token to get expiration (if it's a JWT)
    try {
      const payload = JSON.parse(atob(data.token.split('.')[1]));
      if (payload.exp) {
        const expiresAt = payload.exp * 1000; // Convert to milliseconds
        localStorage.setItem(EXPIRES_AT_KEY, expiresAt.toString());
      }
    } catch {
      // Not a JWT or invalid format, set default expiration (1 hour)
      const expiresAt = Date.now() + 3600 * 1000;
      localStorage.setItem(EXPIRES_AT_KEY, expiresAt.toString());
    }
  }

  return data.user;
}

/**
 * Handle OIDC callback by storing token from URL parameter
 */
export async function handleCallback(token: string): Promise<void> {
  if (!token) {
    throw new Error('No token provided in callback');
  }
  
  // Store token in localStorage
  localStorage.setItem(TOKEN_KEY, token);
  
  // Try to decode token to get expiration (if it's a JWT)
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    if (payload.exp) {
      const expiresAt = payload.exp * 1000; // Convert to milliseconds
      localStorage.setItem(EXPIRES_AT_KEY, expiresAt.toString());
    }
  } catch {
    // Not a JWT or invalid format, set default expiration (1 hour)
    const expiresAt = Date.now() + 3600 * 1000;
    localStorage.setItem(EXPIRES_AT_KEY, expiresAt.toString());
  }
}

/**
 * Get stored auth token
 */
export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

/**
 * Get stored refresh token
 */
export function getRefreshToken(): string | null {
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

/**
 * Check if token is expired
 */
export function isTokenExpired(): boolean {
  const expiresAt = localStorage.getItem(EXPIRES_AT_KEY);
  if (!expiresAt) return true;
  
  const expirationTime = parseInt(expiresAt, 10);
  return Date.now() >= expirationTime;
}

/**
 * Refresh access token using refresh token
 */
export async function refreshToken(): Promise<string | null> {
  const refreshToken = getRefreshToken();
  if (!refreshToken) return null;
  
  try {
    const response = await fetch(`${API_BASE_URL}/api/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
    
    if (!response.ok) {
      logout();
      return null;
    }
    
    const data = await response.json();
    localStorage.setItem(TOKEN_KEY, data.access_token);
    
    if (data.refresh_token) {
      localStorage.setItem(REFRESH_TOKEN_KEY, data.refresh_token);
    }
    
    // Update expiration
    if (data.expires_in) {
      const expiresAt = Date.now() + data.expires_in * 1000;
      localStorage.setItem(EXPIRES_AT_KEY, expiresAt.toString());
    }
    
    return data.access_token;
  } catch (error) {
    console.error('Failed to refresh token:', error);
    logout();
    return null;
  }
}

/**
 * Get current user from /me endpoint
 */
export async function getCurrentUser(): Promise<User | null> {
  const token = getToken();
  if (!token) return null;
  
  // Check if token is expired and try to refresh
  if (isTokenExpired()) {
    const newToken = await refreshToken();
    if (!newToken) return null;
  }
  
  try {
    const currentToken = getToken();
    const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
      headers: { 'Authorization': `Bearer ${currentToken}` },
    });
    
    if (!response.ok) {
      if (response.status === 401) {
        logout();
      }
      return null;
    }
    
    return await response.json();
  } catch (error) {
    console.error('Failed to get current user:', error);
    return null;
  }
}

/**
 * Logout user by clearing tokens and redirecting
 */
export async function logout(): Promise<void> {
  const token = getToken();
  
  // Clear local storage
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
  localStorage.removeItem(EXPIRES_AT_KEY);
  
  // Call backend logout endpoint if we have a token
  if (token) {
    try {
      await fetch(`${API_BASE_URL}/api/auth/logout`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` },
      });
    } catch (error) {
      console.error('Failed to logout on server:', error);
    }
  }
  
  // Redirect to home
  window.location.href = '/';
}

