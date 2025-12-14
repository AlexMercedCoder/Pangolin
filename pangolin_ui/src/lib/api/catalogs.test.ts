import { describe, it, expect, beforeEach, vi } from 'vitest';
import { catalogsApi } from '$lib/api/catalogs';
import { apiClient } from '$lib/api/client';

vi.mock('$lib/api/client');

describe('Catalogs API', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('lists catalogs', async () => {
		const mockCatalogs = [
			{ id: '1', name: 'catalog1', warehouse_name: 'wh1', storage_location: 's3://bucket1/' },
			{ id: '2', name: 'catalog2', warehouse_name: 'wh2', storage_location: 's3://bucket2/' }
		];

		vi.mocked(apiClient.get).mockResolvedValueOnce({
			data: mockCatalogs,
			error: null
		});

		const result = await catalogsApi.list();

		expect(result.data).toEqual(mockCatalogs);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/catalogs');
	});

	it('gets single catalog', async () => {
		const mockCatalog = {
			id: '1',
			name: 'test-catalog',
			warehouse_name: 'test-warehouse',
			storage_location: 's3://test-bucket/catalog/'
		};

		vi.mocked(apiClient.get).mockResolvedValueOnce({
			data: mockCatalog,
			error: null
		});

		const result = await catalogsApi.get('test-catalog');

		expect(result.data).toEqual(mockCatalog);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/catalogs/test-catalog');
	});

	it('creates catalog', async () => {
		const newCatalog = {
			name: 'new-catalog',
			warehouse_name: 'warehouse1',
			storage_location: 's3://bucket/new-catalog/'
		};

		const createdCatalog = { id: '3', ...newCatalog };

		vi.mocked(apiClient.post).mockResolvedValueOnce({
			data: createdCatalog,
			error: null
		});

		const result = await catalogsApi.create(newCatalog);

		expect(result.data).toEqual(createdCatalog);
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/catalogs', newCatalog);
	});

	it('deletes catalog', async () => {
		vi.mocked(apiClient.delete).mockResolvedValueOnce({
			data: null,
			error: null
		});

		const result = await catalogsApi.delete('test-catalog');

		expect(result.error).toBeNull();
		expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/catalogs/test-catalog');
	});

	it('getCatalog convenience method unwraps response', async () => {
		const mockCatalog = {
			id: '1',
			name: 'test',
			warehouse_name: 'wh1',
			storage_location: 's3://bucket/'
		};

		vi.mocked(apiClient.get).mockResolvedValueOnce({
			data: mockCatalog,
			error: null
		});

		const result = await catalogsApi.getCatalog('test');

		expect(result).toEqual(mockCatalog);
	});

	it('deleteCatalog convenience method calls delete', async () => {
		vi.mocked(apiClient.delete).mockResolvedValueOnce({
			data: null,
			error: null
		});

		await catalogsApi.deleteCatalog('test');

		expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/catalogs/test');
	});
});
