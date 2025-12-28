use axum::{
    extract::{Path, State, Extension, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pangolin_store::{CatalogStore, PaginationParams};
use pangolin_core::model::{Warehouse, VendingStrategy};
use uuid::Uuid;
use crate::auth::TenantId;
use crate::iceberg::AppState;
use pangolin_core::user::UserRole;
use utoipa::ToSchema;
use pangolin_store::signer::Credentials;

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub struct GetCredentialsParams {
    location: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateWarehouseRequest {
    pub name: String,
    pub use_sts: Option<bool>, // If true, use STS credential vending; if false, pass through static creds
    pub storage_config: Option<std::collections::HashMap<String, String>>,
    pub vending_strategy: Option<VendingStrategy>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateWarehouseRequest {
    name: Option<String>,
    use_sts: Option<bool>,
    storage_config: Option<std::collections::HashMap<String, String>>,
    vending_strategy: Option<VendingStrategy>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct WarehouseResponse {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub use_sts: bool,
    pub storage_config: std::collections::HashMap<String, String>,
    pub vending_strategy: Option<VendingStrategy>,
}

impl From<Warehouse> for WarehouseResponse {
    fn from(warehouse: Warehouse) -> Self {
        Self {
            id: warehouse.id,
            name: warehouse.name,
            tenant_id: warehouse.tenant_id,
            use_sts: warehouse.use_sts,
            storage_config: warehouse.storage_config,
            vending_strategy: warehouse.vending_strategy,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/warehouses",
    tag = "Warehouses",
    responses(
        (status = 200, description = "List of warehouses", body = Vec<WarehouseResponse>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_warehouses(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    match store.list_warehouses(tenant.0, Some(pagination)).await {
        Ok(warehouses) => {
            let response: Vec<WarehouseResponse> = warehouses.into_iter().map(|w: Warehouse| WarehouseResponse::from(w)).collect();
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/warehouses",
    tag = "Warehouses",
    request_body = CreateWarehouseRequest,
    responses(
        (status = 201, description = "Warehouse created", body = WarehouseResponse),
        (status = 403, description = "Forbidden for root user"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_warehouse(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Json(payload): Json<CreateWarehouseRequest>,
) -> impl IntoResponse {
    if session.role == pangolin_core::user::UserRole::Root {
        return (StatusCode::FORBIDDEN, "Root user cannot create warehouses. Please login as Tenant Admin.").into_response();
    }

    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: payload.name,
        tenant_id: tenant.0,
        use_sts: payload.use_sts.unwrap_or(false), // Default to false (static credentials)
        storage_config: payload.storage_config.unwrap_or_default(),
        vending_strategy: payload.vending_strategy,
    };

    match store.create_warehouse(tenant.0, warehouse.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(WarehouseResponse::from(warehouse))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create warehouse: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal Server Error: {}", e)).into_response()
        },
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/warehouses/{name}",
    tag = "Warehouses",
    params(
        ("name" = String, Path, description = "Warehouse name")
    ),
    responses(
        (status = 200, description = "Warehouse details", body = WarehouseResponse),
        (status = 404, description = "Warehouse not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/warehouses/{name}",
    tag = "Warehouses",
    params(
        ("name" = String, Path, description = "Warehouse name")
    ),
    responses(
        (status = 204, description = "Warehouse deleted"),
        (status = 404, description = "Warehouse not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_warehouse(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match store.delete_warehouse(tenant.0, name.clone()).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, format!("Warehouse '{}' not found", name)).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/warehouses/{name}",
    tag = "Warehouses",
    params(
        ("name" = String, Path, description = "Warehouse name")
    ),
    request_body = UpdateWarehouseRequest,
    responses(
        (status = 200, description = "Warehouse updated", body = WarehouseResponse),
        (status = 404, description = "Warehouse not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_warehouse(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateWarehouseRequest>,
) -> impl IntoResponse {
    let updates = pangolin_core::model::WarehouseUpdate {
        name: payload.name,
        use_sts: payload.use_sts,
        storage_config: payload.storage_config,
        vending_strategy: payload.vending_strategy,
    };
    
    match store.update_warehouse(tenant.0, name, updates).await {
        Ok(warehouse) => (StatusCode::OK, Json(WarehouseResponse::from(warehouse))).into_response(),
        Err(e) => {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "Warehouse not found").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/warehouses/{name}/credentials",
    tag = "Warehouses",
    params(
        ("name" = String, Path, description = "Warehouse name"),
        ("location" = String, Query, description = "Location to get credentials for")
    ),
    responses(
        (status = 200, description = "Credentials returned", body = Credentials),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_warehouse_credentials(
    State(store): State<AppState>,
    Extension(_tenant): Extension<TenantId>,
    Path(_name): Path<String>,
    Query(params): Query<GetCredentialsParams>,
) -> impl IntoResponse {
    match store.get_table_credentials(&params.location).await {
        Ok(creds) => (StatusCode::OK, Json(creds)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}
