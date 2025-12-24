import { apiClient } from './client';

export interface BusinessMetadata {
    asset_id: string; // UUID
    resource_type: string;
    description?: string;
    owner_id: string; // UUID
    tags: string[];
    properties: Record<string, string>;
    discoverable: boolean;
    updated_at: string;
    updated_by: string;
}


export interface AccessRequest {
    id: string; // UUID
    tenant_id?: string; // UUID (optional as it might effectively be on some responses)
    user_id: string; // UUID
    asset_id: string; // UUID
    reason?: string;
    requested_at: string;
    status: 'pending' | 'approved' | 'rejected';
    reviewed_by?: string; // UUID
    reviewed_at?: string;
    review_comment?: string;
}

export interface CreateAccessRequestPayload {
    reason?: string;
}

export interface UpdateRequestStatus {
    status: 'approved' | 'rejected';
    comment?: string;
}

export interface AddMetadataRequest {
    description?: string;
    tags?: string[];
    properties?: Record<string, string>;
    discoverable?: boolean;
}

export const businessMetadataApi = {
    // Metadata Operations
    addMetadata: async (assetId: string, metadata: AddMetadataRequest): Promise<void> => {
        const response = await apiClient.post<void>(`/api/v1/assets/${assetId}/metadata`, metadata);
        if (response.error) throw new Error(response.error.message);
    },

    getMetadata: async (assetId: string): Promise<BusinessMetadata> => {
        const response = await apiClient.get<BusinessMetadata>(`/api/v1/assets/${assetId}/metadata`);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    // Access Request Operations
    requestAccess: async (assetId: string, payload: CreateAccessRequestPayload): Promise<AccessRequest> => {
        const response = await apiClient.post<AccessRequest>(`/api/v1/assets/${assetId}/access-requests`, payload);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    listRequests: async (): Promise<AccessRequest[]> => {
        const response = await apiClient.get<any[]>(`/api/v1/access-requests`);
        if (response.error) throw new Error(response.error.message);
        return (response.data || []).map(r => ({
            id: r.id,
            user_id: r['user-id'],
            asset_id: r['asset-id'],
            tenant_id: r['tenant-id'],
            reason: r.reason,
            requested_at: r['requested-at'],
            status: r.status,
            reviewed_by: r['reviewed-by'],
            reviewed_at: r['reviewed-at'],
            review_comment: r['review-comment']
        }));
    },

    getRequest: async (requestId: string): Promise<AccessRequest> => {
        const response = await apiClient.get<any>(`/api/v1/access-requests/${requestId}`);
        if (response.error) throw new Error(response.error.message);
        const r = response.data!;
        return {
            id: r.id,
            user_id: r['user-id'],
            asset_id: r['asset-id'],
            tenant_id: r['tenant-id'],
            reason: r.reason,
            requested_at: r['requested-at'],
            status: r.status,
            reviewed_by: r['reviewed-by'],
            reviewed_at: r['reviewed-at'],
            review_comment: r['review-comment']
        };
    },

    updateRequestStatus: async (requestId: string, payload: UpdateRequestStatus): Promise<void> => {
        const response = await apiClient.put<void>(`/api/v1/access-requests/${requestId}`, payload);
        if (response.error) throw new Error(response.error.message);
    },
    
    // Search
    searchAssets: async (query: string): Promise<any[]> => {
        const response = await apiClient.get<any[]>(`/api/v1/assets/search?query=${encodeURIComponent(query)}`);
        if (response.error) throw new Error(response.error.message);
        return response.data || [];
    }
};
