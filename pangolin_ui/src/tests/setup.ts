import '@testing-library/jest-dom';
import { vi, afterEach } from 'vitest';
import * as mocks from './mocks';

// SvelteKit's virtual env module. Tests run outside a Vite dev/build pipeline,
// so the real virtual module is not available; the client reads
// `env.PUBLIC_API_URL` from here.
vi.mock('$env/dynamic/public', () => ({
	env: {
		PUBLIC_API_URL: 'http://localhost:8080'
	}
}));

// Mock SvelteKit modules
vi.mock('$app/environment', () => ({
	browser: true,
	dev: true,
	building: false,
	version: 'test'
}));

vi.mock('$app/navigation', () => ({
	goto: mocks.goto,
	invalidate: mocks.invalidate,
	invalidateAll: mocks.invalidateAll,
	preloadData: vi.fn(),
	preloadCode: vi.fn(),
	beforeNavigate: vi.fn(),
	afterNavigate: vi.fn()
}));

vi.mock('$app/stores', () => ({
	getStores: () => ({
		page: mocks.page,
		navigating: mocks.navigating,
		updated: mocks.updated
	}),
	page: mocks.page,
	navigating: mocks.navigating,
	updated: mocks.updated
}));

vi.mock('$lib/stores/auth', () => ({
	authStore: mocks.authStore,
	// The layout and several pages import these derived stores directly.
	isRoot: mocks.isRoot,
	isTenantAdmin: mocks.isTenantAdmin,
	isAuthenticated: mocks.isAuthenticated,
	user: mocks.user,
	token: mocks.token,
	logout: mocks.authStore.logout
}));

vi.mock('$lib/stores/tenant', () => ({
	tenantStore: mocks.tenantStore,
	// The API client imports this constant to read the selected tenant out of
	// localStorage; without it the mock made the client module fail to load.
	TENANT_STORAGE_KEY: 'pangolin_selected_tenant'
}));

vi.mock('$lib/stores/notifications', () => ({
	notifications: mocks.notifications
}));

// Mock API modules
// B46: the warehouse and catalog mocks only declared `list` and `delete`, so an
// edit page calling `get`/`create`/`update` hit "not a function". Kept in step
// with the real clients.
vi.mock('$lib/api/tenants', () => ({
    tenantsApi: {
        list: vi.fn().mockResolvedValue([]),
        get: vi.fn().mockResolvedValue(null),
        create: vi.fn().mockResolvedValue(null),
        update: vi.fn().mockResolvedValue(null),
        delete: vi.fn().mockResolvedValue(undefined)
    }
}));

vi.mock('$lib/api/warehouses', () => ({
    warehousesApi: {
        list: vi.fn().mockResolvedValue([]),
        get: vi.fn().mockResolvedValue(null),
        create: vi.fn().mockResolvedValue(null),
        update: vi.fn().mockResolvedValue(null),
        delete: vi.fn().mockResolvedValue(undefined)
    }
}));

vi.mock('$lib/api/catalogs', () => ({
    catalogsApi: {
        list: vi.fn().mockResolvedValue([]),
        get: vi.fn().mockResolvedValue(null),
        getSummary: vi.fn().mockResolvedValue(null),
        create: vi.fn().mockResolvedValue(null),
        update: vi.fn().mockResolvedValue(null),
        testConnection: vi.fn().mockResolvedValue(undefined),
        delete: vi.fn().mockResolvedValue(undefined)
    }
}));

// Clean up after each test
afterEach(() => {
	vi.clearAllMocks();
});
