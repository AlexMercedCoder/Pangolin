use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
    Extension,
};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::model::SystemSettings;
use pangolin_core::user::UserSession;
use crate::iceberg::AppState;
use pangolin_core::user::UserRole;

/// Get system settings
#[utoipa::path(
    get,
    path = "/api/v1/config/settings",
    tag = "System Config",
    responses(
        (status = 200, description = "System settings", body = SystemSettings),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_system_settings(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
) -> impl IntoResponse {
    // Check if user is admin
    if !matches!(session.role, UserRole::Root | UserRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    let tenant_id = session.tenant_id.unwrap_or_default();
    match store.get_system_settings(tenant_id).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get settings: {}", e)).into_response(),
    }
}

/// Update system settings
#[utoipa::path(
    put,
    path = "/api/v1/config/settings",
    tag = "System Config",
    request_body = SystemSettings,
    responses(
        (status = 200, description = "Settings updated", body = SystemSettings),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_system_settings(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Json(settings): Json<SystemSettings>,
) -> impl IntoResponse {
    // Check if user is admin
    if !matches!(session.role, UserRole::Root | UserRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    let tenant_id = session.tenant_id.unwrap_or_default();
    match store.update_system_settings(tenant_id, settings).await {
        Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update settings: {}", e)).into_response(),
    }
}
