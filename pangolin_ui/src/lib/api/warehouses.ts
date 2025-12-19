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
	// AWS / MinIO
	role_arn?: string;
	external_id?: string;
	access_key_id?: string;
	secret_access_key?: string;
	// Azure
	tenant_id?: string;
	client_id?: string;
	client_secret?: string;
	// GCP
	project_id?: string;
}

export type VendingStrategy = 
	| { type: 'AwsSts'; role_arn: string; external_id?: string }
	| { type: 'AwsStatic'; access_key_id: string; secret_access_key: string }
	| { type: 'AzureSas'; account_name: string; account_key: string }
	| { type: 'GcpDownscoped'; service_account_email: string; private_key: string }
	| { type: 'None' };

export interface Warehouse {
	id: string;
	name: string;
	use_sts: boolean; // Deprecated but kept for compatibility
	storage_config: StorageConfig;
	vending_strategy?: VendingStrategy;
}

export interface CreateWarehouseRequest {
	name: string;
	use_sts: boolean;
	storage_config: StorageConfig;
	vending_strategy?: VendingStrategy;
}

export interface UpdateWarehouseRequest {
	use_sts?: boolean;
	storage_config?: Partial<StorageConfig>;
	vending_strategy?: VendingStrategy;
}

export const warehousesApi = {
	async list(): Promise<Warehouse[]> {
		const response = await apiClient.get<Warehouse[]>('/api/v1/warehouses');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(name: string): Promise<Warehouse> {
		const response = await apiClient.get<Warehouse>(`/api/v1/warehouses/${encodeURIComponent(name)}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateWarehouseRequest): Promise<Warehouse> {
		const response = await apiClient.post<Warehouse>('/api/v1/warehouses', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(name: string, data: UpdateWarehouseRequest): Promise<Warehouse> {
		const response = await apiClient.put<Warehouse>(`/api/v1/warehouses/${encodeURIComponent(name)}`, data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async delete(name: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/warehouses/${encodeURIComponent(name)}`);
		if (response.error) throw new Error(response.error.message);
	},
};
