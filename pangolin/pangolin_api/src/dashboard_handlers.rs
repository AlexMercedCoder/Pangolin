use axum::{
    extract::{State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use pangolin_store::CatalogStore;
use pangolin_core::user::{UserSession, UserRole};
use crate::iceberg::AppState;
use crate::error::ApiError;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct DashboardStats {
    pub catalogs_count: usize,
    pub tables_count: usize,
    pub namespaces_count: usize,
    pub users_count: usize,
    pub warehouses_count: usize,
    pub branches_count: usize,
    pub tenants_count: usize,
    /// Scope of statistics: "system", "tenant", or "user"
    pub scope: String,
}

#[derive(Serialize, ToSchema)]
pub struct CatalogSummary {
    pub name: String,
    pub table_count: usize,
    pub namespace_count: usize,
    pub branch_count: usize,
    pub storage_location: Option<String>,
}

/// Get dashboard statistics based on user role
/// 
/// Returns different scopes based on user role:
/// - Root: system-wide statistics
/// - Tenant Admin: tenant-wide statistics
/// - Tenant User: statistics for accessible resources
#[utoipa::path(
    get,
    path = "/api/v1/dashboard/stats",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Dashboard statistics", body = DashboardStats),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_stats(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
) -> Result<impl IntoResponse, ApiError> {
    match session.role {
        UserRole::Root => {
            // Aggregate statistics across ALL tenants
            let tenants = store.list_tenants(None).await
                .map_err(ApiError::from)?;
            let tenants_count = tenants.len();
            
            let mut catalogs_count = 0;
            let mut warehouses_count = 0;
            
            for tenant in &tenants {
                catalogs_count += store.list_catalogs(tenant.id, None).await
                    .map_err(ApiError::from)?.len();
                warehouses_count += store.list_warehouses(tenant.id, None).await
                    .map_err(ApiError::from)?.len();
            }
            
            let stats = DashboardStats {
                catalogs_count,
                warehouses_count,
                tenants_count,
                tables_count: 0,  // Expensive to compute across all tenants
                namespaces_count: 0,  // Expensive to compute across all tenants
                users_count: 0,  // Expensive to compute across all tenants
                branches_count: 0,  // Expensive to compute across all tenants
                scope: "system".to_string(),
            };
            
            Ok((StatusCode::OK, Json(stats)))
        },

        UserRole::TenantAdmin => {
            let tenant_id = session.tenant_id.unwrap_or_default();
            
            let catalogs = store.list_catalogs(tenant_id, None).await
                .map_err(ApiError::from)?;
            let warehouses = store.list_warehouses(tenant_id, None).await
                .map_err(ApiError::from)?;
            
            // Use efficient count methods
            let namespaces_count = store.count_namespaces(tenant_id).await.unwrap_or(0);
            let tables_count = store.count_assets(tenant_id).await.unwrap_or(0);

            let stats = DashboardStats {
                catalogs_count: catalogs.len(),
                tables_count,
                namespaces_count,
                users_count: 0, 
                warehouses_count: warehouses.len(),
                branches_count: 0,
                tenants_count: 0,  // Not applicable for tenant scope
                scope: "tenant".to_string(),
            };
            
            Ok((StatusCode::OK, Json(stats)))
        },
        UserRole::TenantUser => {
            let tenant_id = session.tenant_id.unwrap_or_default();
            
            // Fetch user permissions for filtering
            let permissions = store.list_user_permissions(session.user_id, None).await
                .map_err(ApiError::from)?;
            
            // Get all catalogs and filter by permissions
            let all_catalogs = store.list_catalogs(tenant_id, None).await
                .map_err(ApiError::from)?;
            let accessible_catalogs = crate::authz_utils::filter_catalogs(
                all_catalogs,
                &permissions,
                session.role.clone()
            );
            
            // Get all warehouses and filter by permissions
            // Note: Warehouses don't have direct permission scopes, so we show warehouses
            // associated with accessible catalogs
            let all_warehouses = store.list_warehouses(tenant_id, None).await
                .map_err(ApiError::from)?;
            let accessible_warehouse_names: std::collections::HashSet<_> = accessible_catalogs
                .iter()
                .filter_map(|c| c.warehouse_name.clone())
                .collect();
            let accessible_warehouses_count = all_warehouses
                .iter()
                .filter(|w| accessible_warehouse_names.contains(&w.name))
                .count();
            
            // Count accessible namespaces
            // Fetch all namespaces and filter by permissions
            let catalog_id_map: std::collections::HashMap<_, _> = accessible_catalogs
                .iter()
                .map(|c| (c.name.clone(), c.id))
                .collect();
            
            let mut accessible_namespaces_count = 0;
            for catalog in &accessible_catalogs {
                if let Ok(namespaces) = store.list_namespaces(tenant_id, &catalog.name, None, None).await {
                    let namespace_tuples: Vec<_> = namespaces
                        .into_iter()
                        .map(|ns| (ns, catalog.name.clone()))
                        .collect();
                    
                    let filtered = crate::authz_utils::filter_namespaces(
                        namespace_tuples,
                        &permissions,
                        session.role.clone(),
                        &catalog_id_map
                    );
                    accessible_namespaces_count += filtered.len();
                }
            }
            
            // Count accessible tables/assets
            // This is more expensive but necessary for accurate counts
            let mut accessible_tables_count = 0;
            if let Ok(all_assets) = store.search_assets(tenant_id, "", None).await {
                let filtered_assets = crate::authz_utils::filter_assets(
                    all_assets,
                    &permissions,
                    session.role.clone(),
                    &catalog_id_map
                );
                accessible_tables_count = filtered_assets.len();
            }

            let stats = DashboardStats {
                catalogs_count: accessible_catalogs.len(),
                tables_count: accessible_tables_count,
                namespaces_count: accessible_namespaces_count,
                users_count: 0,
                warehouses_count: accessible_warehouses_count,
                branches_count: 0,
                tenants_count: 0,  // Not applicable for user scope
                scope: "user".to_string(),
            };
            
            Ok((StatusCode::OK, Json(stats)))
        },
    }
}

/// Get summary statistics for a specific catalog
#[utoipa::path(
    get,
    path = "/api/v1/catalogs/{name}/summary",
    tag = "Catalogs",
    params(
        ("name" = String, Path, description = "Catalog name")
    ),
    responses(
        (status = 200, description = "Catalog summary", body = CatalogSummary),
        (status = 404, description = "Catalog not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_catalog_summary(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = session.tenant_id.unwrap_or_default();
    
    let catalog = store.get_catalog(tenant_id, name.clone()).await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("Catalog not found"))?;
    
    let branches = store.list_branches(tenant_id, &name, None).await
        .map_err(ApiError::from)?;
    
    let namespaces = store.list_namespaces(tenant_id, &name, None, None).await
        .map_err(ApiError::from)?;
    
    let summary = CatalogSummary {
        name: catalog.name,
        table_count: 0,
        namespace_count: namespaces.len(),
        branch_count: branches.len(),
        storage_location: catalog.storage_location,
    };
    
    Ok((StatusCode::OK, Json(summary)))
}
