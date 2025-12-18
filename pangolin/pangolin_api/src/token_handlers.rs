use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;
use crate::auth::Claims;
use crate::iceberg_handlers::AppState;
use pangolin_core::user::UserRole;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GenerateTokenRequest {
    pub tenant_id: String,
    pub username: Option<String>,
    pub roles: Option<Vec<String>>,
    pub expires_in_hours: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub expires_at: String,
    pub tenant_id: String,
}

/// Generate a JWT token for a tenant
/// This endpoint allows generating tokens for testing and development
#[utoipa::path(
    post,
    path = "/api/v1/tokens",
    tag = "Tokens",
    request_body = GenerateTokenRequest,
    responses(
        (status = 200, description = "Token generated", body = GenerateTokenResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn generate_token(
    State(store): State<AppState>,
    Json(payload): Json<GenerateTokenRequest>,
) -> impl IntoResponse {
    // Validate tenant_id is a valid UUID
    let _tenant_uuid = match Uuid::parse_str(&payload.tenant_id) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid tenant_id format").into_response(),
    };
    
    let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "default_secret_for_dev".to_string());
    let expires_in = payload.expires_in_hours.unwrap_or(24);
    let now = chrono::Utc::now();
    let exp = now
        .checked_add_signed(chrono::Duration::hours(expires_in as i64))
        .unwrap()
        .timestamp() as i64;
    
    let username = payload.username.unwrap_or_else(|| "api-user".to_string());

    // Map role strings to UserRole
    // Default to lookup user role or TenantUser if not specified
    let role = if let Some(roles) = &payload.roles {
        if let Some(first_role) = roles.first() {
            match first_role.as_str() {
                "Root" | "root" => UserRole::Root,
                "Admin" | "admin" | "TenantAdmin" | "tenant-admin" => UserRole::TenantAdmin,
                _ => UserRole::TenantUser,
            }
        } else {
            UserRole::TenantUser
        }
    } else {
        // Try to lookup user
        if let Ok(Some(user)) = store.get_user_by_username(&username).await {
            tracing::info!("generate_token: Found user '{}' with role {:?} ({})", username, user.role, user.id);
            user.role
        } else {
            tracing::warn!("generate_token: User '{}' not found, defaulting to TenantUser", username);
            UserRole::TenantUser
        }
    };

    // sub MUST be a UUID for to_session() to work
    // If user exists, use their ID. Else generate one.
    let user_id = if let Ok(Some(user)) = store.get_user_by_username(&username).await {
        user.id
    } else {
        Uuid::new_v4()
    };

    let claims = Claims {
        sub: user_id.to_string(), 
        jti: Some(Uuid::new_v4().to_string()),
        username: username.clone(), 
        tenant_id: Some(payload.tenant_id.clone()),
        role,
        exp,
        iat: now.timestamp(),
    };
    
    match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())) {
        Ok(token) => {
            let response = GenerateTokenResponse {
                token,
                expires_at: chrono::DateTime::from_timestamp(exp, 0)
                    .unwrap()
                    .to_rfc3339(),
                tenant_id: payload.tenant_id,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Token generation failed: {}", e)).into_response(),
    }
}

// ===== Token Revocation Endpoints =====

use axum::extract::Path;
use axum::Extension;
use pangolin_core::user::UserSession;
use pangolin_store::CatalogStore;
use std::sync::Arc;
use chrono::{Utc, Duration};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RevokeTokenRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RevokeTokenResponse {
    pub message: String,
}

/// Revoke the current user's token (logout)
#[utoipa::path(
    post,
    path = "/api/v1/auth/revoke",
    tag = "Tokens",
    request_body = RevokeTokenRequest,
    responses(
        (status = 200, description = "Token revoked", body = RevokeTokenResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn revoke_current_token(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<RevokeTokenRequest>,
) -> impl IntoResponse {
    // Generate an expiration time (tokens typically expire in 24 hours)
    let expires_at = Utc::now() + Duration::hours(24);
    
    // Use the user_id as the token_id for revocation
    let token_id = session.user_id;
    
    match store.revoke_token(token_id, expires_at, payload.reason).await {
        Ok(_) => {
            tracing::info!("Token revoked for user: {}", session.username);
            (
                StatusCode::OK,
                Json(RevokeTokenResponse {
                    message: "Token revoked successfully. Please log in again.".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to revoke token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to revoke token: {}", e),
            )
                .into_response()
        }
    }
}

/// Admin endpoint to revoke any token by ID
#[utoipa::path(
    post,
    path = "/api/v1/auth/revoke/{token_id}",
    tag = "Tokens",
    params(
        ("token_id" = Uuid, Path, description = "Token ID to revoke")
    ),
    request_body = RevokeTokenRequest,
    responses(
        (status = 200, description = "Token revoked", body = RevokeTokenResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn revoke_token_by_id(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(token_id): Path<Uuid>,
    Json(payload): Json<RevokeTokenRequest>,
) -> impl IntoResponse {
    // Check if user is admin
    if !matches!(session.role, UserRole::Root | UserRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }
    
    // Set a default expiration (tokens typically expire in 24h)
    let expires_at = Utc::now() + Duration::hours(24);
    
    match store.revoke_token(token_id, expires_at, payload.reason).await {
        Ok(_) => {
            tracing::info!("Token {} revoked by admin: {}", token_id, session.username);
            (
                StatusCode::OK,
                Json(RevokeTokenResponse {
                    message: format!("Token {} revoked successfully", token_id),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to revoke token {}: {}", token_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to revoke token: {}", e),
            )
                .into_response()
        }
    }
}

/// Admin endpoint to clean up expired tokens
pub async fn cleanup_expired_tokens(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
) -> impl IntoResponse {
    // Check if user is admin
    if !matches!(session.role, UserRole::Root | UserRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }
    
    match store.cleanup_expired_tokens().await {
        Ok(count) => {
            tracing::info!("Cleaned up {} expired tokens by admin: {}", count, session.username);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": format!("Cleaned up {} expired tokens", count),
                    "count": count
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to cleanup expired tokens: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to cleanup expired tokens: {}", e),
            )
                .into_response()
        }
    }
}
