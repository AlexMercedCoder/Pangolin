import { apiClient } from './client';

export interface AccessRequest {
	id: string;
	user_id: string;
	user_name?: string;
	asset_id: string;
	asset_name?: string;
	asset_fqn?: string;
	reason: string;
	status: 'pending' | 'approved' | 'rejected';
	requested_at: string;
	reviewed_at?: string;
	reviewed_by?: string;
}

export interface UpdateAccessRequestRequest {
	status: 'approved' | 'rejected';
}

export const accessRequestsApi = {
	async list(): Promise<AccessRequest[]> {
		const response = await apiClient.get<AccessRequest[]>('/api/v1/access-requests');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(id: string): Promise<AccessRequest> {
		const response = await apiClient.get<AccessRequest>(`/api/v1/access-requests/${id}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async update(id: string, data: UpdateAccessRequestRequest): Promise<AccessRequest> {
		const response = await apiClient.put<AccessRequest>(`/api/v1/access-requests/${id}`, data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},
};
