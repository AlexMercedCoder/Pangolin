use axum::{
    extract::{Path, State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::model::Tenant;
use uuid::Uuid;
use crate::auth::TenantId;
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
    Extension(_tenant): Extension<TenantId>, // Admin check? For now open.
) -> impl IntoResponse {
    match store.list_tenants().await {
        Ok(tenants) => {
            let response: Vec<TenantResponse> = tenants.into_iter().map(|t| TenantResponse::from(t)).collect();
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_tenant(
    State(store): State<AppState>,
    Extension(_tenant): Extension<TenantId>, // Admin check?
    Json(payload): Json<CreateTenantRequest>,
) -> impl IntoResponse {
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
