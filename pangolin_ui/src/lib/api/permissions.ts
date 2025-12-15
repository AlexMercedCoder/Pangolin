import { apiClient } from './client';

export type PermissionScope = 
	| { Global: null }
	| { Catalog: string }
	| { Namespace: { catalog: string; namespace: string[] } }
	| { Table: { catalog: string; namespace: string[]; table: string } }
	| { Tag: string }; // ABAC/TBAC: Tag-based access control

export type Action = 'Read' | 'Write' | 'Delete' | 'Admin' | 'Create' | 'Update' | 'List' | 'All' | 'IngestBranching' | 'ExperimentalBranching';

export interface Permission {
	id: string;
	user_id: string;
	scope: PermissionScope;
	actions: Action[];
	granted_by: string;
	created_at: string;
}

export interface GrantPermissionRequest {
	user_id: string;
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
	if ('Global' in scope) return 'Global';
	if ('Catalog' in scope) return `Catalog: ${scope.Catalog}`;
	if ('Namespace' in scope) {
		const ns = scope.Namespace;
		return `Namespace: ${ns.catalog}.${ns.namespace.join('.')}`;
	}
	if ('Table' in scope) {
		const t = scope.Table;
		return `Table: ${t.catalog}.${t.namespace.join('.')}.${t.table}`;
	}
	if ('Tag' in scope) return `Tag: ${scope.Tag}`;
	return 'Unknown';
}

export function getScopeType(scope: PermissionScope): 'Global' | 'Catalog' | 'Namespace' | 'Table' | 'Tag' {
	if ('Global' in scope) return 'Global';
	if ('Catalog' in scope) return 'Catalog';
	if ('Namespace' in scope) return 'Namespace';
	if ('Table' in scope) return 'Table';
	if ('Tag' in scope) return 'Tag';
	return 'Global';
}
