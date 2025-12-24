use serde::{Deserialize, Serialize};

// Dashboard statistics
#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardStats {
    pub catalogs_count: usize,
    pub tables_count: usize,
    pub namespaces_count: usize,
    pub users_count: usize,
    pub warehouses_count: usize,
    pub branches_count: usize,
    pub tenants_count: Option<usize>,  // New field for Root users
    pub scope: String,
}

// Catalog summary
#[derive(Debug, Deserialize, Serialize)]
pub struct CatalogSummary {
    pub name: String,
    pub table_count: usize,
    pub namespace_count: usize,
    pub branch_count: usize,
    pub storage_location: Option<String>,
}

// Asset search
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub results: Vec<AssetSearchResult>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssetSearchResult {
    pub id: String,
    pub name: String,
    pub namespace: Vec<String>,
    pub catalog: String,
    pub asset_type: String,
}

// Bulk operations
#[derive(Debug, Deserialize, Serialize)]
pub struct BulkOperationResponse {
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

// Name validation
#[derive(Debug, Deserialize, Serialize)]
pub struct ValidateNamesResponse {
    pub results: Vec<NameValidationResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NameValidationResult {
    pub name: String,
    pub available: bool,
    pub reason: Option<String>,
}
