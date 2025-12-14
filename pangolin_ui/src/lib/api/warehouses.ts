import { apiClient, type ApiResponse } from './client';

export interface StorageConfig {
	type: 's3' | 'azure' | 'gcs';
	bucket?: string;
	region?: string;
	endpoint?: string;
	container?: string;
	account_name?: string;
	account_key?: string;
	service_account_json?: string;
}

export interface Warehouse {
	id: string;
	name: string;
	use_sts: boolean;
	storage_config: StorageConfig;
}

export interface CreateWarehouseRequest {
	name: string;
	use_sts: boolean;
	storage_config: StorageConfig;
}

export interface UpdateWarehouseRequest {
	use_sts?: boolean;
	storage_config?: Partial<StorageConfig>;
}

export const warehousesApi = {
	async list(): Promise<ApiResponse<Warehouse[]>> {
		return apiClient.get<Warehouse[]>('/api/v1/warehouses');
	},

	async get(name: string): Promise<ApiResponse<Warehouse>> {
		return apiClient.get<Warehouse>(`/api/v1/warehouses/${encodeURIComponent(name)}`);
	},

	async create(data: CreateWarehouseRequest): Promise<ApiResponse<Warehouse>> {
		return apiClient.post<Warehouse>('/api/v1/warehouses', data);
	},

	async update(name: string, data: UpdateWarehouseRequest): Promise<ApiResponse<Warehouse>> {
		return apiClient.put<Warehouse>(`/api/v1/warehouses/${encodeURIComponent(name)}`, data);
	},

	async delete(name: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/api/v1/warehouses/${encodeURIComponent(name)}`);
	},

	// Convenience methods that unwrap ApiResponse
	async getWarehouse(name: string): Promise<Warehouse> {
		const response = await this.get(name);
		return response.data;
	},

	async deleteWarehouse(name: string): Promise<void> {
		await this.delete(name);
	},
};
