use axum::{
    extract::{Path, State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use pangolin_store::CatalogStore;
use pangolin_core::model::Tenant;
use uuid::Uuid;
use crate::auth::{TenantId, RootUser};
use crate::iceberg_handlers::AppState;

#[derive(Deserialize)]
pub struct CreateTenantRequest {
    name: String,
    properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct TenantResponse {
    id: Uuid,
    name: String,
    properties: std::collections::HashMap<String, String>,
}

impl From<Tenant> for TenantResponse {
    fn from(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            name: tenant.name,
            properties: tenant.properties,
        }
    }
}

pub async fn list_tenants(
    State(store): State<AppState>,
    Extension(_root): Extension<RootUser>,
) -> impl IntoResponse {
    match store.list_tenants().await {
        Ok(tenants) => {
            let response: Vec<TenantResponse> = tenants.into_iter().map(|t: Tenant| TenantResponse::from(t)).collect();
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_tenant(
    State(store): State<AppState>,
    Extension(_root): Extension<RootUser>,
    Json(payload): Json<CreateTenantRequest>,
) -> impl IntoResponse {
    // Check if running in no-auth mode
    // Check if NO_AUTH mode is enabled (must be exactly "true" for security)
    let no_auth_enabled = std::env::var("PANGOLIN_NO_AUTH")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    if no_auth_enabled {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Tenant creation is disabled in NO_AUTH mode",
                "hint": "Remove PANGOLIN_NO_AUTH environment variable and use /api/v1/tokens endpoint to generate tokens"
            })),
        )
            .into_response();
    }
    
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: payload.name,
        properties: payload.properties.unwrap_or_default(),
    };

    match store.create_tenant(tenant.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(TenantResponse::from(tenant))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_tenant(
    State(store): State<AppState>,
    Extension(_tenant): Extension<TenantId>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match store.get_tenant(id).await {
        Ok(Some(tenant)) => (StatusCode::OK, Json(TenantResponse::from(tenant))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Tenant not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}
