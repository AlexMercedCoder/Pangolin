export interface DashboardStats {
  catalogs_count: number;
  tables_count: number;
  namespaces_count: number;
  users_count: number;
  warehouses_count: number;
  branches_count: number;
  tenants_count?: number;  // Only for Root users
  scope: string;
}

export interface CatalogSummary {
  name: string;
  table_count: number;
  namespace_count: number;
  branch_count: number;
  storage_location: string | null;
}

export interface SearchResponse {
  results: AssetSearchResult[];
  total: number;
  limit: number;
  offset: number;
}

export interface AssetSearchResult {
  id: string;
  name: string;
  namespace: string[];
  catalog: string;
  asset_type: string;
  has_access: boolean;
}

export interface BulkOperationResponse {
  succeeded: number;
  failed: number;
  errors: string[];
}

export interface ValidateNamesResponse {
  results: NameValidationResult[];
}

export interface NameValidationResult {
  name: string;
  available: boolean;
  reason: string | null;
}
