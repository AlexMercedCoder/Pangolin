import { apiClient } from './client';

export interface Branch {
	id: string;
	name: string;
	catalog: string;
	from_branch?: string;
	branch_type: 'experimental' | 'production';
	assets?: string[];
	created_at: string;
}

export interface CreateBranchRequest {
	name: string;
	catalog: string;
	from_branch?: string;
	branch_type: 'experimental' | 'production';
	assets?: string[];
}

export interface MergeBranchRequest {
	catalog: string;
	source_branch: string;
	target_branch: string;
}

export const branchesApi = {
	async list(catalog?: string): Promise<Branch[]> {
		let url = '/api/v1/branches';
		if (catalog) {
			url += `?catalog=${encodeURIComponent(catalog)}`;
		}
		const response = await apiClient.get<Branch[]>(url);
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(catalog: string, name: string): Promise<Branch> {
		// Use query param for catalog as per updated backend
		const response = await apiClient.get<Branch>(`/api/v1/branches/${encodeURIComponent(name)}?catalog=${encodeURIComponent(catalog)}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async create(data: CreateBranchRequest): Promise<Branch> {
		const response = await apiClient.post<Branch>('/api/v1/branches', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async merge(data: MergeBranchRequest): Promise<void> {
		const response = await apiClient.post<void>('/api/v1/branches/merge', data);
		if (response.error) throw new Error(response.error.message);
	},

	async delete(catalog: string, name: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/branches/${encodeURIComponent(catalog)}/${encodeURIComponent(name)}`);
		if (response.error) throw new Error(response.error.message);
	},
};
