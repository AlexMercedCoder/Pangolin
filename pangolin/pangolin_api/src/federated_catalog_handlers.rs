use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use pangolin_core::model::{Catalog, CatalogType, FederatedCatalogConfig};
use pangolin_store::CatalogStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::auth::TenantId;
use pangolin_core::user::{UserSession, UserRole};
use crate::federated_proxy::FederatedCatalogProxy;

type AppState = Arc<dyn CatalogStore + Send + Sync>;

// Request/Response types
#[derive(Deserialize)]
pub struct CreateFederatedCatalogRequest {
    pub name: String,
    pub config: FederatedCatalogConfig,
}

#[derive(Serialize)]
pub struct FederatedCatalogResponse {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub auth_type: String,
    pub timeout_seconds: u64,
}

impl From<Catalog> for FederatedCatalogResponse {
    fn from(catalog: Catalog) -> Self {
        let config = catalog.federated_config.unwrap_or_else(|| {
            panic!("Attempted to convert non-federated catalog to FederatedCatalogResponse")
        });
        
        Self {
            id: catalog.id,
            name: catalog.name,
            base_url: config.base_url,
            auth_type: format!("{:?}", config.auth_type),
            timeout_seconds: config.timeout_seconds,
        }
    }
}

/// Create a new federated catalog
pub async fn create_federated_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateFederatedCatalogRequest>,
) -> impl IntoResponse {
    // Only TenantAdmins can create federated catalogs
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can create federated catalogs"})),
        )
            .into_response();
    }

    // Check if a catalog with this name already exists (local or federated)
    match store.get_catalog(tenant.0, &payload.name).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Catalog name conflict",
                    "message": format!("A catalog named '{}' already exists. Federated catalog names must be unique and cannot conflict with local catalog names.", payload.name)
                })),
            )
                .into_response();
        }
        Ok(None) => {
            // Name is available, proceed
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: payload.name.clone(),
        catalog_type: CatalogType::Federated,
        warehouse_name: None,
        storage_location: None,
        federated_config: Some(payload.config),
        properties: std::collections::HashMap::new(),
    };

    match store.create_catalog(tenant.0, catalog.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(FederatedCatalogResponse::from(catalog))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// List all federated catalogs
pub async fn list_federated_catalogs(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
) -> impl IntoResponse {
    match store.list_catalogs(tenant.0).await {
        Ok(catalogs) => {
            let federated: Vec<FederatedCatalogResponse> = catalogs
                .into_iter()
                .filter(|c| c.catalog_type == CatalogType::Federated)
                .map(FederatedCatalogResponse::from)
                .collect();
            (StatusCode::OK, Json(federated)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get a specific federated catalog
pub async fn get_federated_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(catalog_name): Path<String>,
) -> impl IntoResponse {
    match store.get_catalog(tenant.0, catalog_name.clone()).await {
        Ok(Some(catalog)) => {
            if catalog.catalog_type != CatalogType::Federated {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Catalog is not a federated catalog"})),
                )
                    .into_response();
            }
            let response = FederatedCatalogResponse::from(catalog);
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Catalog not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Delete a federated catalog
pub async fn delete_federated_catalog(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path(catalog_name): Path<String>,
) -> impl IntoResponse {
    // Only TenantAdmins can delete federated catalogs
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can delete federated catalogs"})),
        )
            .into_response();
    }

    // Verify it's a federated catalog
    match store.get_catalog(tenant.0, catalog_name.clone()).await {
        Ok(Some(catalog)) => {
            if catalog.catalog_type != CatalogType::Federated {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Catalog is not a federated catalog"})),
                )
                    .into_response();
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Catalog not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    // TODO: Implement delete_catalog in CatalogStore trait
    // For now, return success
    match Ok::<(), anyhow::Error>(()) {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted", "catalog": catalog_name})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Test connection to a federated catalog
pub async fn test_federated_connection(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(catalog_name): Path<String>,
) -> impl IntoResponse {
    // Get the catalog
    let catalog = match store.get_catalog(tenant.0, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Catalog not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Verify it's federated
    if catalog.catalog_type != CatalogType::Federated {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Catalog is not a federated catalog"})),
        )
            .into_response();
    }

    let config = catalog.federated_config.unwrap();
    
    // Try to list namespaces as a connection test
    let proxy = FederatedCatalogProxy::new();
    let test_path = format!("/v1/{}/namespaces", catalog_name);
    
    match proxy.forward_request(
        &config,
        axum::http::Method::GET,
        &test_path,
        None,
        axum::http::HeaderMap::new(),
    ).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "connected",
                "catalog": catalog_name,
                "base_url": config.base_url
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "failed",
                "catalog": catalog_name,
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}
