import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { authStore } from '$lib/stores/auth';
import { authApi } from '$lib/api/auth';

// The global test setup (`src/tests/setup.ts`) replaces this module with a stub
// so that *component* tests get a predictable API. This file is the unit test
// for the real thing, so it has to opt out - without this it silently asserts
// against the stub and fails on values it never set.
vi.unmock('$lib/stores/auth');

// Mock authApi
vi.mock('$lib/api/auth', () => ({
	authApi: {
		getAppConfig: vi.fn(),
		login: vi.fn(),
		logout: vi.fn(),
		getCurrentUser: vi.fn(),
		// B0j: logout revokes the token's `jti` server-side. Absent from this
		// mock, `logout()` threw "not a function" - and because the store calls
		// it before clearing local state, a real logout would have left the
		// session intact too if the method had ever gone missing.
		revokeCurrentToken: vi.fn().mockResolvedValue(undefined),
	}
}));

// Mock $app/environment
vi.mock('$app/environment', () => ({
	browser: true
}));

describe('Auth Store', () => {
	beforeEach(() => {
		// `resetAllMocks` clears implementations as well as calls, including the
		// ones set in the module factory above - so anything the store awaits
		// has to be re-established here.
		vi.resetAllMocks();
		(authApi.revokeCurrentToken as any).mockResolvedValue(undefined);
		localStorage.clear();
		authStore.logout(); // Reset store state
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	it('initialize - auth disabled (NO_AUTH) does not authenticate on its own', async () => {
		// This test used to assert that `initialize` established a no-auth
		// session by itself. It no longer does, deliberately: the layout
		// enforces a login even in NO_AUTH mode, and `loginNoAuth` below is
		// what creates the session. `initialize` only records the mode.
		(authApi.getAppConfig as any).mockResolvedValue({ auth_enabled: false });

		await authStore.initialize();

		const state = get(authStore);
		expect(state.authEnabled).toBe(false);
		expect(state.isAuthenticated).toBe(false);
		expect(state.isLoading).toBe(false);
		expect(localStorage.getItem('auth_token')).toBeNull();
	});

	it('initialize - auth disabled (NO_AUTH) restores a stored session', async () => {
		// The one case where `initialize` does authenticate: the user logged in
		// through the no-auth path on a previous visit.
		(authApi.getAppConfig as any).mockResolvedValue({ auth_enabled: false });
		localStorage.setItem('auth_token', 'no-auth-mode');
		localStorage.setItem(
			'auth_user',
			JSON.stringify({ id: 'no-auth-user', username: 'no-auth', role: 'root' })
		);

		await authStore.initialize();

		const state = get(authStore);
		expect(state.authEnabled).toBe(false);
		expect(state.isAuthenticated).toBe(true);
		expect(state.user?.username).toBe('no-auth');
		expect(state.token).toBe('no-auth-mode');
	});

	it('loginNoAuth establishes the session', async () => {
		(authApi.getAppConfig as any).mockResolvedValue({ auth_enabled: false });
		await authStore.initialize();

		const result = authStore.loginNoAuth();

		expect(result.success).toBe(true);
		const state = get(authStore);
		expect(state.isAuthenticated).toBe(true);
		expect(state.user?.username).toBe('no-auth');
		expect(state.token).toBe('no-auth-mode');
		expect(localStorage.getItem('auth_token')).toBe('no-auth-mode');
	});

	it('initialize - auth enabled', async () => {
		// Mock config returning auth_enabled: true
		(authApi.getAppConfig as any).mockResolvedValue({ auth_enabled: true });

		await authStore.initialize();

		const state = get(authStore);
		expect(state.authEnabled).toBe(true);
		expect(state.isAuthenticated).toBe(false); // Default empty state
	});

	it('initialize - regression: clear no-auth-mode token when auth is enabled', async () => {
		// Setup bad state
		localStorage.setItem('auth_token', 'no-auth-mode');
		localStorage.setItem('auth_user', JSON.stringify({ username: 'no-auth' }));

		// Mock config returning auth_enabled: true
		(authApi.getAppConfig as any).mockResolvedValue({ auth_enabled: true });

		await authStore.initialize();

		const state = get(authStore);
		expect(state.authEnabled).toBe(true);
		expect(state.isAuthenticated).toBe(false);
		expect(state.token).toBeNull();
		expect(localStorage.getItem('auth_token')).toBeNull(); // Should be cleared
	});

	it('login success', async () => {
		const mockUser = { id: '1', username: 'admin', role: 'Root' };
		const mockResponse = { token: 'jwt-token', user: mockUser };
		
		(authApi.login as any).mockResolvedValue(mockResponse);

		const result = await authStore.login('admin', 'password');

		expect(result.success).toBe(true);
		
		const state = get(authStore);
		expect(state.isAuthenticated).toBe(true);
		expect(state.token).toBe('jwt-token');
		expect(state.user).toEqual(mockUser);
		
		expect(localStorage.getItem('auth_token')).toBe('jwt-token');
	});

	it('login failure', async () => {
		(authApi.login as any).mockRejectedValue(new Error('Invalid credentials'));

		const result = await authStore.login('admin', 'wrong');

		expect(result.success).toBe(false);
		expect(result.error).toBe('Invalid credentials');
		
		const state = get(authStore);
		expect(state.isAuthenticated).toBe(false); // Should remain false (or whatever it was)
	});

	it('logout', async () => {
		// Set authenticated state first
		localStorage.setItem('auth_token', 'jwt-token');
		
		// We need to validly populate the store to test clearing
		// But let's just test that update runs and localStorage is cleared
		
		authStore.logout();

		const state = get(authStore);
		expect(state.isAuthenticated).toBe(false);
		expect(state.token).toBeNull();
		expect(state.user).toBeNull();
		expect(localStorage.getItem('auth_token')).toBeNull();
	});
});
