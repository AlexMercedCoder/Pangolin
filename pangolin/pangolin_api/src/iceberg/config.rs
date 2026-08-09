use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use super::types::CatalogConfig;
use super::AppState;
use crate::auth::TenantId;

/// Query parameters the Iceberg spec defines for `GET /v1/config`.
#[derive(Debug, Default, Deserialize)]
pub struct ConfigParams {
    /// The warehouse the client intends to use.
    pub warehouse: Option<String>,
}

/// `GET /v1/config`
#[utoipa::path(
    get,
    path = "/v1/config",
    tag = "Iceberg REST",
    params(("warehouse" = Option<String>, Query, description = "Warehouse the client will use")),
    responses(
        (status = 200, description = "Catalog configuration", body = CatalogConfig),
    )
)]
pub async fn get_iceberg_catalog_config_handler(
    State(store): State<AppState>,
    tenant: Option<axum::extract::Extension<TenantId>>,
    Query(params): Query<ConfigParams>,
) -> Json<CatalogConfig> {
    build_config(&store, tenant.map(|t| t.0), None, params.warehouse).await
}

/// `GET /v1/{prefix}/config`
///
/// The handler used to take **no arguments at all**: it ignored the `:prefix`
/// path segment and the spec's `?warehouse=` query parameter, and built
/// `defaults` from process-wide environment variables. Every catalog in a
/// multi-warehouse, multi-cloud deployment therefore received the same S3
/// endpoint and region — a tenant with an Azure warehouse got the server's AWS
/// settings (A-4). `overrides` was always empty, so the server never returned
/// the `prefix` the spec uses to tell clients which path to address.
#[utoipa::path(
    get,
    path = "/v1/{prefix}/config",
    tag = "Iceberg REST",
    params(
        ("prefix" = String, Path, description = "Catalog name"),
        ("warehouse" = Option<String>, Query, description = "Warehouse the client will use"),
    ),
    responses(
        (status = 200, description = "Catalog configuration", body = CatalogConfig),
    )
)]
pub async fn get_prefixed_catalog_config_handler(
    State(store): State<AppState>,
    tenant: Option<axum::extract::Extension<TenantId>>,
    Path(prefix): Path<String>,
    Query(params): Query<ConfigParams>,
) -> Json<CatalogConfig> {
    build_config(&store, tenant.map(|t| t.0), Some(prefix), params.warehouse).await
}

/// Storage keys that must never be handed to a client through `/config`.
///
/// `/config` is unauthenticated by design, so it may expose connection shape
/// but never credentials. Credentials are vended through the authenticated
/// credentials endpoint instead.
const SECRET_KEY_FRAGMENTS: &[&str] = &[
    "secret",
    "access-key",
    "password",
    "token",
    "credential",
    "private",
    "sas",
];

fn is_secret(key: &str) -> bool {
    let lowered = key.to_ascii_lowercase();
    SECRET_KEY_FRAGMENTS.iter().any(|f| lowered.contains(f))
}

/// Build the config document for a specific catalog and warehouse when we can
/// resolve them, falling back to the process-wide storage settings otherwise.
async fn build_config(
    store: &AppState,
    tenant: Option<TenantId>,
    prefix: Option<String>,
    warehouse: Option<String>,
) -> Json<CatalogConfig> {
    let mut defaults = HashMap::new();
    let mut overrides = HashMap::new();

    // Tells PyIceberg to request credentials from the vending endpoint.
    defaults.insert(
        "header.X-Iceberg-Access-Delegation".to_string(),
        "vended-credentials".to_string(),
    );

    // The spec uses `overrides.prefix` to tell the client which path to
    // address. Without it a client cannot discover the prefixed routes.
    if let Some(prefix) = prefix.as_ref() {
        overrides.insert("prefix".to_string(), prefix.clone());
    }

    // Resolve the warehouse: explicit `?warehouse=`, else the one the named
    // catalog is attached to.
    let resolved = match (tenant, warehouse.clone(), prefix.as_ref()) {
        (Some(TenantId(tenant_id)), Some(name), _) => {
            store.get_warehouse(tenant_id, name).await.ok().flatten()
        }
        (Some(TenantId(tenant_id)), None, Some(catalog_name)) => {
            match store.get_catalog(tenant_id, catalog_name.clone()).await {
                Ok(Some(catalog)) => match catalog.warehouse_name {
                    Some(name) => store.get_warehouse(tenant_id, name).await.ok().flatten(),
                    None => None,
                },
                _ => None,
            }
        }
        _ => None,
    };

    match resolved {
        Some(found) => {
            defaults.insert("warehouse".to_string(), found.name.clone());
            for (key, value) in found.storage_config.iter() {
                if is_secret(key) {
                    continue;
                }
                defaults.insert(key.clone(), value.clone());
            }
        }
        None => {
            // No warehouse in scope: fall back to the server's storage
            // settings, which is all the old handler ever did.
            if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                defaults.insert("s3.endpoint".to_string(), endpoint);
            } else if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
                defaults.insert("s3.endpoint".to_string(), endpoint);
            }
            if let Ok(region) = std::env::var("AWS_REGION") {
                defaults.insert("s3.region".to_string(), region);
            }
            if let Some(name) = warehouse {
                defaults.insert("warehouse".to_string(), name);
            }
        }
    }

    Json(CatalogConfig {
        defaults,
        overrides,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_bearing_keys_are_never_exposed() {
        for key in [
            "s3.secret-access-key",
            "s3.access-key-id",
            "s3.session-token",
            "gcs.private-key",
            "adls.sas-token",
            "azure.account-password",
            "S3.SECRET-ACCESS-KEY",
        ] {
            assert!(is_secret(key), "{key} must be treated as a secret");
        }
    }

    #[test]
    fn connection_shape_keys_are_exposed() {
        for key in [
            "s3.endpoint",
            "s3.region",
            "s3.path-style-access",
            "warehouse",
        ] {
            assert!(!is_secret(key), "{key} should be safe to publish");
        }
    }
}
