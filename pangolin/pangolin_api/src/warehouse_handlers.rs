use axum::{
    extract::{Path, State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::model::Warehouse;
use uuid::Uuid;
use crate::auth::TenantId;
use crate::iceberg_handlers::AppState;

#[derive(Deserialize)]
pub struct CreateWarehouseRequest {
    name: String,
    storage_config: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct WarehouseResponse {
    id: Uuid,
    name: String,
    tenant_id: Uuid,
    storage_config: std::collections::HashMap<String, String>,
}

impl From<Warehouse> for WarehouseResponse {
    fn from(warehouse: Warehouse) -> Self {
        Self {
            id: warehouse.id,
            name: warehouse.name,
            tenant_id: warehouse.tenant_id,
            storage_config: warehouse.storage_config,
        }
    }
}

pub async fn list_warehouses(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
) -> impl IntoResponse {
    match store.list_warehouses(tenant.0).await {
        Ok(warehouses) => {
            let response: Vec<WarehouseResponse> = warehouses.into_iter().map(|w: Warehouse| WarehouseResponse::from(w)).collect();
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn create_warehouse(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Json(payload): Json<CreateWarehouseRequest>,
) -> impl IntoResponse {
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: payload.name,
        tenant_id: tenant.0,
        storage_config: payload.storage_config.unwrap_or_default(),
    };

    match store.create_warehouse(tenant.0, warehouse.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(WarehouseResponse::from(warehouse))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

pub async fn get_warehouse(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match store.get_warehouse(tenant.0, name).await {
        Ok(Some(warehouse)) => (StatusCode::OK, Json(WarehouseResponse::from(warehouse))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Warehouse not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}
