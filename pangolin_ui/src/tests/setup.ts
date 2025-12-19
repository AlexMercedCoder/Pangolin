import '@testing-library/jest-dom';
import { vi, afterEach } from 'vitest';
import * as mocks from './mocks';

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
	authStore: mocks.authStore
}));

vi.mock('$lib/stores/tenant', () => ({
	tenantStore: mocks.tenantStore
}));

vi.mock('$lib/stores/notifications', () => ({
	notifications: mocks.notifications
}));

// Mock API modules
vi.mock('$lib/api/tenants', () => ({
    tenantsApi: {
        list: vi.fn().mockResolvedValue([]),
        get: vi.fn(),
        create: vi.fn(),
        update: vi.fn(),
        delete: vi.fn()
    }
}));

vi.mock('$lib/api/warehouses', () => ({
    warehousesApi: {
        list: vi.fn().mockResolvedValue([]),
        delete: vi.fn()
    }
}));

vi.mock('$lib/api/catalogs', () => ({
    catalogsApi: {
        list: vi.fn().mockResolvedValue([]),
        delete: vi.fn()
    }
}));

// Clean up after each test
afterEach(() => {
	vi.clearAllMocks();
});
