use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use crate::iceberg_handlers::AppState;
use crate::auth::TenantId;
use axum::Extension;

#[derive(Serialize)]
pub struct CredentialsResponse {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub expiration: Option<String>,
}

#[derive(Deserialize)]
pub struct PresignParams {
    pub location: String,
}

#[derive(Serialize)]
pub struct PresignResponse {
    pub url: String,
}

/// Get credentials for accessing a table
/// This endpoint vends credentials based on the catalog's warehouse configuration
pub async fn get_table_credentials(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path((catalog_name, _namespace, _table)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;
    
    // 1. Get catalog to find associated warehouse
    let catalog = match store.get_catalog(tenant_id, catalog_name.clone()).await {
        Ok(Some(cat)) => cat,
        Ok(None) => return (StatusCode::NOT_FOUND, "Catalog not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    
    // 2. Check if catalog has a warehouse
    let warehouse_name = match catalog.warehouse_name {
        Some(name) => name,
        None => {
            // No warehouse configured - client must provide credentials
            return (StatusCode::BAD_REQUEST, "Catalog has no warehouse configured. Client must provide storage credentials.").into_response();
        }
    };
    
    // 3. Get warehouse configuration
    let warehouse = match store.get_warehouse(tenant_id, warehouse_name).await {
        Ok(Some(wh)) => wh,
        Ok(None) => return (StatusCode::NOT_FOUND, "Warehouse not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    
    // 4. Vend credentials based on warehouse configuration
    if warehouse.use_sts {
        // STS mode: Generate temporary credentials
        // For MVP, we'll return a placeholder indicating STS would be used
        // In production, this would call AWS STS AssumeRole
        
        // Get role ARN from storage_config
        let role_arn = warehouse.storage_config.get("role_arn")
            .cloned()
            .unwrap_or_else(|| "arn:aws:iam::123456789012:role/PangolinRole".to_string());
        
        // TODO: Implement actual STS AssumeRole call
        // For now, return placeholder credentials
        let resp = CredentialsResponse {
            access_key_id: format!("STS_ACCESS_KEY_FOR_{}", role_arn),
            secret_access_key: "STS_SECRET_KEY_PLACEHOLDER".to_string(),
            session_token: Some("STS_SESSION_TOKEN_PLACEHOLDER".to_string()),
            expiration: Some(chrono::Utc::now().checked_add_signed(chrono::Duration::hours(1)).unwrap().to_rfc3339()),
        };
        
        (StatusCode::OK, Json(resp)).into_response()
    } else {
        // Static mode: Pass through credentials from warehouse
        let access_key = warehouse.storage_config.get("access_key_id")
            .cloned()
            .unwrap_or_default();
        let secret_key = warehouse.storage_config.get("secret_access_key")
            .cloned()
            .unwrap_or_default();
        
        if access_key.is_empty() || secret_key.is_empty() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Warehouse has no credentials configured").into_response();
        }
        
        let resp = CredentialsResponse {
            access_key_id: access_key,
            secret_access_key: secret_key,
            session_token: None,
            expiration: None,
        };
        
        (StatusCode::OK, Json(resp)).into_response()
    }
}

/// Get a presigned URL for a specific file location
pub async fn get_presigned_url(
    State(_store): State<AppState>,
    Query(params): Query<PresignParams>,
) -> impl IntoResponse {
    // For MVP, return a placeholder
    // In production, this would use the S3 SDK to generate a presigned URL
    let url = format!("https://presigned-url-placeholder.s3.amazonaws.com/{}?X-Amz-Expires=3600", params.location);
    
    (StatusCode::OK, Json(PresignResponse { url })).into_response()
}
