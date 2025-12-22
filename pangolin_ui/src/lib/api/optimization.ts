import { apiClient } from './client';
import type { 
  DashboardStats, 
  SearchResponse, 
  BulkOperationResponse, 
  ValidateNamesResponse 
} from '$lib/types/optimization';

export const dashboardApi = {
  async getStats(): Promise<DashboardStats> {
    const response = await apiClient.get<DashboardStats>('/api/v1/dashboard/stats');
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  }
};

export const searchApi = {
  async searchAssets(params: {
    q: string;
    catalog?: string;
    limit?: number;
    offset?: number;
  }): Promise<SearchResponse> {
    const searchParams = new URLSearchParams();
    searchParams.append('q', params.q);
    if (params.catalog) searchParams.append('catalog', params.catalog);
    if (params.limit) searchParams.append('limit', params.limit.toString());
    if (params.offset) searchParams.append('offset', params.offset.toString());
    
    const response = await apiClient.get<SearchResponse>(`/api/v1/search/assets?${searchParams.toString()}`);
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  }
};

export const optimizationApi = {
  async bulkDeleteAssets(assetIds: string[]): Promise<BulkOperationResponse> {
    const response = await apiClient.post<BulkOperationResponse>(
      '/api/v1/bulk/assets/delete',
      { asset_ids: assetIds }
    );
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  },

  async validateNames(request: {
    resource_type: 'catalog' | 'warehouse';
    names: string[];
  }): Promise<ValidateNamesResponse> {
    const response = await apiClient.post<ValidateNamesResponse>(
      '/api/v1/validate/names',
      request
    );
    if (response.error) throw new Error(response.error.message);
    return response.data!;
  }
};
