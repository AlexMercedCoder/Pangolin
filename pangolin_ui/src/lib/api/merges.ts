import { apiClient } from './client';

export type ResolutionStrategy = 'source' | 'target' | 'manual';
export type MergeStatus = 'pending' | 'analyzing' | 'conflicted' | 'ready' | 'merging' | 'completed' | 'failed' | 'aborted';

export interface MergeOperation {
	id: string;
	status: MergeStatus;
	source_branch: string;
	target_branch: string;
	conflicts_count: number;
	initiated_at: string; // ISO Date
	completed_at?: string; // ISO Date
}

export interface MergeConflict {
	id: string;
	merge_operation_id: string;
	conflict_type: 'schema_change' | 'deletion_conflict' | 'metadata_conflict' | 'data_overlap';
	asset_id?: string;
	asset_name?: string; // Derived or extra field if available
	description: string;
	resolution?: ConflictResolution;
}

export interface ConflictResolution {
	conflict_id: string;
	strategy: ResolutionStrategy;
	resolved_value?: any;
	resolved_by: string;
	resolved_at: string;
}

export interface ResolveConflictRequest {
	strategy: ResolutionStrategy;
	resolved_value?: any;
}

export interface MergeResponse {
	status: string; // "merged" | "conflicted"
	operation_id: string;
	commit_id?: string;
	conflicts?: number;
	message?: string;
}

export const mergesApi = {
	async list(catalog: string): Promise<MergeOperation[]> {
		const response = await apiClient.get<MergeOperation[]>(`/api/v1/catalogs/${encodeURIComponent(catalog)}/merge-operations`);
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async get(id: string): Promise<MergeOperation> {
		const response = await apiClient.get<MergeOperation>(`/api/v1/merge-operations/${id}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async listConflicts(id: string): Promise<MergeConflict[]> {
		const response = await apiClient.get<MergeConflict[]>(`/api/v1/merge-operations/${id}/conflicts`);
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async resolveConflict(conflictId: string, data: ResolveConflictRequest): Promise<void> {
		const response = await apiClient.post<void>(`/api/v1/conflicts/${conflictId}/resolve`, data);
		if (response.error) throw new Error(response.error.message);
	},

	async complete(id: string): Promise<MergeResponse> {
		const response = await apiClient.post<MergeResponse>(`/api/v1/merge-operations/${id}/complete`, {});
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async abort(id: string): Promise<void> {
		const response = await apiClient.post<void>(`/api/v1/merge-operations/${id}/abort`, {});
		if (response.error) throw new Error(response.error.message);
	},
};
