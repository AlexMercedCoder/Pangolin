import { apiClient } from './client';

export type PermissionScope = 
	| { type: 'tenant' }
	| { type: 'catalog'; catalog_id: string }
	| { type: 'namespace'; catalog_id: string; namespace: string }
	| { type: 'asset'; catalog_id: string; namespace: string; asset_id: string }
	| { type: 'tag'; tag_name: string };

export type Action = 'read' | 'write' | 'delete' | 'create' | 'update' | 'list' | 'all' | 'ingest-branching' | 'experimental-branching' | 'manage-discovery';

export interface Permission {
	id: string;
	user_id: string;
	scope: PermissionScope;
	actions: Action[];
	granted_by: string;
	created_at: string;
}

export interface GrantPermissionRequest {
	'user-id': string;
	scope: PermissionScope;
	actions: Action[];
}

export const permissionsApi = {
	async grant(data: GrantPermissionRequest): Promise<Permission> {
		const response = await apiClient.post<Permission>('/api/v1/permissions', data);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async revoke(id: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/permissions/${id}`);
		if (response.error) throw new Error(response.error.message);
	},

	async getUserPermissions(userId: string): Promise<Permission[]> {
		const response = await apiClient.get<Permission[]>(`/api/v1/users/${userId}/permissions`);
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},
};

// Helper functions for permission scope display
export function getScopeDisplay(scope: PermissionScope): string {
	if (scope.type === 'tenant') return 'Tenant';
	if (scope.type === 'catalog') return `Catalog: ${scope.catalog_id.slice(0, 8)}...`;
	if (scope.type === 'namespace') {
		return `Namespace: ${scope.catalog_id.slice(0, 8)}.../${scope.namespace}`;
	}
	if (scope.type === 'asset') {
		return `Asset: ${scope.catalog_id.slice(0, 8)}.../${scope.namespace}/${scope.asset_id.slice(0, 8)}...`;
	}
	if (scope.type === 'tag') return `Tag: ${scope.tag_name}`;
	return 'Unknown';
}

export function getScopeType(scope: PermissionScope): 'tenant' | 'catalog' | 'namespace' | 'asset' | 'tag' {
	return scope.type;
}
