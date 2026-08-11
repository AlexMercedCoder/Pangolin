
import { vi } from 'vitest';
import type { Readable } from 'svelte/store';
import { readable, writable } from 'svelte/store';

// Mock $app/navigation
export const goto = vi.fn();
export const invalidate = vi.fn();
export const invalidateAll = vi.fn();

// Mock $app/stores
export const page = readable({
    url: new URL('http://localhost'),
    params: {},
    route: { id: null },
    status: 200,
    error: null,
    data: {},
    form: null
});

export const navigating = readable(null);
export const updated = readable(false);

// Mock auth store.
//
// B46: these mocks were incomplete - `initialize`, `sessionExpired`,
// `updateSession` and the derived role stores were all missing. That went
// unnoticed because the suites importing them failed to *load* at all (the
// `import.meta.env` override in vitest.config.ts broke SvelteKit's virtual env
// module), so the mocks were never exercised. With the suites loading, an
// incomplete mock is a "not a function" TypeError, so they are completed here.
export const authStore = {
    subscribe: vi.fn((run) => {
        run({
            isAuthenticated: false,
            isLoading: false,
            authEnabled: true,
            user: null,
            token: null
        });
        return () => {};
    }),
    initialize: vi.fn().mockResolvedValue(undefined),
    login: vi.fn(),
    logout: vi.fn(),
    sessionExpired: vi.fn(),
    updateSession: vi.fn(),
    handleOAuthLogin: vi.fn(),
    reset: vi.fn()
};

export const isRoot = readable(false);
export const isTenantAdmin = readable(false);
export const isAuthenticated = readable(false);
export const user = readable(null);
export const token = readable(null);

// Mock tenant store
export const tenantStore = {
    subscribe: vi.fn((run) => {
        run({ selectedTenantId: null, selectedTenantName: null });
        return () => {};
    }),
    loadTenants: vi.fn(),
    selectTenant: vi.fn(),
    clearTenant: vi.fn(),
    reset: vi.fn()
};

// Mock notifications
export const notifications = {
    subscribe: writable([]).subscribe,
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
    remove: vi.fn(),
    clear: vi.fn()
};
