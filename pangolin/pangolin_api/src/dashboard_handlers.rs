use axum::{
    extract::{State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use pangolin_store::CatalogStore;
use pangolin_core::user::{UserSession, UserRole};
use crate::iceberg_handlers::AppState;
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
            let catalogs = store.list_catalogs(session.tenant_id.unwrap_or_default()).await
                .map_err(ApiError::from)?;
            let warehouses = store.list_warehouses(session.tenant_id.unwrap_or_default()).await
                .map_err(ApiError::from)?;
            
            let stats = DashboardStats {
                catalogs_count: catalogs.len(),
                tables_count: 0,
                namespaces_count: 0,
                users_count: 0,
                warehouses_count: warehouses.len(),
                branches_count: 0,
                scope: "system".to_string(),
            };
            
            Ok((StatusCode::OK, Json(stats)))
        },

        UserRole::TenantAdmin => {
            let tenant_id = session.tenant_id.unwrap_or_default();
            
            let catalogs = store.list_catalogs(tenant_id).await
                .map_err(ApiError::from)?;
            let warehouses = store.list_warehouses(tenant_id).await
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
                scope: "tenant".to_string(),
            };
            
            Ok((StatusCode::OK, Json(stats)))
        },
        UserRole::TenantUser => {
            let tenant_id = session.tenant_id.unwrap_or_default();
            
            let catalogs = store.list_catalogs(tenant_id).await
                .map_err(ApiError::from)?;
            
            // For now, users verify all counts in tenant. 
            // TODO: In future, filter by permissions if needed, but count_assets provides O(1) stats.
            let namespaces_count = store.count_namespaces(tenant_id).await.unwrap_or(0);
            let tables_count = store.count_assets(tenant_id).await.unwrap_or(0);

            let stats = DashboardStats {
                catalogs_count: catalogs.len(),
                tables_count,
                namespaces_count,
                users_count: 0,
                warehouses_count: 0,
                branches_count: 0,
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
    
    let branches = store.list_branches(tenant_id, &name).await
        .map_err(ApiError::from)?;
    
    let namespaces = store.list_namespaces(tenant_id, &name, None).await
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
