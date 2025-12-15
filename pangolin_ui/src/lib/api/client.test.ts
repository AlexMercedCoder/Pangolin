import { describe, it, expect, vi, beforeEach } from 'vitest';
import { apiClient } from '$lib/api/client';
import { TENANT_STORAGE_KEY } from '$lib/stores/tenant';

// Mock fetch
global.fetch = vi.fn();

// Mock browser environment
vi.mock('$app/environment', () => ({
	browser: true
}));

describe('apiClient tenant header handling', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		(global.fetch as any).mockResolvedValue({
			ok: true,
			json: async () => ({ data: 'test' })
		});
	});

	it('should include X-Pangolin-Tenant header when tenant is selected', async () => {
		const tenantId = '123e4567-e89b-12d3-a456-426614174000';
		localStorage.setItem(TENANT_STORAGE_KEY, tenantId);

		await apiClient.get('/api/v1/catalogs');

		expect(global.fetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/v1/catalogs'),
			expect.objectContaining({
				headers: expect.objectContaining({
					'X-Pangolin-Tenant': tenantId
				})
			})
		);
	});

	it('should not include X-Pangolin-Tenant header when no tenant is selected', async () => {
		// No tenant in localStorage
		await apiClient.get('/api/v1/catalogs');

		const fetchCall = (global.fetch as any).mock.calls[0];
		const headers = fetchCall[1].headers;
		
		expect(headers['X-Pangolin-Tenant']).toBeUndefined();
	});

	it('should update header when tenant changes', async () => {
		// First request with tenant A
		localStorage.setItem(TENANT_STORAGE_KEY, 'tenant-a');
		await apiClient.get('/api/v1/catalogs');

		expect(global.fetch).toHaveBeenLastCalledWith(
			expect.any(String),
			expect.objectContaining({
				headers: expect.objectContaining({
					'X-Pangolin-Tenant': 'tenant-a'
				})
			})
		);

		// Change to tenant B
		localStorage.setItem(TENANT_STORAGE_KEY, 'tenant-b');
		await apiClient.get('/api/v1/warehouses');

		expect(global.fetch).toHaveBeenLastCalledWith(
			expect.any(String),
			expect.objectContaining({
				headers: expect.objectContaining({
					'X-Pangolin-Tenant': 'tenant-b'
				})
			})
		);
	});

	it('should include Authorization header along with tenant header', async () => {
		const tenantId = 'test-tenant';
		const token = 'test-token';
		
		localStorage.setItem(TENANT_STORAGE_KEY, tenantId);
		localStorage.setItem('auth_token', token);

		await apiClient.get('/api/v1/catalogs');

		expect(global.fetch).toHaveBeenCalledWith(
			expect.any(String),
			expect.objectContaining({
				headers: expect.objectContaining({
					'X-Pangolin-Tenant': tenantId,
					'Authorization': `Bearer ${token}`
				})
			})
		);
	});

	it('should handle POST requests with tenant header', async () => {
		const tenantId = 'test-tenant';
		localStorage.setItem(TENANT_STORAGE_KEY, tenantId);

		await apiClient.post('/api/v1/catalogs', { name: 'test-catalog' });

		expect(global.fetch).toHaveBeenCalledWith(
			expect.any(String),
			expect.objectContaining({
				method: 'POST',
				headers: expect.objectContaining({
					'X-Pangolin-Tenant': tenantId,
					'Content-Type': 'application/json'
				}),
				body: JSON.stringify({ name: 'test-catalog' })
			})
		);
	});
});
