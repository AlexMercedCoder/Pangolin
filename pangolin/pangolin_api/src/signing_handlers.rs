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
use utoipa::{ToSchema, IntoParams};
use std::collections::HashMap;

// Cloud provider SDK imports (conditional on features)
#[cfg(feature = "aws-sts")]
use aws_sdk_sts::Client as StsClient;
#[cfg(feature = "aws-sts")]
use aws_config;

#[cfg(feature = "azure-oauth")]
use azure_identity::ClientSecretCredential;
#[cfg(feature = "azure-oauth")]
use azure_core::auth::TokenCredential;

#[cfg(feature = "gcp-oauth")]
use gcp_auth::{CustomServiceAccount, TokenProvider};

#[derive(Serialize, ToSchema)]
pub struct StorageCredential {
    pub prefix: String,
    pub config: HashMap<String, String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LoadCredentialsResponse {
    pub storage_credentials: Vec<StorageCredential>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct PresignParams {
    pub location: String,
}

#[derive(Serialize, ToSchema)]
pub struct PresignResponse {
    pub url: String,
}

// ============================================================================
// AWS STS Helper Functions
// ============================================================================

/// Assume an AWS IAM role using STS and return temporary credentials
#[cfg(feature = "aws-sts")]
pub async fn assume_role_aws(
    role_arn: &str,
    external_id: Option<&str>,
    session_name: &str,
) -> Result<(String, String, String, String), String> {
    let config = aws_config::load_from_env().await;
    let sts_client = StsClient::new(&config);
    
    let mut request = sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name(session_name)
        .duration_seconds(3600); // 1 hour
    
    if let Some(ext_id) = external_id {
        request = request.external_id(ext_id);
    }
    
    let response = request.send().await
        .map_err(|e| format!("STS AssumeRole failed: {}", e))?;
    
    let creds = response.credentials()
        .ok_or_else(|| "No credentials returned from STS".to_string())?;
    
    Ok((
        creds.access_key_id().to_string(),
        creds.secret_access_key().to_string(),
        creds.session_token().to_string(),
        creds.expiration().to_string(),
    ))
}

/// Fallback implementation when AWS STS feature is not enabled
#[cfg(not(feature = "aws-sts"))]
pub async fn assume_role_aws(
    role_arn: &str,
    external_id: Option<&str>,
    session_name: &str,
) -> Result<(String, String, String, String), String> {
    tracing::warn!("AWS STS feature not enabled, returning placeholder credentials");
    Ok((
        format!("STS_ACCESS_KEY_FOR_{}", role_arn),
        "STS_SECRET_KEY_PLACEHOLDER".to_string(),
        "STS_SESSION_TOKEN_PLACEHOLDER".to_string(),
        chrono::Utc::now().checked_add_signed(chrono::Duration::hours(1))
            .unwrap().to_rfc3339(),
    ))
}

// ============================================================================
// Azure OAuth2 Helper Functions
// ============================================================================

/// Get an Azure OAuth2 token using client credentials
#[cfg(feature = "azure-oauth")]
pub async fn get_azure_token(
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String, String> {
    let authority_host = azure_core::Url::parse("https://login.microsoftonline.com")
        .map_err(|e| format!("Failed to parse authority host: {}", e))?;
    
    let credential = ClientSecretCredential::new(
        azure_core::new_http_client(),
        authority_host,
        tenant_id.to_string(),
        client_id.to_string(),
        client_secret.to_string(),
    );
    
    let token = credential
        .get_token(&["https://storage.azure.com/.default"])
        .await
        .map_err(|e| format!("Azure token acquisition failed: {}", e))?;
    
    Ok(token.token.secret().to_string())
}

/// Fallback implementation when Azure OAuth feature is not enabled
#[cfg(not(feature = "azure-oauth"))]
pub async fn get_azure_token(
    _tenant_id: &str,
    _client_id: &str,
    _client_secret: &str,
) -> Result<String, String> {
    tracing::warn!("Azure OAuth feature not enabled, returning placeholder token");
    Ok("AZURE_OAUTH_TOKEN_PLACEHOLDER".to_string())
}

// ============================================================================
// GCP Service Account Helper Functions
// ============================================================================

/// Get a GCP OAuth2 token using service account credentials
#[cfg(feature = "gcp-oauth")]
pub async fn get_gcp_token(
    service_account_key_json: &str,
) -> Result<String, String> {
    let service_account = CustomServiceAccount::from_json(service_account_key_json)
        .map_err(|e| format!("Failed to parse GCP service account key: {}", e))?;
    
    let scopes = &["https://www.googleapis.com/auth/devstorage.read_write"];
    let token = service_account
        .token(scopes)
        .await
        .map_err(|e| format!("GCP token acquisition failed: {}", e))?;
    
    Ok(token.as_str().to_string())
}

/// Fallback implementation when GCP OAuth feature is not enabled
#[cfg(not(feature = "gcp-oauth"))]
pub async fn get_gcp_token(
    _service_account_key_json: &str,
) -> Result<String, String> {
    tracing::warn!("GCP OAuth feature not enabled, returning placeholder token");
    Ok("GCS_OAUTH_TOKEN_PLACEHOLDER".to_string())
}

// ============================================================================
// Credential Vending Handlers
// ============================================================================

/// Get credentials for accessing a table
/// This endpoint vends credentials based on the catalog's warehouse configuration
/// Get credentials for accessing a table
/// This endpoint vends credentials based on the catalog's warehouse configuration
#[utoipa::path(
    get,
    path = "/v1/{prefix}/namespaces/{namespace}/tables/{table}/credentials",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("namespace" = String, Path, description = "Namespace"),
        ("table" = String, Path, description = "Table")
    ),
    responses(
        (status = 200, description = "Credentials vended", body = LoadCredentialsResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Catalog or Warehouse not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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
                // Azure OAuth2 mode - get token using client credentials
                let tenant_id = match warehouse.storage_config.get("tenant_id") {
                    Some(id) => id,
                    None => return (StatusCode::BAD_REQUEST, "Azure tenant_id required for OAuth2").into_response(),
                };
                
                let client_id = match warehouse.storage_config.get("client_id") {
                    Some(id) => id,
                    None => return (StatusCode::BAD_REQUEST, "Azure client_id required for OAuth2").into_response(),
                };
                
                let client_secret = match warehouse.storage_config.get("client_secret") {
                    Some(secret) => secret,
                    None => return (StatusCode::BAD_REQUEST, "Azure client_secret required for OAuth2").into_response(),
                };
                
                match get_azure_token(tenant_id, client_id, client_secret).await {
                    Ok(token) => {
                        config.insert("adls.auth.type".to_string(), "OAuth2".to_string());
                        config.insert("adls.oauth2.token".to_string(), token);
                        tracing::info!("✅ Successfully vended Azure OAuth2 token");
                    },
                    Err(e) => {
                        tracing::error!("Failed to get Azure token: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to vend Azure credentials: {}", e)).into_response();
                    }
                }
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
                // GCP OAuth2 mode - get token using service account
                let service_account_key = match warehouse.storage_config.get("service_account_key") {
                    Some(key) => key,
                    None => return (StatusCode::BAD_REQUEST, "GCS service_account_key required for OAuth2").into_response(),
                };
                
                match get_gcp_token(service_account_key).await {
                    Ok(token) => {
                        config.insert("gcs.auth.type".to_string(), "OAuth2".to_string());
                        config.insert("gcs.oauth2.token".to_string(), token);
                        tracing::info!("✅ Successfully vended GCP OAuth2 token");
                    },
                    Err(e) => {
                        tracing::error!("Failed to get GCP token: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to vend GCP credentials: {}", e)).into_response();
                    }
                }
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
                // AWS STS mode: Generate temporary credentials via AssumeRole
                let role_arn = warehouse.storage_config.get("role_arn")
                    .cloned()
                    .unwrap_or_else(|| "arn:aws:iam::123456789012:role/PangolinRole".to_string());
                
                let external_id = warehouse.storage_config.get("external_id")
                    .map(|s| s.as_str());
                
                let session_name = format!("pangolin-{}-{}", tenant_id, catalog_name);
                
                match assume_role_aws(&role_arn, external_id, &session_name).await {
                    Ok((access_key, secret_key, session_token, expiration)) => {
                        config.insert("access-key".to_string(), access_key);
                        config.insert("secret-key".to_string(), secret_key);
                        config.insert("session-token".to_string(), session_token);
                        config.insert("expiration".to_string(), expiration);
                        tracing::info!("✅ Successfully assumed AWS role: {}", role_arn);
                    },
                    Err(e) => {
                        tracing::error!("Failed to assume AWS role: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to vend AWS credentials: {}", e)).into_response();
                    }
                }
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
/// Get a presigned URL for a specific file location
#[utoipa::path(
    get,
    path = "/api/v1/storage/presigned-url",
    tag = "Data Explorer",
    params(
        PresignParams
    ),
    responses(
        (status = 200, description = "Presigned URL generated", body = PresignResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_presigned_url(
    State(_store): State<AppState>,
    Query(params): Query<PresignParams>,
) -> impl IntoResponse {
    // For MVP, return a placeholder
    // In production, this would use the S3 SDK to generate a presigned URL
    let url = format!("https://presigned-url-placeholder.s3.amazonaws.com/{}?X-Amz-Expires=3600", params.location);
    
    (StatusCode::OK, Json(PresignResponse { url })).into_response()
}
