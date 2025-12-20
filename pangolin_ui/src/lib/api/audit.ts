import { apiClient } from './client';

export interface AuditLogEntry {
    id: string;
    tenant_id: string;
    user_id: string;
    username: string;
    action: string;
    resource_type: string;
    resource_id: string | null;
    resource_name: string | null;
    timestamp: string;
    result: 'success' | 'failure';
    error_message: string | null;
    ip_address: string | null;
    user_agent: string | null;
    details: any;
}

export interface AuditListQuery {
    user_id?: string;
    action?: string;
    resource_type?: string;
    resource_id?: string;
    start_time?: string;
    end_time?: string;
    result?: 'success' | 'failure';
    limit?: number;
    offset?: number;
}

export const auditApi = {
    async list(query: AuditListQuery = {}): Promise<AuditLogEntry[]> {
        const params = new URLSearchParams();
        Object.entries(query).forEach(([key, value]) => {
            if (value !== undefined && value !== null) {
                params.append(key, value.toString());
            }
        });
        
        const queryString = params.toString();
        const url = `/api/v1/audit${queryString ? `?${queryString}` : ''}`;
        
        const response = await apiClient.get<AuditLogEntry[]>(url);
        if (response.error) throw new Error(response.error.message);
        return response.data || [];
    },

    async get(id: string): Promise<AuditLogEntry> {
        const response = await apiClient.get<AuditLogEntry>(`/api/v1/audit/${id}`);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async count(query: AuditListQuery = {}): Promise<number> {
        const params = new URLSearchParams();
        Object.entries(query).forEach(([key, value]) => {
            if (value !== undefined && value !== null) {
                params.append(key, value.toString());
            }
        });
        
        const queryString = params.toString();
        const url = `/api/v1/audit/count${queryString ? `?${queryString}` : ''}`;
        
        const response = await apiClient.get<{ count: number }>(url);
        if (response.error) throw new Error(response.error.message);
        return response.data?.count || 0;
    }
};
