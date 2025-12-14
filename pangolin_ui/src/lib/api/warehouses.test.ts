import { describe, it, expect, beforeEach, vi } from 'vitest';
import { warehousesApi } from '$lib/api/warehouses';
import { apiClient } from '$lib/api/client';

vi.mock('$lib/api/client');

describe('Warehouses API', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('lists warehouses', async () => {
		const mockWarehouses = [
			{ id: '1', name: 'warehouse1', use_sts: false, storage_config: {} },
			{ id: '2', name: 'warehouse2', use_sts: true, storage_config: {} }
		];

		vi.mocked(apiClient.get).mockResolvedValueOnce({
			data: mockWarehouses,
			error: null
		});

		const result = await warehousesApi.list();

		expect(result.data).toEqual(mockWarehouses);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/warehouses');
	});

	it('gets single warehouse', async () => {
		const mockWarehouse = {
			id: '1',
			name: 'test-warehouse',
			use_sts: false,
			storage_config: { type: 's3', bucket: 'test' }
		};

		vi.mocked(apiClient.get).mockResolvedValueOnce({
			data: mockWarehouse,
			error: null
		});

		const result = await warehousesApi.get('test-warehouse');

		expect(result.data).toEqual(mockWarehouse);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/warehouses/test-warehouse');
	});

	it('creates warehouse', async () => {
		const newWarehouse = {
			name: 'new-warehouse',
			use_sts: false,
			storage_config: { type: 's3', bucket: 'new-bucket' }
		};

		const createdWarehouse = { id: '3', ...newWarehouse };

		vi.mocked(apiClient.post).mockResolvedValueOnce({
			data: createdWarehouse,
			error: null
		});

		const result = await warehousesApi.create(newWarehouse);

		expect(result.data).toEqual(createdWarehouse);
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/warehouses', newWarehouse);
	});

	it('getWarehouse convenience method unwraps response', async () => {
		const mockWarehouse = {
			id: '1',
			name: 'test',
			use_sts: false,
			storage_config: {}
		};

		vi.mocked(apiClient.get).mockResolvedValueOnce({
			data: mockWarehouse,
			error: null
		});

		const result = await warehousesApi.getWarehouse('test');

		expect(result).toEqual(mockWarehouse);
	});
});
