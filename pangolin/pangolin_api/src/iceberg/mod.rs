use crate::federated_proxy::FederatedCatalogProxy;
use axum::{
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use pangolin_core::model::CatalogType;
use pangolin_store::CatalogStore;
use std::sync::Arc;

pub mod commit;
pub mod config;
pub mod error;
pub mod namespaces;
pub mod oauth;
pub mod tables;
pub mod types;

// Re-export types for convenience
pub use error::{
    bad_request, forbidden, iceberg_error, internal, namespace_already_exists, namespace_not_empty,
    no_such_namespace, no_such_table, no_such_view, table_already_exists,
};
pub use types::*;
pub type AppState = std::sync::Arc<dyn pangolin_store::CatalogStore + Send + Sync>;

/// Helper function to check if a catalog is federated and forward the request if so
pub async fn check_and_forward_if_federated(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    tenant_id: uuid::Uuid,
    catalog_name: &str,
    method: Method,
    path: &str,
    body: Option<Bytes>,
    headers: HeaderMap,
) -> Option<axum::response::Response> {
    // Get the catalog
    let catalog = match store.get_catalog(tenant_id, catalog_name.to_string()).await {
        Ok(Some(c)) => c,
        Ok(None) => return None, // Catalog not found, let handler deal with it
        Err(_) => return None,
    };

    // Check if it's federated
    if catalog.catalog_type == CatalogType::Federated {
        if let Some(config) = catalog.federated_config {
            let proxy = FederatedCatalogProxy::new();
            match proxy
                .forward_request(&config, method, path, body, headers)
                .await
            {
                Ok(response) => Some(response),
                Err(e) => Some(
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(
                            serde_json::json!({"error": format!("Federated catalog error: {}", e)}),
                        ),
                    )
                        .into_response(),
                ),
            }
        } else {
            Some(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Federated catalog missing configuration"})),
                )
                    .into_response(),
            )
        }
    } else {
        None // Not federated, continue with local handling
    }
}
