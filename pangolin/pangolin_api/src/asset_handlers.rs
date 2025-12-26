use axum::{
    extract::{Path, State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Asset, AssetType};
use uuid::Uuid;
use crate::auth::TenantId;
use crate::authz::check_permission;
use pangolin_core::permission::{PermissionScope, Action};
use pangolin_core::user::UserSession;
use crate::iceberg::AppState;
use crate::iceberg::parse_table_identifier;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateViewRequest {
    pub name: String,
    pub sql: String,
    pub dialect: Option<String>,
    pub properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, ToSchema)]
pub struct ViewResponse {
    pub name: String,
    pub sql: String,
    pub dialect: String,
    pub properties: std::collections::HashMap<String, String>,
}

impl From<Asset> for ViewResponse {
    fn from(asset: Asset) -> Self {
        Self {
            name: asset.name,
            sql: asset.properties.get("sql").cloned().unwrap_or_default(),
            dialect: asset.properties.get("dialect").cloned().unwrap_or_else(|| "ansi".to_string()),
            properties: asset.properties,
        }
    }
}

/// Create a database view
#[utoipa::path(
    post,
    path = "/v1/{prefix}/namespaces/{namespace}/views",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace")
    ),
    request_body = CreateViewRequest,
    responses(
        (status = 201, description = "View created", body = ViewResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_view(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace)): Path<(String, String)>,
    Json(payload): Json<CreateViewRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (view_name, branch_from_name) = parse_table_identifier(&payload.name);
    let branch = branch_from_name.unwrap_or("main".to_string());
    
    // Parse namespace
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    let mut properties = payload.properties.unwrap_or_default();
    properties.insert("sql".to_string(), payload.sql);
    properties.insert("dialect".to_string(), payload.dialect.unwrap_or_else(|| "ansi".to_string()));

    let asset = Asset {
        id: Uuid::new_v4(),
        name: view_name.clone(),
        kind: AssetType::View,
        location: "".to_string(), // Views might not have a physical location, or we store the SQL path?
        properties,
    };

    match store.create_asset(tenant_id, &catalog_name, Some(branch), namespace_parts, asset.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(ViewResponse::from(asset))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

/// Get a database view
#[utoipa::path(
    get,
    path = "/v1/{prefix}/namespaces/{namespace}/views/{view}",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace"),
        ("view" = String, Path, description = "View name")
    ),
    responses(
        (status = 200, description = "View details", body = ViewResponse),
        (status = 404, description = "View not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_view(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((prefix, namespace, view)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    let catalog_name = prefix;
    
    let (view_name, branch_from_name) = parse_table_identifier(&view);
    let branch = branch_from_name.unwrap_or("main".to_string());
    
    let namespace_parts: Vec<String> = namespace.split('\x1F').map(|s| s.to_string()).collect();

    match store.get_asset(tenant_id, &catalog_name, Some(branch), namespace_parts, view_name).await {
        Ok(Some(asset)) => {
            if asset.kind == AssetType::View {
                (StatusCode::OK, Json(ViewResponse::from(asset))).into_response()
            } else {
                (StatusCode::NOT_FOUND, "Asset is not a view").into_response()
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, "View not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterAssetRequest {
    pub name: String,
    pub kind: AssetType,
    pub location: String,
    #[serde(default)]
    pub properties: std::collections::HashMap<String, String>,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterAssetResponse {
    pub id: String,
    pub name: String,
    pub kind: AssetType,
    pub location: String,
}

/// Register a generic (non-Iceberg) asset for discovery and metadata management
#[utoipa::path(
    post,
    path = "/api/v1/catalogs/{catalog_name}/namespaces/{namespace}/assets",
    tag = "Assets",
    params(
        ("catalog_name" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace")
    ),
    request_body = RegisterAssetRequest,
    responses(
        (status = 201, description = "Asset registered", body = RegisterAssetResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn register_asset(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((catalog_name, namespace)): Path<(String, String)>,
    Json(payload): Json<RegisterAssetRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    // Resolve catalog
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            tracing::warn!("Catalog not found: {}", catalog_name);
            return (StatusCode::NOT_FOUND, "Catalog not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get catalog: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
        }
    };
    
    // Check permissions (Create on Namespace)
    let scope = PermissionScope::Namespace { 
        catalog_id: catalog.id, 
        namespace: namespace.clone() 
    };
    match check_permission(&store, &session, &Action::Create, &scope).await {
        Ok(true) => (),
        Ok(false) => {
            tracing::warn!(
                "User {} denied Create permission on namespace {}/{}",
                session.username,
                catalog_name,
                namespace
            );
            return (StatusCode::FORBIDDEN, "Forbidden: Create permission required").into_response();
        }
        Err(e) => {
            tracing::error!("Permission check failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response();
        }
    }
    
    // Generate asset ID
    let asset_id = Uuid::new_v4();
    
    // Parse namespace parts (assuming dot-separated for generic registration)
    let namespace_parts: Vec<String> = namespace.split('.').map(|s| s.to_string()).collect();
    
    tracing::info!(
        "Registering generic asset: id={}, name={}, kind={:?}, catalog={}, namespace={:?}, user={}",
        asset_id,
        payload.name,
        payload.kind,
        catalog_name,
        namespace_parts,
        session.username
    );
    
    let asset = Asset {
        id: asset_id,
        name: payload.name.clone(),
        kind: payload.kind.clone(),
        location: payload.location.clone(),
        properties: payload.properties,
    };
    
    // Use "main" branch by default for generic assets
    // In future we could allow parsing @branch from namespace or name if needed
    let branch = "main".to_string();

    match store.create_asset(tenant_id, &catalog_name, Some(branch), namespace_parts, asset).await {
        Ok(_) => {
            let response = RegisterAssetResponse {
                id: asset_id.to_string(),
                name: payload.name,
                kind: payload.kind,
                location: payload.location,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        },
        Err(e) => {
             tracing::error!("Failed to create asset in store: {}", e);
             (StatusCode::INTERNAL_SERVER_ERROR, "Failed to persist asset").into_response()
        }
    }
}

/// Get a generic asset details
#[utoipa::path(
    get,
    path = "/api/v1/catalogs/{catalog_name}/namespaces/{namespace}/assets/{asset}",
    tag = "Assets",
    params(
        ("catalog_name" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace"),
        ("asset" = String, Path, description = "Asset name")
    ),
    responses(
        (status = 200, description = "Asset details", body = Asset),
        (status = 404, description = "Asset not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_asset(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((catalog_name, namespace, asset_name)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    // Resolve catalog to get ID for permission check
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get catalog: {}", e)).into_response(),
    };

    // Parse namespace
    let namespace_parts: Vec<String> = namespace.split('.').map(|s| s.to_string()).collect();

    // Get asset first to resolve ID for permission check
    let asset = match store.get_asset(tenant_id, &catalog_name, None, namespace_parts.clone(), asset_name.clone()).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, "Asset not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get asset: {}", e)).into_response(),
    };

    // Permission Check
    let scope = PermissionScope::Asset { 
        catalog_id: catalog.id, 
        namespace: namespace.clone(), 
        asset_id: asset.id 
    };
    
    match check_permission(&store, &session, &Action::Read, &scope).await {
         Ok(true) => (StatusCode::OK, Json(asset)).into_response(),
         Ok(false) => (StatusCode::FORBIDDEN, "Forbidden: Read permission required").into_response(),
         Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Permission check failed: {}", e)).into_response(),
    }
}
#[derive(Serialize, ToSchema)]
pub struct AssetSummary {
    pub id: Uuid,
    pub name: String,
    pub namespace: Vec<String>,
    pub kind: AssetType,
    pub identifier: crate::iceberg::TableIdentifier,
}

/// List all assets in a namespace (Iceberg tables + Generic assets)
#[utoipa::path(
    get,
    path = "/api/v1/catalogs/{catalog_name}/namespaces/{namespace}/assets",
    tag = "Assets",
    params(
        ("catalog_name" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace")
    ),
    responses(
        (status = 200, description = "List of assets", body = Vec<AssetSummary>),
        (status = 404, description = "Catalog not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_assets(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<UserSession>,
    Path((catalog_name, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    // Resolve catalog
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get catalog: {}", e)).into_response(),
    };
    
    // Parse namespace
    let namespace_parts: Vec<String> = namespace.split('.').map(|s| s.to_string()).collect();
    
    // Get all assets
    // We use "main" branch for listings by default, or iterate all branches?
    // Ideally store.list_assets would take an optional branch.
    // However, existing store.list_assets might depend on implementation.
    // For now, let's assume "main" branch or whatever the store defaults to.
    
    match store.list_assets(tenant_id, &catalog_name, None, namespace_parts.clone()).await {
        Ok(assets) => {
            // Get user permissions
            let permissions = match store.list_user_permissions(session.user_id).await {
                Ok(p) => p,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get permissions: {}", e)).into_response(),
            };
            
            // Map for filtering
            let mut catalog_map = std::collections::HashMap::new();
            catalog_map.insert(catalog_name.clone(), catalog.id);
            
            // Prepare assets with metadata for filtering
            let mut assets_with_metadata = Vec::new();
            
            for asset in assets {
                // Fetch business metadata for discoverability check
                let metadata = match store.get_business_metadata(asset.id).await {
                    Ok(m) => m,
                    Err(e) => {
                         tracing::warn!("Failed to fetch metadata for asset {}: {}", asset.id, e);
                         None
                    }
                };
                
                assets_with_metadata.push((asset, metadata, catalog_name.clone(), namespace_parts.clone()));
            }

            // Filter assets based on permissions
            let filtered = crate::authz_utils::filter_assets(
                assets_with_metadata, 
                &permissions, 
                session.role, 
                &catalog_map
            );
            
            // Map to AssetSummary
            let summaries: Vec<AssetSummary> = filtered.into_iter().map(|(asset, _, _, _)| {
                AssetSummary {
                    id: asset.id,
                    name: asset.name.clone(),
                    namespace: namespace_parts.clone(),
                    kind: asset.kind,
                    identifier: crate::iceberg::TableIdentifier {
                        namespace: namespace_parts.clone(),
                        name: asset.name,
                    }
                }
            }).collect();
            
            (StatusCode::OK, Json(summaries)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list assets: {}", e)).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iceberg::AppState;
    use pangolin_core::user::UserSession;
    use pangolin_core::permission::{PermissionScope, Action};
    use crate::tenant::TenantId;
    use axum::Extension;
    use pangolin_core::model::{Catalog, CatalogType, Tenant, User, UserRole, Permission};
    use pangolin_store::memory::MemoryStore;
    use std::sync::Arc;
    use uuid::Uuid;

    async fn setup_test_store() -> (AppState, Uuid, Uuid, Uuid, Uuid) {
        let store = Arc::new(MemoryStore::new());
        
        // Create tenant
        let tenant = Tenant {
            id: Uuid::new_v4(),
            name: "test-tenant".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant.clone()).await.unwrap();
        
        // Create admin user
        let admin_user = User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            password_hash: "hash".to_string(),
            role: UserRole::TenantAdmin,
            tenant_id: Some(tenant.id),
        };
        store.create_user(admin_user.clone()).await.unwrap();
        
        // Create regular user
        let regular_user = User {
            id: Uuid::new_v4(),
            username: "user".to_string(),
            password_hash: "hash".to_string(),
            role: UserRole::TenantUser,
            tenant_id: Some(tenant.id),
        };
        store.create_user(regular_user.clone()).await.unwrap();
        
        // Create catalog
        let catalog = Catalog {
            id: Uuid::new_v4(),
            name: "test-catalog".to_string(),
            catalog_type: CatalogType::Local,
            warehouse_name: Some("test-warehouse".to_string()),
            storage_location: Some("s3://bucket/catalog".to_string()),
            properties: std::collections::HashMap::new(),
            federated_config: None,
        };
        store.create_catalog(tenant.id, catalog.clone()).await.unwrap();
        
        // Grant Create permission to regular user on namespace
        let permission = Permission {
            id: Uuid::new_v4(),
            user_id: Some(regular_user.id),
            role_id: None,
            scope: PermissionScope::Namespace {
                catalog_id: catalog.id,
                namespace: "test_ns".to_string(),
            },
            action: Action::Create,
        };
        store.create_permission(permission).await.unwrap();
        
        (
            store.clone() as AppState,
            tenant.id,
            catalog.id,
            admin_user.id,
            regular_user.id,
        )
    }

    #[tokio::test]
    async fn test_register_asset_success() {
        let (state, tenant_id, _catalog_id, _admin_id, user_id) = setup_test_store().await;
        
        let session = UserSession {
            user_id,
            username: "user".to_string(),
            role: UserRole::TenantUser,
            tenant_id: Some(tenant_id),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            issued_at: chrono::Utc::now(),
        };
        
        let payload = RegisterAssetRequest {
            name: "my_delta_table".to_string(),
            kind: AssetType::DeltaTable,
            location: "s3://bucket/delta/table".to_string(),
            properties: std::collections::HashMap::new(),
        };
        
        let response = register_asset(
            State(state),
            Extension(TenantId(tenant_id)),
            Extension(session),
            Path(("test-catalog".to_string(), "test_ns".to_string())),
            Json(payload),
        )
        .await
        .into_response();
        
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_register_asset_forbidden() {
        let (state, tenant_id, _catalog_id, _admin_id, user_id) = setup_test_store().await;
        
        let session = UserSession {
            user_id,
            username: "user".to_string(),
            role: UserRole::TenantUser,
            tenant_id: Some(tenant_id),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            issued_at: chrono::Utc::now(),
        };
        
        let payload = RegisterAssetRequest {
            name: "my_delta_table".to_string(),
            kind: AssetType::DeltaTable,
            location: "s3://bucket/delta/table".to_string(),
            properties: std::collections::HashMap::new(),
        };
        
        // Try to register in a namespace without permission
        let response = register_asset(
            State(state),
            Extension(TenantId(tenant_id)),
            Extension(session),
            Path(("test-catalog".to_string(), "other_ns".to_string())),
            Json(payload),
        )
        .await
        .into_response();
        
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_register_asset_catalog_not_found() {
        let (state, tenant_id, _catalog_id, _admin_id, user_id) = setup_test_store().await;
        
        let session = UserSession {
            user_id,
            username: "user".to_string(),
            role: UserRole::TenantUser,
            tenant_id: Some(tenant_id),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            issued_at: chrono::Utc::now(),
        };
        
        let payload = RegisterAssetRequest {
            name: "my_delta_table".to_string(),
            kind: AssetType::DeltaTable,
            location: "s3://bucket/delta/table".to_string(),
            properties: std::collections::HashMap::new(),
        };
        
        let response = register_asset(
            State(state),
            Extension(TenantId(tenant_id)),
            Extension(session),
            Path(("nonexistent-catalog".to_string(), "test_ns".to_string())),
            Json(payload),
        )
        .await
        .into_response();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_register_different_asset_types() {
        let (state, tenant_id, _catalog_id, _admin_id, user_id) = setup_test_store().await;
        
        let session = UserSession {
            user_id,
            username: "user".to_string(),
            role: UserRole::TenantUser,
            tenant_id: Some(tenant_id),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            issued_at: chrono::Utc::now(),
        };
        
        let asset_types = vec![
            AssetType::DeltaTable,
            AssetType::HudiTable,
            AssetType::ParquetTable,
            AssetType::CsvTable,
            AssetType::JsonTable,
            AssetType::View,
            AssetType::MlModel,
        ];
        
        for asset_type in asset_types {
            let payload = RegisterAssetRequest {
                name: format!("test_{:?}", asset_type),
                kind: asset_type.clone(),
                location: "s3://bucket/path".to_string(),
                properties: std::collections::HashMap::new(),
            };
            
            let response = register_asset(
                State(state.clone()),
                Extension(TenantId(tenant_id)),
                Extension(session.clone()),
                Path(("test-catalog".to_string(), "test_ns".to_string())),
                Json(payload),
            )
            .await
            .into_response();
            
            assert_eq!(response.status(), StatusCode::CREATED, "Failed for {:?}", asset_type);
        }
    }
}
