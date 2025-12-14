import { writable, derived } from 'svelte/store';
import type { User } from '$lib/api/auth';

interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
}

const initialState: AuthState = {
	user: null,
	token: null,
	isAuthenticated: false,
	isLoading: true,
};

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,
		setUser: (user: User, token: string) => {
			localStorage.setItem('auth_token', token);
			localStorage.setItem('auth_user', JSON.stringify(user));
			update(state => ({
				...state,
				user,
				token,
				isAuthenticated: true,
				isLoading: false,
			}));
		},
		clearUser: () => {
			localStorage.removeItem('auth_token');
			localStorage.removeItem('auth_user');
			set({
				user: null,
				token: null,
				isAuthenticated: false,
				isLoading: false,
			});
		},
		loadFromStorage: () => {
			const token = localStorage.getItem('auth_token');
			const userStr = localStorage.getItem('auth_user');
			
			if (token && userStr) {
				try {
					const user = JSON.parse(userStr) as User;
					update(state => ({
						...state,
						user,
						token,
						isAuthenticated: true,
						isLoading: false,
					}));
				} catch (e) {
					// Invalid stored data, clear it
					localStorage.removeItem('auth_token');
					localStorage.removeItem('auth_user');
					update(state => ({ ...state, isLoading: false }));
				}
			} else {
				update(state => ({ ...state, isLoading: false }));
			}
		},
		setLoading: (isLoading: boolean) => {
			update(state => ({ ...state, isLoading }));
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
