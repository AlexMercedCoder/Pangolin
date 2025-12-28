use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use pangolin_core::user::{ServiceUser, ApiKeyResponse, UserRole};
use pangolin_store::{CatalogStore, PaginationParams};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};
use bcrypt::{hash, DEFAULT_COST};
use crate::auth::TenantId;
use std::sync::Arc;
use utoipa::ToSchema;

type AppState = Arc<dyn CatalogStore + Send + Sync>;

// Request/Response types
#[derive(Deserialize, ToSchema)]
pub struct CreateServiceUserRequest {
    pub name: String,
    pub description: Option<String>,
    pub role: UserRole,
    pub expires_in_days: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateServiceUserRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
}

// Generate a secure random API key
fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const KEY_LEN: usize = 64;
    let mut rng = rand::thread_rng();

    let key: String = (0..KEY_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("pgl_{}", key)
}

/// Create a new service user
/// POST /api/v1/service-users
#[utoipa::path(
    post,
    path = "/api/v1/service-users",
    tag = "Service Users",
    request_body = CreateServiceUserRequest,
    responses(
        (status = 201, description = "Service user created", body = ApiKeyResponse),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_service_user(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Json(payload): Json<CreateServiceUserRequest>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;

    // Only tenant admins and root users can create service users
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can create service users"})),
        ).into_response();
    }

    // Generate API key
    let api_key = generate_api_key();
    let api_key_hash = match hash(&api_key, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to hash API key: {}", e)})),
            ).into_response();
        }
    };

    // Calculate expiration
    let expires_at = payload.expires_in_days.map(|days| Utc::now() + Duration::days(days));

    // Create service user
    let service_user = ServiceUser::new(
        payload.name.clone(),
        payload.description.clone(),
        tenant_id,
        api_key_hash,
        payload.role,
        session.user_id,
        expires_at,
    );

    let service_user_id = service_user.id;

    match store.create_service_user(service_user.clone()).await {
        Ok(_) => {
            let response = ApiKeyResponse {
                id: service_user.id,
                name: service_user.name,
                description: service_user.description,
                tenant_id: service_user.tenant_id,
                role: service_user.role,
                api_key,  // Only shown once!
                created_at: service_user.created_at,
                created_by: service_user.created_by,
                expires_at: service_user.expires_at,
                active: service_user.active,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to create service user: {}", e)})),
        ).into_response(),
    }
}

/// List service users for the tenant
/// GET /api/v1/service-users
#[utoipa::path(
    get,
    path = "/api/v1/service-users",
    tag = "Service Users",
    responses(
        (status = 200, description = "List of service users", body = Vec<ServiceUser>),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_service_users(
    State(store): State<AppState>,
    Extension(tenant): Extension<TenantId>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    let tenant_id = tenant.0;

    // Only admins can list service users
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can list service users"})),
        ).into_response();
    }

    match store.list_service_users(tenant_id, Some(pagination)).await {
        Ok(service_users) => (StatusCode::OK, Json(service_users)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to list service users: {}", e)})),
        ).into_response(),
    }
}

/// Get a specific service user
/// GET /api/v1/service-users/{id}
#[utoipa::path(
    get,
    path = "/api/v1/service-users/{id}",
    tag = "Service Users",
    params(
        ("id" = Uuid, Path, description = "Service user ID")
    ),
    responses(
        (status = 200, description = "Service user details", body = ServiceUser),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service user not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_service_user(
    State(store): State<AppState>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Only admins can view service users
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can view service users"})),
        ).into_response();
    }

    match store.get_service_user(id).await {
        Ok(Some(service_user)) => (StatusCode::OK, Json(service_user)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Service user not found"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to get service user: {}", e)})),
        ).into_response(),
    }
}

/// Update a service user
/// PUT /api/v1/service-users/{id}
#[utoipa::path(
    put,
    path = "/api/v1/service-users/{id}",
    tag = "Service Users",
    params(
        ("id" = Uuid, Path, description = "Service user ID")
    ),
    request_body = UpdateServiceUserRequest,
    responses(
        (status = 200, description = "Service user updated"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_service_user(
    State(store): State<AppState>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateServiceUserRequest>,
) -> impl IntoResponse {
    // Only admins can update service users
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can update service users"})),
        ).into_response();
    }

    match store.update_service_user(id, payload.name, payload.description, payload.active).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "updated"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to update service user: {}", e)})),
        ).into_response(),
    }
}

/// Delete a service user
/// DELETE /api/v1/service-users/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/service-users/{id}",
    tag = "Service Users",
    params(
        ("id" = Uuid, Path, description = "Service user ID")
    ),
    responses(
        (status = 200, description = "Service user deleted"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_service_user(
    State(store): State<AppState>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Only admins can delete service users
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can delete service users"})),
        ).into_response();
    }

    match store.delete_service_user(id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to delete service user: {}", e)})),
        ).into_response(),
    }
}

/// Rotate API key for a service user
/// POST /api/v1/service-users/{id}/rotate
#[utoipa::path(
    post,
    path = "/api/v1/service-users/{id}/rotate",
    tag = "Service Users",
    params(
        ("id" = Uuid, Path, description = "Service user ID")
    ),
    responses(
        (status = 200, description = "API key rotated", body = ApiKeyResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service user not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn rotate_api_key(
    State(store): State<AppState>,
    Extension(session): Extension<pangolin_core::user::UserSession>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Only admins can rotate API keys
    if session.role != UserRole::TenantAdmin && session.role != UserRole::Root {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only admins can rotate API keys"})),
        ).into_response();
    }

    // Get existing service user
    let service_user = match store.get_service_user(id).await {
        Ok(Some(su)) => su,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Service user not found"})),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to get service user: {}", e)})),
            ).into_response();
        }
    };

    // Generate new API key
    let new_api_key = generate_api_key();
    let new_api_key_hash = match hash(&new_api_key, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to hash API key: {}", e)})),
            ).into_response();
        }
    };

    // Update service user with new hash
    let mut updated_service_user = service_user.clone();
    updated_service_user.api_key_hash = new_api_key_hash;

    match store.create_service_user(updated_service_user.clone()).await {
        Ok(_) => {
            let response = ApiKeyResponse {
                id: updated_service_user.id,
                name: updated_service_user.name,
                description: updated_service_user.description,
                tenant_id: updated_service_user.tenant_id,
                role: updated_service_user.role,
                api_key: new_api_key,  // Only shown once!
                created_at: updated_service_user.created_at,
                created_by: updated_service_user.created_by,
                expires_at: updated_service_user.expires_at,
                active: updated_service_user.active,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to rotate API key: {}", e)})),
        ).into_response(),
    }
}
