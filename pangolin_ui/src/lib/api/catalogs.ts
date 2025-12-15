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
};
