use axum::{
    extract::{State, Extension, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use pangolin_store::CatalogStore;
use pangolin_core::user::UserSession;
use crate::iceberg_handlers::AppState;
use crate::error::ApiError;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Optional catalog name to filter results
    #[serde(default)]
    pub catalog: Option<String>,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Serialize, ToSchema)]
pub struct AssetSearchResult {
    pub id: String,
    pub name: String,
    pub namespace: Vec<String>,
    pub catalog: String,
    pub asset_type: String,
}

#[derive(Serialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<AssetSearchResult>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

/// Search for assets across catalogs
/// 
/// Performs a case-insensitive search across asset names.
/// Supports filtering by catalog and pagination.
#[utoipa::path(
    get,
    path = "/api/v1/search/assets",
    tag = "Search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("catalog" = Option<String>, Query, description = "Filter by catalog name"),
        ("limit" = Option<usize>, Query, description = "Maximum results (default: 50)"),
        ("offset" = Option<usize>, Query, description = "Pagination offset (default: 0)")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn search_assets_by_name(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    if query.q.is_empty() {
        return Err(ApiError::bad_request("Search query cannot be empty"));
    }
    
    // Use the optimized search_assets method from the store
    // This pushes filtering down to the database level (SQL/Mongo)
    let assets = store.search_assets(tenant_id, &query.q, None).await
        .map_err(ApiError::from)?;
    
    // Filter by catalog if specified
    let mut all_results = Vec::new();
    for (asset, _metadata, catalog_name, namespace) in assets {
        if let Some(ref cat_filter) = query.catalog {
            if &catalog_name != cat_filter {
                continue;
            }
        }
        
        all_results.push(AssetSearchResult {
            id: asset.id.to_string(),
            name: asset.name,
            namespace, // already a Vec<String>
            catalog: catalog_name,
            asset_type: format!("{:?}", asset.kind), // Use debug formatter for AssetType enum
        });
    }
    
    let total = all_results.len();
    let results: Vec<_> = all_results
        .into_iter()
        .skip(query.offset)
        .take(query.limit)
        .collect();
    
    Ok((StatusCode::OK, Json(SearchResponse {
        results,
        total,
        limit: query.limit,
        offset: query.offset,
    })))
}

#[derive(Deserialize, ToSchema)]
pub struct BulkDeleteAssetsRequest {
    /// List of asset UUIDs to delete (maximum 100)
    pub asset_ids: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BulkOperationResponse {
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Bulk delete multiple assets
/// 
/// Deletes up to 100 assets in a single request.
/// Returns detailed results including any errors.
#[utoipa::path(
    post,
    path = "/api/v1/bulk/assets/delete",
    tag = "Bulk Operations",
    request_body = BulkDeleteAssetsRequest,
    responses(
        (status = 200, description = "Bulk delete results", body = BulkOperationResponse),
        (status = 400, description = "Bad request (e.g., too many assets)"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn bulk_delete_assets(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<BulkDeleteAssetsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    if payload.asset_ids.len() > 100 {
        return Err(ApiError::bad_request("Maximum 100 assets can be deleted at once"));
    }
    
    let mut succeeded = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for asset_id_str in payload.asset_ids {
        let asset_id = match Uuid::parse_str(&asset_id_str) {
            Ok(id) => id,
            Err(_) => {
                failed += 1;
                errors.push(format!("Invalid UUID: {}", asset_id_str));
                continue;
            }
        };
        
        match store.get_asset_by_id(tenant_id, asset_id).await {
            Ok(Some((asset, catalog_name, namespace))) => {
                match store.delete_asset(
                    tenant_id,
                    &catalog_name,
                    None,
                    namespace,
                    asset.name
                ).await {
                    Ok(_) => succeeded += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("Failed to delete {}: {}", asset_id_str, e));
                    }
                }
            },
            Ok(None) => {
                failed += 1;
                errors.push(format!("Asset not found: {}", asset_id_str));
            },
            Err(e) => {
                failed += 1;
                errors.push(format!("Error fetching asset {}: {}", asset_id_str, e));
            }
        }
    }
    
    Ok((StatusCode::OK, Json(BulkOperationResponse {
        succeeded,
        failed,
        errors,
    })))
}

#[derive(Deserialize, ToSchema)]
pub struct ValidateNamesRequest {
    /// Resource type: "catalog" or "warehouse"
    pub resource_type: String,
    /// List of names to validate
    pub names: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct NameValidationResult {
    pub name: String,
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ValidateNamesResponse {
    pub results: Vec<NameValidationResult>,
}

/// Validate resource name availability
/// 
/// Checks if catalog or warehouse names are available.
/// Supports batch validation of multiple names.
#[utoipa::path(
    post,
    path = "/api/v1/validate/names",
    tag = "Validation",
    request_body = ValidateNamesRequest,
    responses(
        (status = 200, description = "Validation results", body = ValidateNamesResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn validate_names(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<ValidateNamesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    let mut results = Vec::new();
    
    for name in payload.names {
        let (available, reason) = match payload.resource_type.as_str() {
            "catalog" => {
                match store.get_catalog(tenant_id, name.clone()).await {
                    Ok(Some(_)) => (false, Some("Catalog already exists".to_string())),
                    Ok(None) => (true, None),
                    Err(e) => (false, Some(format!("Error checking: {}", e))),
                }
            },
            "warehouse" => {
                match store.get_warehouse(tenant_id, name.clone()).await {
                    Ok(Some(_)) => (false, Some("Warehouse already exists".to_string())),
                    Ok(None) => (true, None),
                    Err(e) => (false, Some(format!("Error checking: {}", e))),
                }
            },
            _ => (false, Some("Unsupported resource type".to_string())),
        };
        
        results.push(NameValidationResult {
            name,
            available,
            reason,
        });
    }
    
    Ok((StatusCode::OK, Json(ValidateNamesResponse { results })))
}
#[derive(Serialize, ToSchema)]
pub enum SearchResultType {
    Asset,
    Catalog,
    Namespace,
    Branch,
}

#[derive(Serialize, ToSchema)]
pub struct UnifiedSearchResult {
    pub id: Option<String>, // Catalogs/Assets have IDs, Namespaces/Branches might not
    pub name: String,
    pub kind: SearchResultType,
    pub description: Option<String>,
    pub context: Option<String>, // e.g., "catalog.namespace" or "catalog table"
}

#[derive(Serialize, ToSchema)]
pub struct UnifiedSearchResponse {
    pub results: Vec<UnifiedSearchResult>,
}

#[derive(Deserialize, ToSchema)]
pub struct UnifiedSearchQuery {
    pub q: String,
    #[serde(default)]
    pub limit: usize,
}

/// Unified search across all resources
/// 
/// Searches Assets, Catalogs, Namespaces, and Branches.
#[utoipa::path(
    get,
    path = "/api/v1/search",
    tag = "Search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<usize>, Query, description = "Max results (default 20)")
    ),
    responses(
        (status = 200, description = "Search results", body = UnifiedSearchResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn unified_search(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Query(query): Query<UnifiedSearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = session.tenant_id.unwrap_or_default();
    let limit = if query.limit == 0 { 20 } else { query.limit };
    
    let mut results = Vec::new();

    // 1. Search Catalogs
    let catalogs = store.search_catalogs(tenant_id, &query.q).await.map_err(ApiError::from)?;
    for c in catalogs {
        results.push(UnifiedSearchResult {
            id: Some(c.id.to_string()),
            name: c.name,
            kind: SearchResultType::Catalog,
            description: None, // Catalog doesn't have description yet
            context: None,
        });
    }

    // 2. Search Namespaces
    let namespaces = store.search_namespaces(tenant_id, &query.q).await.map_err(ApiError::from)?;
    for (ns, cat_name) in namespaces {
        results.push(UnifiedSearchResult {
            id: None,
            name: ns.name.join("."),
            kind: SearchResultType::Namespace,
            description: None,
            context: Some(format!("Catalog: {}", cat_name)),
        });
    }

    // 3. Search Assets
    let assets = store.search_assets(tenant_id, &query.q, None).await.map_err(ApiError::from)?;
    for (asset, metadata, cat_name, ns) in assets {
        results.push(UnifiedSearchResult {
            id: Some(asset.id.to_string()),
            name: asset.name,
            kind: SearchResultType::Asset,
            description: metadata.and_then(|m| m.description),
            context: Some(format!("{}.{}", cat_name, ns.join("."))),
        });
    }

    // 4. Search Branches
    let branches = store.search_branches(tenant_id, &query.q).await.map_err(ApiError::from)?;
    for (branch, cat_name) in branches {
        results.push(UnifiedSearchResult {
            id: None,
            name: branch.name,
            kind: SearchResultType::Branch,
            description: None,
            context: Some(format!("Catalog: {}", cat_name)),
        });
    }

    // Sort by name length (simple relevance) and truncate
    results.sort_by_key(|r| r.name.len());
    results.truncate(limit);

    Ok((StatusCode::OK, Json(UnifiedSearchResponse { results })))
}
