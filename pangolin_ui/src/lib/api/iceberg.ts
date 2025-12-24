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
    "table-uuid"?: string;
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
        const query = parent ? `?parent=${parent.join('.')}` : ''; 
        // Correct route per lib.rs: /v1/:prefix/namespaces
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        
        // The backend defines /v1/:prefix/namespaces, so we append /namespaces
        const url = `${baseUrl}/namespaces${query}`;
        console.log('iceberg.listNamespaces fetching:', url);
        const response = await apiClient.get<ListNamespacesResponse>(url);
        if (response.error) throw new Error(response.error.message);
        return response.data?.namespaces || [];
    },

    async getNamespace(catalogName: string, namespace: string[]): Promise<Namespace> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        // Note: Client usually joins with x1F, but backend often supports dots if standard
        // lib.rs route: /v1/:prefix/namespaces/:namespace
        const response = await apiClient.get<Namespace>(`${baseUrl}/namespaces/${namespace.join('.')}`);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async listTables(catalogName: string, namespace: string[]): Promise<TableIdentifier[]> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        // lib.rs: /v1/:prefix/namespaces/:namespace/tables
        const response = await apiClient.get<ListTablesResponse>(`${baseUrl}/namespaces/${nsPath}/tables`);
        if (response.error) throw new Error(response.error.message);
        return response.data!.identifiers;
    },

    async loadTable(catalogName: string, namespace: string[], table: string): Promise<Table> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        // lib.rs: /v1/:prefix/namespaces/:namespace/tables/:table
        const response = await apiClient.get<Table>(`${baseUrl}/namespaces/${nsPath}/tables/${table}`);
        if (response.error) throw new Error(response.error.message);
        
        const data = response.data as any;
        
        // Unwrap metadata if it's wrapped (standard Iceberg REST Response)
        let result = data;
        if (data.metadata) {
            result = data.metadata;
            // unexpected but useful to keep metadata-location available if needed, usually it's in config or top level
        }

        if (!result.identifier) {
            result.identifier = { namespace, name: table };
        }
        return result;
    },

    async createNamespace(catalogName: string, request: CreateNamespaceRequest): Promise<Namespace> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const response = await apiClient.post<Namespace>(`${baseUrl}/namespaces`, request);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async createTable(catalogName: string, namespace: string[], request: CreateTableRequest): Promise<Table> {
        const baseUrl = `/v1/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        const response = await apiClient.post<Table>(`${baseUrl}/namespaces/${nsPath}/tables`, request);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    async listAssets(catalogName: string, namespace: string[]): Promise<AssetSummary[]> {
        const baseUrl = `/api/v1/catalogs/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        const response = await apiClient.get<AssetSummary[]>(`${baseUrl}/namespaces/${nsPath}/assets`);
        if (response.error) throw new Error(response.error.message);
        return response.data || [];
    },

    async getAsset(catalogName: string, namespace: string[], asset: string): Promise<Asset> {
        const baseUrl = `/api/v1/catalogs/${encodeURIComponent(catalogName)}`;
        const nsPath = namespace.join('.');
        const response = await apiClient.get<any>(`${baseUrl}/namespaces/${nsPath}/assets/${asset}`);
        if (response.error) throw new Error(response.error.message);
        
        const data = response.data;
        return {
             identifier: { namespace, name: data.name },
             name: data.name,
             "table-uuid": data.id, 
             // @ts-ignore
             kind: data.kind,
             location: data.location,
             properties: data.properties,
             schemas: [],
             snapshots: [],
             history: [],
             "format-version": 1
        } as Asset;
    }
};

export interface AssetSummary {
    id: string;
    name: string;
    namespace: string[];
    kind: string; 
    identifier: TableIdentifier;
}
