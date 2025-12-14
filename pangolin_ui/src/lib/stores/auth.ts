import { writable, derived } from 'svelte/store';
import type { User } from '$lib/api/auth';
import { authApi } from '$lib/api/auth';
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
			// Check if server has auth enabled
			const config = await authApi.getAppConfig();
			
			if (!config.auth_enabled) {
				// NO_AUTH mode - auto-authenticate with mock session
				const mockUser: User = {
					id: 'no-auth-user',
					username: 'no-auth',
					role: 'Root',
				};
				
				update(state => ({
					...state,
					authEnabled: false,
					isAuthenticated: true,
					user: mockUser,
					token: 'no-auth-mode',
					isLoading: false,
				}));
				
				// Store in localStorage for consistency
				if (browser) {
					localStorage.setItem('auth_token', 'no-auth-mode');
					localStorage.setItem('auth_user', JSON.stringify(mockUser));
				}
				return;
			}

			// Auth is enabled - check for existing token
			update(state => ({ ...state, authEnabled: true }));
			
			if (browser) {
				const token = localStorage.getItem('auth_token');
				const userStr = localStorage.getItem('auth_user');

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
		async login(username: string, password: string) {
			try {
				const response = await authApi.login({ username, password });
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
		logout() {
			update(state => ({
				...state,
				token: null,
				user: null,
				isAuthenticated: false,
			}));

			if (browser) {
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}
		},
	};
}

export const authStore = createAuthStore();

// Derived stores for convenience
export const isAuthenticated = derived(authStore, $auth => $auth.isAuthenticated);
export const currentUser = derived(authStore, $auth => $auth.user);
export const isRoot = derived(authStore, $auth => $auth.user?.role === 'Root');
export const isTenantAdmin = derived(authStore, $auth => 
	$auth.user?.role === 'Root' || $auth.user?.role === 'TenantAdmin'
);
