import { apiClient } from './client';

export interface User {
	id: string;
	username: string;
	email: string;
	role: 'Root' | 'TenantAdmin' | 'TenantUser';
	tenant_id?: string;
	tenant_name?: string;
	created_at: string;
	last_login?: string;
}

export interface CreateUserRequest {
	username: string;
	email: string;
	password: string;
	role: string;
	tenant_id?: string;
}

export interface UpdateUserRequest {
	email?: string;
	role?: string;
	password?: string;
}

export const usersApi = {
	async list(): Promise<User[]> {
		const response = await apiClient.get<User[]>('/api/v1/users');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(id: string): Promise<User> {
		const response = await apiClient.get<User>(`/api/v1/users/${id}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateUserRequest): Promise<User> {
		const response = await apiClient.post<User>('/api/v1/users', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(id: string, data: UpdateUserRequest): Promise<User> {
		const response = await apiClient.put<User>(`/api/v1/users/${id}`, data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async delete(id: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/users/${id}`);
		if (response.error) throw new Error(response.error.message);
	},
};
