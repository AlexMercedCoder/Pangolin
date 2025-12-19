import { apiClient } from './client';

export type UserRole = 'Root' | 'TenantAdmin' | 'TenantUser';

export interface ServiceUser {
	id: string;
	name: string;
	description?: string;
	tenant_id: string;
	role: UserRole;
	created_by: string;
	created_at: string;
	expires_at?: string;
	active: boolean;
}

export interface CreateServiceUserRequest {
	name: string;
	description?: string;
	role: UserRole;
	expires_in_days?: number;
}

export interface UpdateServiceUserRequest {
	name?: string;
	description?: string;
	active?: boolean;
}

export interface ApiKeyResponse {
	service_user_id: string;
	name: string;
	api_key: string;
	expires_at?: string;
}

export const serviceUsersApi = {
	async list(): Promise<ServiceUser[]> {
		const response = await apiClient.get<ServiceUser[]>('/api/v1/service-users');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(id: string): Promise<ServiceUser> {
		const response = await apiClient.get<ServiceUser>(`/api/v1/service-users/${id}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateServiceUserRequest): Promise<ApiKeyResponse> {
		const response = await apiClient.post<ApiKeyResponse>('/api/v1/service-users', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(id: string, data: UpdateServiceUserRequest): Promise<void> {
		const response = await apiClient.put<void>(`/api/v1/service-users/${id}`, data);
		if (response.error) throw new Error(response.error.message);
	},

	async delete(id: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/service-users/${id}`);
		if (response.error) throw new Error(response.error.message);
	},

	async rotateApiKey(id: string): Promise<ApiKeyResponse> {
		const response = await apiClient.post<ApiKeyResponse>(`/api/v1/service-users/${id}/rotate`, {});
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	}
};
