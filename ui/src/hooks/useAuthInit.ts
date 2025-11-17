import { useEffect } from 'react';
import { onAuthStateChanged, signInAnonymously } from 'firebase/auth';
import { auth } from '@/lib/firebase';
import { useAppStore } from '@/stores/appStore';

const LOGOUT_RESET_DELAY_MS = 1000;

/**
 * Hook to initialize Firebase auth and sync with Zustand store
 * This replaces the AuthProvider component logic
 */
export function useAuthInit() {
  const {
    setUser,
    setUserProfile,
    setLoading,
    setProfileLoading,
    setIsLoggedOut,
    fetchUserProfile,
    isLoggedOut,
    refreshTrigger,
  } = useAppStore();

  useEffect(() => {
    // Create a flag to track if this effect is still active
    let isActive = true;

    const initializeAuth = async () => {
      try {
        setLoading(true);

        // Set up Firebase auth listener
        const unsubscribe = onAuthStateChanged(auth, async (user) => {
          // If this effect has been cleaned up, ignore the callback
          if (!isActive) {
            return;
          }

          setUser(user);
          setLoading(false);

          if (!user) {
            // Check if anonymous users are allowed (defaults to true if not set)
            const allowAnonymous = import.meta.env.VITE_ALLOW_ANONYMOUS_USERS === 'true';

            // Create anonymous user if allowed (and not explicitly logged out)
            if (!isLoggedOut && allowAnonymous) {
              try {
                await signInAnonymously(auth);
              } catch (error) {
                console.error('Failed to create anonymous user:', error);
                if (isActive) {
                  setUserProfile(null);
                  setProfileLoading(false);
                }
              }
            } else {
              // Anonymous users not allowed or user logged out
              if (isActive) {
                setUserProfile(null);
                setProfileLoading(false);
              }

              // If logout occurred, reset state after delay
              if (isLoggedOut) {
                setTimeout(() => {
                  if (isActive) {
                    setIsLoggedOut(false);
                  }
                }, LOGOUT_RESET_DELAY_MS);
              }
            }
          } else {
            // Reset logout state when user successfully logs in
            if (isActive) {
              setIsLoggedOut(false);
            }

            // Fetch user profile for authenticated users (non-anonymous with email)
            if (!user.isAnonymous && user.email && !isLoggedOut && isActive) {
              fetchUserProfile();
            } else if (isActive) {
              setUserProfile(null);
              setProfileLoading(false);
            }
          }
        });

        // Store unsubscribe function for cleanup
        return unsubscribe;
      } catch (error) {
        console.error('Auth initialization error:', error);
        if (isActive) {
          setLoading(false);
          setProfileLoading(false);
        }
      }
    };

    let unsubscribe: (() => void) | undefined;

    initializeAuth().then((unsub) => {
      if (unsub && isActive) {
        unsubscribe = unsub;
      }
    });

    return () => {
      isActive = false;
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [isLoggedOut, refreshTrigger, setUser, setUserProfile, setLoading, setProfileLoading, setIsLoggedOut, fetchUserProfile]);
}

