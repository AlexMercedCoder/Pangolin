import { apiClient, type ApiResponse } from './client';

export interface Catalog {
	id: string;
	name: string;
	warehouse_name: string;
	storage_location: string;
	properties: Record<string, string>;
}

export interface CreateCatalogRequest {
	name: string;
	warehouse_name: string;
	storage_location: string;
	properties?: Record<string, string>;
}

export interface UpdateCatalogRequest {
	warehouse_name?: string;
	storage_location?: string;
	properties?: Record<string, string>;
}

export const catalogsApi = {
	async list(): Promise<ApiResponse<Catalog[]>> {
		return apiClient.get<Catalog[]>('/api/v1/catalogs');
	},

	async get(name: string): Promise<ApiResponse<Catalog>> {
		return apiClient.get<Catalog>(`/api/v1/catalogs/${encodeURIComponent(name)}`);
	},

	async create(data: CreateCatalogRequest): Promise<ApiResponse<Catalog>> {
		return apiClient.post<Catalog>('/api/v1/catalogs', data);
	},

	async update(name: string, data: UpdateCatalogRequest): Promise<ApiResponse<Catalog>> {
		return apiClient.put<Catalog>(`/api/v1/catalogs/${encodeURIComponent(name)}`, data);
	},

	async delete(name: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/api/v1/catalogs/${encodeURIComponent(name)}`);
	},
};
