import { apiClient } from './client';

export interface Role {
	id: string;
	name: string;
	description?: string;
	tenant_id: string;
	created_by: string;
	created_at: string;
	permissions: any[]; // Or specific permission type if available
}

// Backend expects strict Update structure usually merging with existing? 
// No, backend update_role expects the Full Role object.
// So we should expose a way to pass the role.
// But for type safety let's define Role.
export interface CreateRoleRequest {
	name: string;
	description?: string;
	'tenant-id': string; // Required (kebab-case for backend)
}

// Ensure update accepts Role
export const rolesApi = {
    // ... list, get ...
	async list(): Promise<Role[]> {
		const response = await apiClient.get<Role[]>('/api/v1/roles');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},
    
	async get(id: string): Promise<Role> {
		const response = await apiClient.get<Role>(`/api/v1/roles/${id}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateRoleRequest): Promise<Role> {
		const response = await apiClient.post<Role>('/api/v1/roles', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(id: string, data: Role): Promise<void> {
		const response = await apiClient.put<void>(`/api/v1/roles/${id}`, data);
		if (response.error) throw new Error(response.error.message);
	},
    // ... delete, assign ...

	async delete(id: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/roles/${id}`);
		if (response.error) throw new Error(response.error.message);
	},

	async assignToUser(userId: string, roleId: string): Promise<void> {
		const response = await apiClient.post<void>(`/api/v1/users/${userId}/roles`, { 'role-id': roleId });
		if (response.error) throw new Error(response.error.message);
	},

	async revokeFromUser(userId: string, roleId: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/users/${userId}/roles/${roleId}`);
		if (response.error) throw new Error(response.error.message);
	},

    async getUserRoles(userId: string): Promise<Role[]> {
        const response = await apiClient.get<Role[]>(`/api/v1/users/${userId}/roles`);
        if (response.error) throw new Error(response.error.message);
        return response.data || [];
    }
};
