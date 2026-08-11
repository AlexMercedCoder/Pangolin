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
		// B46: the mock used to provide only `ok` and `json()`. The client reads
		// the body with `response.text()`, so every one of these "passing" tests
		// actually threw inside the client and fell through to the catch arm -
		// they asserted on the request headers, which are set before the throw,
		// and so passed while exercising the error path. A mock has to answer
		// the calls the code under test makes.
		(global.fetch as any).mockResolvedValue({
			ok: true,
			status: 200,
			statusText: 'OK',
			json: async () => ({ data: 'test' }),
			text: async () => JSON.stringify({ data: 'test' })
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

	// B46 regression: with a mock that lacks `.text()`, the client throws and
	// returns an error. Asserting on the *response* - not just the request -
	// is what distinguishes a working call from one that silently failed.
	it('returns the parsed body on success', async () => {
		const res = await apiClient.get<{ data: string }>('/api/v1/catalogs');

		expect(res.error).toBeUndefined();
		expect(res.data).toEqual({ data: 'test' });
	});

	it('surfaces a structured error envelope as a message', async () => {
		(global.fetch as any).mockResolvedValue({
			ok: false,
			status: 409,
			statusText: 'Conflict',
			json: async () => ({
				error: { message: 'ref main points at 222, expected 111', type: 'CommitFailedException', code: 409 }
			}),
			text: async () => ''
		});

		const res = await apiClient.get('/api/v1/catalogs');

		expect(res.error?.status).toBe(409);
		// Previously `errorData.error` was an object, so this rendered as
		// "[object Object]" in the UI.
		expect(res.error?.message).toBe('ref main points at 222, expected 111');
	});
});
