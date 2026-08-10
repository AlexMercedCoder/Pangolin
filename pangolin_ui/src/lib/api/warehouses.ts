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

	// The server models `storage_config` as a free-form `HashMap<String,
	// String>`, and both this UI and the backend use dotted keys that are not
	// listed above (`s3.bucket`, `adls.account-name`, `s3.path-style-access`).
	// Without this index signature every one of those reads is a type error,
	// which is most of what `svelte-check` reports for the warehouse pages.
	[key: string]: string | undefined;
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
	async list(limit?: number, offset?: number): Promise<Warehouse[]> {
		const params = new URLSearchParams();
		if (limit) params.append('limit', limit.toString());
		if (offset) params.append('offset', offset.toString());

		// An empty `params` used to leave a bare `?` on the end of every
		// unparameterised list call.
		const query = params.toString();
		const response = await apiClient.get<Warehouse[]>(`/api/v1/warehouses${query ? `?${query}` : ''}`);
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
