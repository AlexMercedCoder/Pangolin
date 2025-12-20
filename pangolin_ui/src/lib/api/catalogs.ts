import { apiClient, type ApiResponse } from './client';

export type CatalogType = 'Local' | 'Federated';

export type FederatedAuthType = 'None' | 'BasicAuth' | 'BearerToken' | 'ApiKey';

export interface FederatedCredentials {
	username?: string;
	password?: string;
	token?: string;
	api_key?: string;
}

export interface FederatedCatalogConfig {
	base_url: string;
	auth_type: FederatedAuthType;
	credentials?: FederatedCredentials;
	timeout_seconds: number;
}

export interface Catalog {
	id: string;
	name: string;
	catalog_type: CatalogType;
	warehouse_name?: string;
	storage_location?: string;
	federated_config?: FederatedCatalogConfig;
	properties: Record<string, string>;
}

export interface CreateCatalogRequest {
	name: string;
	catalog_type?: CatalogType;
	warehouse_name?: string;
	storage_location?: string;
	federated_config?: FederatedCatalogConfig;
	properties?: Record<string, string>;
}

export interface UpdateCatalogRequest {
	warehouse_name?: string;
	storage_location?: string;
	federated_config?: FederatedCatalogConfig;
	properties?: Record<string, string>;
}

export const catalogsApi = {
	async list(): Promise<Catalog[]> {
		const response = await apiClient.get<Catalog[]>('/api/v1/catalogs');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(name: string): Promise<Catalog> {
		const response = await apiClient.get<Catalog>(`/api/v1/catalogs/${encodeURIComponent(name)}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateCatalogRequest): Promise<Catalog> {
		const response = await apiClient.post<Catalog>('/api/v1/catalogs', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(name: string, data: UpdateCatalogRequest): Promise<Catalog> {
		const response = await apiClient.put<Catalog>(`/api/v1/catalogs/${encodeURIComponent(name)}`, data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async delete(name: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/catalogs/${encodeURIComponent(name)}`);
		if (response.error) throw new Error(response.error.message);
	},

	async testConnection(name: string): Promise<{ status: string; message?: string }> {
		const response = await apiClient.post<{ status: string; message?: string }>(`/api/v1/federated-catalogs/${encodeURIComponent(name)}/test`, {});
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

    async getStats(name: string): Promise<SyncStats> {
        const response = await apiClient.get<SyncStats>(`/api/v1/federated-catalogs/${encodeURIComponent(name)}/stats`);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async sync(name: string): Promise<void> {
        const response = await apiClient.post<void>(`/api/v1/federated-catalogs/${encodeURIComponent(name)}/sync`, {});
        if (response.error) throw new Error(response.error.message);
    }
};

export interface SyncStats {
    last_synced_at?: string;
    sync_status: string;
    tables_synced: number;
    namespaces_synced: number;
    error_message?: string;
}
