import { writable, derived } from 'svelte/store';
import type { User } from '$lib/api/auth';
import { authApi } from '$lib/api/auth';
import { API_URL } from '$lib/api/client';
import { browser } from '$app/environment';

interface AuthState {
	token: string | null;
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	authEnabled: boolean; // Track if auth is enabled on server
}

const initialState: AuthState = {
	token: null,
	user: null,
	isAuthenticated: false,
	isLoading: true,
	authEnabled: true, // Default to true until we check
};

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

		// Check server config and initialize auth state
	async function initialize() {
		try {
			// Try to check if server has auth enabled
			// If the endpoint doesn't exist or returns error, assume auth is enabled
			let authEnabled = true;
			
			try {
				const config = await authApi.getAppConfig();
				authEnabled = config.auth_enabled;
			} catch (configError) {
                console.warn('App Config failed:', configError);
				// If app-config endpoint doesn't exist, try to detect NO_AUTH mode
				// by attempting to access a protected endpoint without auth
				try {
					// B32: this probed a *relative* /api/v1/catalogs, which only
					// resolves under the dev proxy. Against a real deployment it
					// hit the SvelteKit server and 404'd, so the probe always
					// concluded "auth enabled" regardless of the server's actual
					// mode. Going through API_URL probes the API.
					const response = await fetch(`${API_URL}/api/v1/catalogs`, {
						method: 'GET',
						headers: { 'Content-Type': 'application/json' }
					});

					// A 200 without credentials means the server is in NO_AUTH mode.
					authEnabled = !response.ok;
				} catch {
					// If the request fails outright, assume auth is enabled: that
					// is the safe direction to be wrong in.
					authEnabled = true;
				}
			}
			
			if (!authEnabled) {
				// NO_AUTH mode - set flag
                let restored = false;
                
                // RESTORE SESSION IF EXISTS IN LOCALSTORAGE
                if (browser) {
                    const token = localStorage.getItem('auth_token');
                    const userStr = localStorage.getItem('auth_user');
                    
                    // Relaxed check: Allow either specific 'no-auth-mode' token OR a real JWT if one was set by a manual login
                    if (token && userStr) {
                        try {
                             const user = JSON.parse(userStr);
                             update(state => ({
                                ...state,
                                authEnabled: false,
                                isAuthenticated: true,
                                user: user,
                                token: token,
                                isLoading: false
                             }));
                             restored = true;
                         } catch (e) {
                             console.error("Failed to restore no-auth session", e);
                         }
                    }
                }
                
                if (!restored) {
                    update(state => ({
                        ...state,
                        authEnabled: false,
                        isLoading: false,
                    }));
                }
				return;
			}
            
            // ... (rest of initialize) ...
            
			// Auth is enabled - check for existing token
			update(state => ({ ...state, authEnabled: true }));
			
			if (browser) {
				let token = localStorage.getItem('auth_token');
				const userStr = localStorage.getItem('auth_user');

				// If we have a no-auth-mode token but auth is enabled, clear it
				if (token === 'no-auth-mode') {
					localStorage.removeItem('auth_token');
					localStorage.removeItem('auth_user');
					token = null;
					update(state => ({ 
						...state, 
						token: null, 
						user: null, 
						isAuthenticated: false, 
						isLoading: false 
					}));
					return;
				}

				if (token && userStr) {
					const user = JSON.parse(userStr);
					update(state => ({
						...state,
						token,
						user,
						isAuthenticated: true,
						isLoading: false,
					}));
				} else {
					update(state => ({ ...state, isLoading: false }));
				}
			} else {
				update(state => ({ ...state, isLoading: false }));
			}
		} catch (error) {
			console.error('Failed to initialize auth:', error);
			// On error, assume auth is enabled and not authenticated
			update(state => ({ ...state, isLoading: false, authEnabled: true }));
		}
	}

	return {
		subscribe,
		initialize,
        
        // Manual trigger for No Auth Login
        loginNoAuth() {
            const mockUser: User = {
                id: 'no-auth-user',
                username: 'no-auth',
                role: 'root',
                tenant_id: '00000000-0000-0000-0000-000000000000'
            };
            
            update(state => ({
                ...state,
                authEnabled: false,
                isAuthenticated: true,
                user: mockUser,
                token: 'no-auth-mode',
                isLoading: false,
            }));
            
            if (browser) {
                localStorage.setItem('auth_token', 'no-auth-mode');
                localStorage.setItem('auth_user', JSON.stringify(mockUser));
            }
            
            return { success: true };
        },
        
		async handleOAuthLogin(token: string) {
			try {
				if (browser) {
					localStorage.setItem('auth_token', token);
				}
				
				update(state => ({ ...state, token }));

				// Fetch user details
				const user = await authApi.getCurrentUser();

				update(state => ({
					...state,
					user,
					isAuthenticated: true,
					isLoading: false
				}));

				if (browser) {
					localStorage.setItem('auth_user', JSON.stringify(user));
				}

				return { success: true };
			} catch (error: any) {
				console.error('OAuth login failed:', error);
				// Clean up invalid token
				if (browser) {
					localStorage.removeItem('auth_token');
				}
				update(state => ({ ...state, token: null, isAuthenticated: false, isLoading: false }));
				
				return { success: false, error: error.message || 'OAuth login failed' };
			}
		},
		async login(username: string, password: string, tenantId?: string | null) {
			try {
				const response = await authApi.login(username, password, tenantId);
				const user: User = response.user;

				update(state => ({
					...state,
					token: response.token,
					user,
					isAuthenticated: true,
				}));

				if (browser) {
					localStorage.setItem('auth_token', response.token);
					localStorage.setItem('auth_user', JSON.stringify(user));
				}

				return { success: true };
			} catch (error: any) {
				console.error('Login failed:', error);
				return { success: false, error: error.message || 'Login failed' };
			}
		},
		/**
		 * End the session, server-side as well as locally.
		 *
		 * B34: logout only cleared localStorage. `authApi.logout()` and the token
		 * revocation endpoint were never called, so the JWT stayed valid on the
		 * server for its full 24-hour lifetime - "logging out" on a shared
		 * machine left a working credential behind. The revocation is fired
		 * first but not awaited for the local clear: the local session must end
		 * even if the network call fails.
		 */
		logout() {
			const token = browser ? localStorage.getItem('auth_token') : null;

			if (browser) {
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}

			update(state => ({
				...state,
				token: null,
				user: null,
				isAuthenticated: false,
			}));

			// Real sessions only: the NO_AUTH sentinel has nothing to revoke.
			//
			// Local state is cleared first and this is best-effort: a logout
			// that throws because the revocation call misbehaved would leave the
			// caller's redirect unexecuted, stranding the user on a page they
			// are no longer authenticated for. The `void`/`.catch` pair only
			// covered a rejected promise, not a synchronous throw or a
			// non-promise return.
			if (browser && token && token !== 'no-auth-mode') {
				try {
					void Promise.resolve(authApi.revokeCurrentToken({ reason: 'logout' })).catch(
						(e) => console.warn('token revocation on logout failed', e)
					);
				} catch (e) {
					console.warn('token revocation on logout failed', e);
				}
			}
		},

		/**
		 * End the session because the server rejected our credentials.
		 *
		 * B34: nothing in `src/` handled a 401, so an expired JWT left the user
		 * in a permanently broken "authenticated" state. Registered with the API
		 * client by the root layout.
		 */
		sessionExpired() {
			if (browser) {
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}

			update(state => ({
				...state,
				token: null,
				user: null,
				isAuthenticated: false,
			}));
		},
		updateSession(token: string, user: User) {
			update(state => ({
				...state,
				token,
				user,
				isAuthenticated: true,
			}));

			if (browser) {
				localStorage.setItem('auth_token', token);
				localStorage.setItem('auth_user', JSON.stringify(user));
			}
		},
	};
}

export const authStore = createAuthStore();

// Derived stores for convenience
export const isAuthenticated = derived(authStore, $auth => $auth.isAuthenticated);
export const currentUser = derived(authStore, $auth => $auth.user);
export const user = currentUser; // Alias for compatibility with legacy imports
export const token = derived(authStore, $auth => $auth.token);
export const isRoot = derived(authStore, $auth => $auth.user?.role?.toLowerCase() === 'root');
export const isTenantAdmin = derived(authStore, $auth => {
	const role = $auth.user?.role?.toLowerCase();
	return role === 'root' || role === 'tenantadmin' || role === 'tenant_admin' || role === 'tenant-admin'; // Handle potential variations
});

// Provide a legacy-compatible logout
export const logout = () => authStore.logout();
