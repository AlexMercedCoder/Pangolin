import { apiClient } from './client';

export interface Namespace {
    namespace: string[];
    properties?: Record<string, string>;
}

export interface TableIdentifier {
    namespace: string[];
    name: string;
}

export interface Table {
    identifier: TableIdentifier;
    // We can expand this with full schema/snapshot types as needed
    schemas?: any[];
    schema?: any;
    current_snapshot_id?: number;
    snapshots?: any[];
    history?: any[];
    properties?: Record<string, string>;
    location?: string;
    "format-version"?: number;
}

export interface Asset extends Table {
    // Asset can have more metadata if needed, but for now it's compatible with Table
    description?: string;
    tags?: string[];
}

export interface ListNamespacesResponse {
    namespaces: string[][];
}

export interface ListTablesResponse {
    identifiers: TableIdentifier[];
}

export interface CreateNamespaceRequest {
    namespace: string[];
    properties?: Record<string, string>;
}

export interface CreateTableRequest {
    name: string;
    schema: {
        type: "struct";
        fields: {
            id: number;
            name: string;
            required: boolean;
            type: string; // "string", "int", etc.
        }[];
    };
    location?: string;
    properties?: Record<string, string>;
}

export const icebergApi = {
    async listNamespaces(catalogName: string, parent?: string[]): Promise<string[][]> {
        const query = parent ? `?parent=${parent.join('.')}` : ''; // Simplified parent handling, usually passed as query param if supported or encoded in url
        // Pangolin/Iceberg REST maps namespaces strictly. 
        // Standard Iceberg REST: GET /v1/namespaces
        const baseUrl = `/api/v1/catalogs/${encodeURIComponent(catalogName)}/iceberg`;
        
        const response = await apiClient.get<ListNamespacesResponse>(`${baseUrl}/v1/namespaces${query}`);
        if (response.error) throw new Error(response.error.message);
        return response.data?.namespaces || [];
    },

    async getNamespace(catalogName: string, namespace: string[]): Promise<Namespace> {
        const baseUrl = `/api/v1/catalogs/${encodeURIComponent(catalogName)}/iceberg`;
        const nsStr = namespace.join('\x1F'); // Iceberg REST spec often uses specific encoding or joiner.
        // Actually standard REST API usually encodes namespace parts in URL path if simpler, or uses escaped dots.
        // Pangolin implementation likely expects standard REST behavior.
        // Let's assume joining by encoded dot or verify based on typical python client behavior.
        // PyIceberg joins with dot if simple, else multipart.
        // For REST, it's usually /v1/namespaces/{namespace}
        // Let's proceed with standard dot join for now, safe for simple names.
        
        const response = await apiClient.get<Namespace>(`${baseUrl}/v1/namespaces/${namespace.join('.')}`);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async listTables(catalogName: string, namespace: string[]): Promise<TableIdentifier[]> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        const response = await apiClient.get<ListTablesResponse>(`${baseUrl}/v1/namespaces/${nsPath}/tables`);
        if (response.error) throw new Error(response.error.message);
        return response.data!.identifiers;
    },

    async loadTable(catalogName: string, namespace: string[], table: string): Promise<Table> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        const response = await apiClient.get<Table>(`${baseUrl}/v1/namespaces/${nsPath}/tables/${table}`);
        if (response.error) throw new Error(response.error.message);
        
        const result = response.data!;
        if (!result.identifier) {
            result.identifier = { namespace, name: table };
        }
        return result;
    },

    async createNamespace(catalogName: string, request: CreateNamespaceRequest): Promise<Namespace> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const response = await apiClient.post<Namespace>(`${baseUrl}/v1/namespaces`, request);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async createTable(catalogName: string, namespace: string[], request: CreateTableRequest): Promise<Table> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        const response = await apiClient.post<Table>(`${baseUrl}/v1/namespaces/${nsPath}/tables`, request);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    }
};
