import { apiClient } from './client';

export interface Tenant {
	id: string;
	name: string;
	description?: string;
	properties?: Record<string, any>;
	created_at?: string;
}

export interface CreateTenantRequest {
	name: string;
	description?: string;
	properties?: Record<string, any>;
}

export interface UpdateTenantRequest {
	name?: string;
	description?: string;
	properties?: Record<string, any>;
}

export const tenantsApi = {
	async list(limit?: number, offset?: number): Promise<Tenant[]> {
		const params = new URLSearchParams();
		if (limit) params.append('limit', limit.toString());
		if (offset) params.append('offset', offset.toString());
		
		const response = await apiClient.get<Tenant[]>(`/api/v1/tenants?${params.toString()}`);
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(id: string): Promise<Tenant> {
		const response = await apiClient.get<Tenant>(`/api/v1/tenants/${id}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateTenantRequest): Promise<Tenant> {
		const response = await apiClient.post<Tenant>('/api/v1/tenants', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(id: string, data: UpdateTenantRequest): Promise<Tenant> {
		const response = await apiClient.put<Tenant>(`/api/v1/tenants/${id}`, data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async delete(id: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/tenants/${id}`);
		if (response.error) throw new Error(response.error.message);
	},
};
