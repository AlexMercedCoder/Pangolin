
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

// Mock auth store
export const authStore = {
    subscribe: vi.fn((run) => {
        run({ isAuthenticated: false, user: null, token: null });
        return () => {};
    }),
    login: vi.fn(),
    logout: vi.fn(),
    handleOAuthLogin: vi.fn(),
    reset: vi.fn()
};

// Mock tenant store
export const tenantStore = {
    subscribe: vi.fn((run) => {
        run({ tenants: [], selectedTenantId: null, loading: false, error: null });
        return () => {};
    }),
    loadTenants: vi.fn(),
    selectTenant: vi.fn(),
    reset: vi.fn()
};

// Mock notifications
export const notifications = {
    subscribe: writable([]).subscribe,
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
    remove: vi.fn()
};
