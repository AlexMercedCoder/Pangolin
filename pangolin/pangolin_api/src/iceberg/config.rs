use axum::Json;
use std::collections::HashMap;
use super::types::CatalogConfig;

/// Get Iceberg Catalog Configuration
#[utoipa::path(
    get,
    path = "/v1/config",
    tag = "Iceberg REST",
    responses(
        (status = 200, description = "Catalog configuration", body = CatalogConfig),
    )
)]
pub async fn get_iceberg_catalog_config_handler() -> Json<CatalogConfig> {
    // Return Iceberg REST catalog config
    // Use X-Iceberg-Access-Delegation header to enable credential vending
    let mut defaults = HashMap::new();
    
    // This header tells PyIceberg to request credentials via the vend-credentials endpoint
    // PyIceberg will call POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/credentials
    defaults.insert("header.X-Iceberg-Access-Delegation".to_string(), "vended-credentials".to_string());
    
    // Optionally provide S3 endpoint if using MinIO or custom S3
    if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
        defaults.insert("s3.endpoint".to_string(), endpoint);
    } else if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
        defaults.insert("s3.endpoint".to_string(), endpoint);
    }
    
    // Add region if specified
    if let Ok(region) = std::env::var("AWS_REGION") {
        defaults.insert("s3.region".to_string(), region);
    }
    
    Json(CatalogConfig {
        defaults,
        overrides: HashMap::new(),
    })
}
