use axum::{
    extract::{Path, State, Extension},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use pangolin_store::CatalogStore;
use pangolin_core::model::Tenant;
use uuid::Uuid;
use crate::auth::TenantId;
use crate::iceberg::AppState;
use utoipa::ToSchema;
use pangolin_core::user::{UserSession, UserRole};

#[derive(Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    name: String,
    properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    name: Option<String>,
    properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, ToSchema)]
pub struct TenantResponse {
    id: Uuid,
    pub name: String,
    pub properties: Option<HashMap<String, String>>,
}

impl From<Tenant> for TenantResponse {
    fn from(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            name: tenant.name,
            properties: Some(tenant.properties),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/tenants",
    tag = "Tenants",
    responses(
        (status = 200, description = "List of tenants", body = Vec<TenantResponse>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_tenants(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
) -> impl IntoResponse {
    let no_auth_enabled = std::env::var("PANGOLIN_NO_AUTH")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if !no_auth_enabled && session.role != UserRole::Root {
         return ApiError::forbidden("Only Root can list tenants").into_response();
    }

    match store.list_tenants().await {
        Ok(tenants) => {
            let response: Vec<TenantResponse> = tenants.into_iter().map(|t: Tenant| TenantResponse::from(t)).collect();
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => ApiError::from(e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/tenants",
    tag = "Tenants",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant created", body = TenantResponse),
        (status = 403, description = "Forbidden in NO_AUTH mode"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_tenant(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<CreateTenantRequest>,
) -> impl IntoResponse {
    // Check if running in no-auth mode
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

    if session.role != UserRole::Root {
         return (StatusCode::FORBIDDEN, "Only Root can create tenants").into_response();
    }
    
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: payload.name,
        properties: payload.properties.unwrap_or_default(),
    };

    match store.create_tenant(tenant.clone()).await {
        Ok(_) => {
            (StatusCode::CREATED, Json(TenantResponse::from(tenant))).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/tenants/{id}",
    tag = "Tenants",
    params(
        ("id" = Uuid, Path, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Tenant details", body = TenantResponse),
        (status = 404, description = "Tenant not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_tenant(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let no_auth_enabled = std::env::var("PANGOLIN_NO_AUTH")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if !no_auth_enabled && session.role != UserRole::Root {
        // Tenants can view themselves? Usually yes.
        // If session tenant_id matches requested id
        if session.tenant_id != Some(id) {
             return ApiError::forbidden("Forbidden").into_response();
        }
    }

    match store.get_tenant(id).await {
        Ok(Some(tenant)) => (StatusCode::OK, Json(TenantResponse::from(tenant))).into_response(),
        Ok(None) => ApiError::not_found("Tenant not found").into_response(),
        Err(e) => ApiError::from(e).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/tenants/{id}",
    tag = "Tenants",
    params(
        ("id" = Uuid, Path, description = "Tenant ID")
    ),
    request_body = UpdateTenantRequest,
    responses(
        (status = 200, description = "Tenant updated", body = TenantResponse),
        (status = 404, description = "Tenant not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_tenant(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTenantRequest>,
) -> impl IntoResponse {
    // Only root can update tenants (for now)
    if session.role != UserRole::Root {
         return (StatusCode::FORBIDDEN, "Only Root can update tenants").into_response();
    }

    let updates = pangolin_core::model::TenantUpdate {
        name: payload.name,
        properties: payload.properties,
    };
    
    match store.update_tenant(id, updates).await {
        Ok(tenant) => (StatusCode::OK, Json(TenantResponse::from(tenant))).into_response(),
        Err(e) => {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "Tenant not found").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/tenants/{id}",
    tag = "Tenants",
    params(
        ("id" = Uuid, Path, description = "Tenant ID")
    ),
    responses(
        (status = 204, description = "Tenant deleted"),
        (status = 404, description = "Tenant not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_tenant(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if session.role != UserRole::Root {
         return (StatusCode::FORBIDDEN, "Only Root can delete tenants").into_response();
    }

    match store.delete_tenant(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "Tenant not found").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}
