import { describe, it, expect, beforeEach, vi } from 'vitest';
import { apiClient } from '$lib/api/client';

// Mock fetch
global.fetch = vi.fn();

describe('API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('makes GET request successfully', async () => {
		const mockData = { id: 1, name: 'Test' };
		(global.fetch as any).mockResolvedValueOnce({
			ok: true,
			json: async () => mockData
		});

		const result = await apiClient.get('/test');

		expect(result.data).toEqual(mockData);
		expect(result.error).toBeNull();
		expect(global.fetch).toHaveBeenCalledWith(
			expect.stringContaining('/test'),
			expect.objectContaining({
				method: 'GET'
			})
		);
	});

	it('handles GET request error', async () => {
		(global.fetch as any).mockResolvedValueOnce({
			ok: false,
			status: 404,
			statusText: 'Not Found'
		});

		const result = await apiClient.get('/test');

		expect(result.data).toBeNull();
		expect(result.error).toBeTruthy();
		expect(result.error?.message).toContain('404');
	});

	it('makes POST request with data', async () => {
		const mockResponse = { id: 1, created: true };
		const postData = { name: 'New Item' };

		(global.fetch as any).mockResolvedValueOnce({
			ok: true,
			json: async () => mockResponse
		});

		const result = await apiClient.post('/test', postData);

		expect(result.data).toEqual(mockResponse);
		expect(global.fetch).toHaveBeenCalledWith(
			expect.stringContaining('/test'),
			expect.objectContaining({
				method: 'POST',
				body: JSON.stringify(postData)
			})
		);
	});

	it('makes DELETE request', async () => {
		(global.fetch as any).mockResolvedValueOnce({
			ok: true,
			json: async () => ({})
		});

		const result = await apiClient.delete('/test/123');

		expect(result.error).toBeNull();
		expect(global.fetch).toHaveBeenCalledWith(
			expect.stringContaining('/test/123'),
			expect.objectContaining({
				method: 'DELETE'
			})
		);
	});

	it('handles network error', async () => {
		(global.fetch as any).mockRejectedValueOnce(new Error('Network error'));

		const result = await apiClient.get('/test');

		expect(result.data).toBeNull();
		expect(result.error).toBeTruthy();
		expect(result.error?.message).toContain('Network error');
	});
});
