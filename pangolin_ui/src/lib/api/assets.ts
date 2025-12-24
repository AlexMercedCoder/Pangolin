import { apiClient } from './client';

export type AssetType = 
    | 'ICEBERG_TABLE'
    | 'DELTA_TABLE'
    | 'HUDI_TABLE'
    | 'PARQUET_TABLE'
    | 'CSV_TABLE'
    | 'JSON_TABLE'
    | 'VIEW'
    | 'ML_MODEL';

export interface RegisterAssetRequest {
    name: string;
    kind: AssetType;
    location: string;
    properties?: Record<string, string>;
}

export interface RegisterAssetResponse {
    id: string;
    name: string;
    kind: AssetType;
    location: string;
}

export const assetsApi = {
    async register(
        catalogName: string,
        namespace: string,
        data: RegisterAssetRequest
    ): Promise<RegisterAssetResponse> {
        const response = await apiClient.post<RegisterAssetResponse>(
            `/api/v1/catalogs/${encodeURIComponent(catalogName)}/namespaces/${encodeURIComponent(namespace)}/assets`,
            data
        );
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },
};
