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
use std::collections::HashMap;

#[derive(Serialize)]
pub struct StorageCredential {
    pub prefix: String,
    pub config: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct LoadCredentialsResponse {
    pub storage_credentials: Vec<StorageCredential>,
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
    
    tracing::info!("🔑 Credential vending requested for catalog: {}", catalog_name);
    
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
    // Get the storage location prefix from the warehouse
    let storage_type = warehouse.storage_config.get("type")
        .map(|s| s.as_str())
        .unwrap_or("s3");
    
    let storage_location = match storage_type {
        "azure" => {
            let container = warehouse.storage_config.get("container")
                .cloned()
                .unwrap_or_else(|| "warehouse".to_string());
            let account = warehouse.storage_config.get("account_name")
                .cloned()
                .unwrap_or_else(|| "account".to_string());
            format!("abfss://{}@{}.dfs.core.windows.net/", container, account)
        },
        "gcs" => {
            let bucket = warehouse.storage_config.get("bucket")
                .cloned()
                .unwrap_or_else(|| "warehouse".to_string());
            format!("gs://{}/", bucket)
        },
        _ => {
            // Default to S3
            warehouse.storage_config.get("bucket")
                .map(|bucket| format!("s3://{}/", bucket))
                .unwrap_or_else(|| "s3://warehouse/".to_string())
        }
    };
    
    let mut config = HashMap::new();
    
    match storage_type {
        "azure" => {
            // Azure ADLS Gen2 credentials
            if warehouse.use_sts {
                // Azure STS/OAuth2 mode
                // TODO: Implement Azure AD OAuth2 token generation
                config.insert("adls.auth.type".to_string(), "OAuth2".to_string());
                config.insert("adls.oauth2.token".to_string(), "AZURE_OAUTH_TOKEN_PLACEHOLDER".to_string());
            } else {
                // Azure account key mode
                let account_name = warehouse.storage_config.get("account_name")
                    .cloned()
                    .unwrap_or_default();
                let account_key = warehouse.storage_config.get("account_key")
                    .cloned()
                    .unwrap_or_default();
                
                if account_name.is_empty() || account_key.is_empty() {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Azure warehouse has no credentials configured").into_response();
                }
                
                config.insert("adls.account-name".to_string(), account_name);
                config.insert("adls.account-key".to_string(), account_key);
                
                // Add endpoint if configured
                if let Some(endpoint) = warehouse.storage_config.get("endpoint") {
                    config.insert("adls.endpoint".to_string(), endpoint.clone());
                }
            }
        },
        "gcs" => {
            // Google Cloud Storage credentials
            if warehouse.use_sts {
                // GCS OAuth2 mode
                // TODO: Implement GCS OAuth2 token generation
                config.insert("gcs.auth.type".to_string(), "OAuth2".to_string());
                config.insert("gcs.oauth2.token".to_string(), "GCS_OAUTH_TOKEN_PLACEHOLDER".to_string());
            } else {
                // GCS service account key mode
                let project_id = warehouse.storage_config.get("project_id")
                    .cloned()
                    .unwrap_or_default();
                let service_account_key = warehouse.storage_config.get("service_account_key")
                    .cloned()
                    .unwrap_or_default();
                
                if project_id.is_empty() || service_account_key.is_empty() {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "GCS warehouse has no credentials configured").into_response();
                }
                
                config.insert("gcs.project-id".to_string(), project_id);
                config.insert("gcs.service-account-key".to_string(), service_account_key);
                
                // Add endpoint if configured (for GCS emulator)
                if let Some(endpoint) = warehouse.storage_config.get("endpoint") {
                    config.insert("gcs.endpoint".to_string(), endpoint.clone());
                }
            }
        },
        _ => {
            // S3 credentials (existing logic)
            if warehouse.use_sts {
                // STS mode: Generate temporary credentials
                // For MVP, we'll return a placeholder indicating STS would be used
                // In production, this would call AWS STS AssumeRole
                
                // Get role ARN from storage_config
                let role_arn = warehouse.storage_config.get("role_arn")
                    .cloned()
                    .unwrap_or_else(|| "arn:aws:iam::123456789012:role/PangolinRole".to_string());
                
                // TODO: Implement actual STS AssumeRole call
                config.insert("access-key".to_string(), format!("STS_ACCESS_KEY_FOR_{}", role_arn));
                config.insert("secret-key".to_string(), "STS_SECRET_KEY_PLACEHOLDER".to_string());
                config.insert("session-token".to_string(), "STS_SESSION_TOKEN_PLACEHOLDER".to_string());
                config.insert("expiration".to_string(), chrono::Utc::now().checked_add_signed(chrono::Duration::hours(1)).unwrap().to_rfc3339());
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
                
                config.insert("access-key".to_string(), access_key);
                config.insert("secret-key".to_string(), secret_key);
            }
            
            // Add S3 endpoint if configured
            if let Some(endpoint) = warehouse.storage_config.get("endpoint") {
                config.insert("s3.endpoint".to_string(), endpoint.clone());
            }
            
            // Add region if configured
            if let Some(region) = warehouse.storage_config.get("region") {
                config.insert("s3.region".to_string(), region.clone());
            }
        }
    }
    
    let storage_credential = StorageCredential {
        prefix: storage_location,
        config,
    };
    
    let resp = LoadCredentialsResponse {
        storage_credentials: vec![storage_credential],
    };
    
    (StatusCode::OK, Json(resp)).into_response()
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
