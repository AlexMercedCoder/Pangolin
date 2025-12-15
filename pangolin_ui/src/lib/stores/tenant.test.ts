import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { tenantStore, TENANT_STORAGE_KEY } from '$lib/stores/tenant';

// Mock browser environment
vi.mock('$app/environment', () => ({
	browser: true
}));

describe('tenantStore', () => {
	beforeEach(() => {
		// Clear localStorage before each test
		localStorage.clear();
		// Reset the store
		tenantStore.clearTenant();
	});

	it('should initialize with no tenant selected', () => {
		const state = get(tenantStore);
		expect(state.selectedTenantId).toBeNull();
		expect(state.selectedTenantName).toBeNull();
	});

	it('should select tenant and persist to localStorage', () => {
		const tenantId = '123e4567-e89b-12d3-a456-426614174000';
		const tenantName = 'Test Tenant';

		tenantStore.selectTenant(tenantId, tenantName);

		const state = get(tenantStore);
		expect(state.selectedTenantId).toBe(tenantId);
		expect(state.selectedTenantName).toBe(tenantName);
		
		// Verify localStorage
		expect(localStorage.getItem(TENANT_STORAGE_KEY)).toBe(tenantId);
		expect(localStorage.getItem(`${TENANT_STORAGE_KEY}_name`)).toBe(tenantName);
	});

	it('should clear tenant and remove from localStorage', () => {
		// First set a tenant
		tenantStore.selectTenant('test-id', 'Test Tenant');
		
		// Then clear it
		tenantStore.clearTenant();

		const state = get(tenantStore);
		expect(state.selectedTenantId).toBeNull();
		expect(state.selectedTenantName).toBeNull();
		
		// Verify localStorage is cleared
		expect(localStorage.getItem(TENANT_STORAGE_KEY)).toBeNull();
		expect(localStorage.getItem(`${TENANT_STORAGE_KEY}_name`)).toBeNull();
	});

	it('should handle switching between tenants', () => {
		// Set first tenant
		tenantStore.selectTenant('tenant-1', 'Tenant One');
		let state = get(tenantStore);
		expect(state.selectedTenantId).toBe('tenant-1');

		// Switch to second tenant
		tenantStore.selectTenant('tenant-2', 'Tenant Two');
		state = get(tenantStore);
		expect(state.selectedTenantId).toBe('tenant-2');
		expect(state.selectedTenantName).toBe('Tenant Two');
		
		// Verify localStorage was updated
		expect(localStorage.getItem(TENANT_STORAGE_KEY)).toBe('tenant-2');
	});

	it('should notify subscribers when tenant changes', () => {
		const mockSubscriber = vi.fn();
		
		const unsubscribe = tenantStore.subscribe(mockSubscriber);
		
		// Should be called once on subscription with initial value
		expect(mockSubscriber).toHaveBeenCalledTimes(1);
		
		// Change tenant
		tenantStore.selectTenant('new-tenant', 'New Tenant');
		
		// Should be called again with new value
		expect(mockSubscriber).toHaveBeenCalledTimes(2);
		expect(mockSubscriber).toHaveBeenLastCalledWith({
			selectedTenantId: 'new-tenant',
			selectedTenantName: 'New Tenant'
		});
		
		unsubscribe();
	});
});
