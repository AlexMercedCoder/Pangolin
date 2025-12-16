import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CatalogsPage from './+page.svelte';
import { catalogsApi } from '$lib/api/catalogs';
import { tenantStore } from '$lib/stores/tenant';

// Mock the API
vi.mock('$lib/api/catalogs', () => ({
	catalogsApi: {
		list: vi.fn()
	}
}));

// Mock navigation
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

describe('Catalogs Page - Tenant Isolation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		tenantStore.clearTenant();
	});

	it('should reload catalogs when tenant changes', async () => {
		const mockCatalogs = [
			{ id: '1', name: 'catalog-1', warehouse_name: null, storage_location: 's3://bucket' }
		];

		(catalogsApi.list as any).mockResolvedValue(mockCatalogs);

		render(CatalogsPage);

		// Wait for initial load
		await waitFor(() => {
			expect(catalogsApi.list).toHaveBeenCalledTimes(1);
		});

		// Change tenant
		tenantStore.selectTenant('tenant-123', 'Test Tenant');

		// Should trigger reload
		await waitFor(() => {
			expect(catalogsApi.list).toHaveBeenCalledTimes(2);
		});
	});

	it('should display catalogs from API', async () => {
		const mockCatalogs = [
			{ id: '1', name: 'test-catalog', warehouse_name: 'test-warehouse', storage_location: 's3://bucket/path' }
		];

		(catalogsApi.list as any).mockResolvedValue(mockCatalogs);

		render(CatalogsPage);

		await waitFor(() => {
			expect(screen.getByText('test-catalog')).toBeInTheDocument();
		});
	});

	it('should show empty state when no catalogs exist', async () => {
		(catalogsApi.list as any).mockResolvedValue([]);

		render(CatalogsPage);

		await waitFor(() => {
			expect(screen.getByText(/no catalogs found/i)).toBeInTheDocument();
		});
	});

	it('should handle API errors gracefully', async () => {
		(catalogsApi.list as any).mockRejectedValue(new Error('Failed to fetch'));

		render(CatalogsPage);

		await waitFor(() => {
			// Should still render the page without crashing
			expect(screen.getByText('Catalogs')).toBeInTheDocument();
		});
	});

	it('should clear catalogs when switching to empty tenant', async () => {
		// Start with catalogs
		(catalogsApi.list as any).mockResolvedValue([
			{ id: '1', name: 'catalog-1', warehouse_name: null, storage_location: 's3://bucket' }
		]);

		render(CatalogsPage);

		await waitFor(() => {
			expect(screen.getByText('catalog-1')).toBeInTheDocument();
		});

		// Switch to tenant with no catalogs
		(catalogsApi.list as any).mockResolvedValue([]);
		tenantStore.selectTenant('empty-tenant', 'Empty Tenant');

		await waitFor(() => {
			expect(screen.queryByText('catalog-1')).not.toBeInTheDocument();
			expect(screen.getByText(/no catalogs found/i)).toBeInTheDocument();
		});
	});
});
