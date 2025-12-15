import { describe, it, expect, beforeEach, vi } from 'vitest';
import { authApi } from './auth';
import { apiClient } from './client';

// Mock apiClient
vi.mock('./client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
	}
}));

describe('Auth API', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('getAppConfig unwraps data', async () => {
		const mockData = { auth_enabled: true };
		(apiClient.get as any).mockResolvedValue({ data: mockData });

		const result = await authApi.getAppConfig();
		expect(result).toEqual(mockData);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/app-config');
	});

	it('getAppConfig throws on error', async () => {
		const mockError = { message: 'Failed' };
		(apiClient.get as any).mockResolvedValue({ error: mockError });

		await expect(authApi.getAppConfig()).rejects.toThrow('Failed');
	});

	it('login unwraps data', async () => {
		const mockData = { token: 't', user: { id: '1' } };
		(apiClient.post as any).mockResolvedValue({ data: mockData });

		const result = await authApi.login({ username: 'u', password: 'p' });
		expect(result).toEqual(mockData);
	});

	it('login throws on error', async () => {
		(apiClient.post as any).mockResolvedValue({ error: { message: 'Invalid' } });
		await expect(authApi.login({ username: 'u', password: 'p' })).rejects.toThrow('Invalid');
	});

	// Regression test for unwrapping logic
	it('logout unwraps success', async () => {
		(apiClient.post as any).mockResolvedValue({ data: undefined });
		await expect(authApi.logout()).resolves.toBeUndefined();
	});
});
