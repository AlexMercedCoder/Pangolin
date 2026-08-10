use crate::auth::Claims;
use crate::iceberg::AppState;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use pangolin_core::token::TokenInfo;
use pangolin_core::user::UserRole;
use pangolin_store::PaginationParams;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GenerateTokenRequest {
    pub tenant_id: String,
    pub username: Option<String>,
    pub roles: Option<Vec<String>>,
    pub expires_in_hours: Option<u64>,
}

/// Upper bound on a caller-requested token lifetime.
///
/// `expires_in_hours` is attacker-controlled and used to be fed straight into
/// `chrono::Duration::hours`, which *panics* on a large enough value - before
/// the `checked_add_signed().unwrap()` below it could even run (B0m). There is
/// no `CatchPanicLayer` under the router, so that panic aborted the connection
/// task. Clamping first makes the arithmetic total.
const MAX_TOKEN_LIFETIME_HOURS: u64 = 24 * 365;

/// Rank roles so a caller cannot mint a token more privileged than itself.
fn role_rank(role: &UserRole) -> u8 {
    match role {
        UserRole::Root => 3,
        UserRole::TenantAdmin => 2,
        UserRole::TenantUser => 1,
    }
}

/// Parse a role name from a token request.
///
/// Accepts both the Debug-ish spellings the old code matched and the kebab-case
/// serde names, but unknown values are now an error rather than a silent
/// downgrade to `TenantUser`.
fn parse_role(name: &str) -> Option<UserRole> {
    match name {
        "Root" | "root" => Some(UserRole::Root),
        "Admin" | "admin" | "TenantAdmin" | "tenant-admin" => Some(UserRole::TenantAdmin),
        "TenantUser" | "tenant-user" | "user" => Some(UserRole::TenantUser),
        _ => None,
    }
}

#[derive(Serialize, ToSchema)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub expires_at: String,
    pub tenant_id: String,
}

/// Generate a JWT token for a tenant.
///
/// Authorization (B0a): this handler used to take no session at all, so any
/// authenticated principal - including the lowest-privilege `TenantUser` or any
/// service-user API key - could POST `{"tenant_id": "<any>", "roles":["Root"]}`
/// and receive a signed `Root` token for an arbitrary tenant. That is a total
/// privilege escalation, since `check_permission` short-circuits for `Root`.
///
/// The rules now are:
///   * `Root` may mint anything.
///   * `TenantAdmin` may mint only for its own tenant, and only a role at or
///     below its own.
///   * everyone else is refused.
/// A role supplied in the body is never trusted for a non-`Root` caller beyond
/// those bounds.
#[utoipa::path(
    post,
    path = "/api/v1/tokens",
    tag = "Tokens",
    request_body = GenerateTokenRequest,
    responses(
        (status = 200, description = "Token generated", body = GenerateTokenResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn generate_token(
    State(store): State<AppState>,
    Extension(session): Extension<UserSession>,
    Json(payload): Json<GenerateTokenRequest>,
) -> impl IntoResponse {
    // Validate tenant_id is a valid UUID
    let tenant_uuid = match Uuid::parse_str(&payload.tenant_id) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid tenant_id format").into_response(),
    };

    let caller_is_root = session.role == UserRole::Root;
    if !caller_is_root {
        if session.role != UserRole::TenantAdmin {
            return (
                StatusCode::FORBIDDEN,
                "Root or tenant-admin access required to mint tokens",
            )
                .into_response();
        }
        if session.tenant_id != Some(tenant_uuid) {
            return (
                StatusCode::FORBIDDEN,
                "Cannot mint a token for another tenant",
            )
                .into_response();
        }
    }

    let secret = crate::config::jwt_secret();
    let expires_in = payload
        .expires_in_hours
        .unwrap_or(24)
        .min(MAX_TOKEN_LIFETIME_HOURS);
    let now = chrono::Utc::now();
    let Some(exp) = now
        .checked_add_signed(chrono::Duration::hours(expires_in as i64))
        .map(|t| t.timestamp())
    else {
        return (StatusCode::BAD_REQUEST, "expires_in_hours out of range").into_response();
    };

    let username = payload.username.unwrap_or_else(|| "api-user".to_string());

    // Map role strings to UserRole. An unknown name is now a 400 rather than a
    // silent downgrade, so a typo cannot quietly hand out the wrong role.
    let requested_role = if let Some(roles) = &payload.roles {
        match roles.first() {
            Some(first_role) => match parse_role(first_role) {
                Some(r) => Some(r),
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        format!("Unknown role: {}", first_role),
                    )
                        .into_response()
                }
            },
            None => None,
        }
    } else {
        None
    };

    let role = match requested_role {
        Some(r) => r,
        None => {
            // Try to look up the user's own role.
            if let Ok(Some(user)) = store.get_user_by_username(&username).await {
                tracing::info!(
                    "generate_token: Found user '{}' with role {:?} ({})",
                    username,
                    user.role,
                    user.id
                );
                user.role
            } else {
                tracing::warn!(
                    "generate_token: User '{}' not found, defaulting to TenantUser",
                    username
                );
                UserRole::TenantUser
            }
        }
    };

    // A non-root caller can never mint above its own rank, whatever the body or
    // the looked-up user says.
    if !caller_is_root && role_rank(&role) > role_rank(&session.role) {
        return (
            StatusCode::FORBIDDEN,
            "Cannot mint a token more privileged than the caller",
        )
            .into_response();
    }

    // sub MUST be a UUID for to_session() to work
    // If user exists, use their ID. Else generate one.
    let user_id = if let Ok(Some(user)) = store.get_user_by_username(&username).await {
        user.id
    } else {
        Uuid::new_v4()
    };

    let token_id = Uuid::new_v4();
    let claims = Claims {
        sub: user_id.to_string(),
        jti: Some(token_id.to_string()),
        username: username.clone(),
        tenant_id: Some(payload.tenant_id.clone()),
        role,
        exp,
        iat: now.timestamp(),
    };

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(token) => {
            // Store token info for listing
            let token_info = TokenInfo {
                id: token_id,
                tenant_id: tenant_uuid,
                user_id,
                username: username.clone(),
                expires_at: chrono::DateTime::from_timestamp(exp, 0).unwrap_or_default(),
                created_at: Utc::now(),
                is_valid: true,
                token: Some(token.clone()),
            };

            if let Err(e) = store.store_token(token_info).await {
                tracing::warn!("Failed to store token info: {}", e);
                // Continue, as returning the token is more important, but listing might be incomplete
            }

            let response = GenerateTokenResponse {
                token,
                expires_at: chrono::DateTime::from_timestamp(exp, 0)
                    .unwrap_or_default()
                    .to_rfc3339(),
                tenant_id: payload.tenant_id,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Token generation failed: {}", e),
        )
            .into_response(),
    }
}

// ===== Token Revocation Endpoints =====

use axum::extract::Path;
use axum::Extension;
use pangolin_core::user::UserSession;
use pangolin_store::CatalogStore;
use std::sync::Arc;

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
    // Revoke until the token would have expired anyway; the blacklist entry
    // only has to outlive the token.
    let expires_at = session.expires_at.max(Utc::now());

    // B0j: revoke the token's own `jti`. This used to revoke `session.user_id`,
    // which no token ever carries as its `jti`, so the middleware's revocation
    // check (keyed by `jti`) never matched: logout returned 200 and the token
    // stayed valid for its full lifetime.
    let Some(token_id) = session.token_id else {
        // API-key and root-basic-auth sessions have no revocable JWT.
        return (
            StatusCode::BAD_REQUEST,
            "This session is not backed by a revocable token",
        )
            .into_response();
    };

    match store
        .revoke_token(token_id, expires_at, payload.reason)
        .await
    {
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

    match store
        .revoke_token(token_id, expires_at, payload.reason)
        .await
    {
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
/// Admin endpoint to clean up expired tokens
#[utoipa::path(
    post,
    path = "/api/v1/auth/tokens/cleanup",
    tag = "Tokens",
    responses(
        (status = 200, description = "Cleaned up expired tokens", body = serde_json::Value),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
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
            tracing::info!(
                "Cleaned up {} expired tokens by admin: {}",
                count,
                session.username
            );
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

// ===== New Endpoints for Token Management =====

/// List tokens for current user
#[utoipa::path(
    get,
    path = "/api/v1/users/me/tokens",
    tag = "Tokens",
    responses(
        (status = 200, description = "List of active tokens", body = Vec<TokenInfo>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_my_tokens(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    let tenant_id = session.tenant_id.unwrap_or_default();
    match store
        .list_active_tokens(tenant_id, Some(session.user_id), Some(pagination))
        .await
    {
        Ok(tokens) => (StatusCode::OK, Json(tokens)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list tokens: {}", e),
        )
            .into_response(),
    }
}

/// List tokens for a specific user (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/tokens",
    tag = "Tokens",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "List of active tokens", body = Vec<TokenInfo>),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_user_tokens(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(target_user_id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    if !matches!(session.role, UserRole::Root | UserRole::TenantAdmin) {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    let tenant_id = session.tenant_id.unwrap_or_default();
    match store
        .list_active_tokens(tenant_id, Some(target_user_id), Some(pagination))
        .await
    {
        Ok(tokens) => (StatusCode::OK, Json(tokens)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list tokens: {}", e),
        )
            .into_response(),
    }
}

/// Delete (Revoke) a token by ID
#[utoipa::path(
    delete,
    path = "/api/v1/tokens/{token_id}",
    tag = "Tokens",
    params(
        ("token_id" = Uuid, Path, description = "Token ID to revoke")
    ),
    responses(
        (status = 204, description = "Token revoked"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_token(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(token_id): Path<Uuid>,
) -> impl IntoResponse {
    // Determine target token's owner to allow self-revocation (complex without DB lookup)
    // For now, allow if Admin OR if we can verify ownership?
    // Simplified: Admin only for DELETE /tokens/{id}, user uses /auth/revoke (logout).
    // OR: We try to list users tokens and see if it's there?
    // Let's enforce Admin for arbitrary ID revocation for safety, unless it's their own token.

    let is_admin = matches!(session.role, UserRole::Root | UserRole::TenantAdmin);

    // Ideally we should check if the token belongs to the user, but we don't have easy lookup from ID -> User without scanning active_tokens.
    // However, `revoke_token` just adds to blacklist.

    // Let's stick to Admin check for now for this specific endpoint.
    if !is_admin {
        // Allow if it matches current session token?
        // We can't easily check if token_id belongs to user without a lookup handler.
        return (
            StatusCode::FORBIDDEN,
            "Admin access required to delete arbitrary token",
        )
            .into_response();
    }

    let expires_at = Utc::now() + Duration::hours(24);
    match store
        .revoke_token(token_id, expires_at, Some("Deleted via API".to_string()))
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to revoke token: {}", e),
        )
            .into_response(),
    }
}

/// Rotate current token
#[utoipa::path(
    post,
    path = "/api/v1/tokens/rotate",
    tag = "Tokens",
    responses(
        (status = 200, description = "Token rotated", body = GenerateTokenResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn rotate_token(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
) -> impl IntoResponse {
    // 1. Generate new token
    // We need to construct a GenerateTokenRequest or reuse logic.
    // Reusing logic is better to avoid duplication but `generate_token` takes a Json payload.
    // We'll reimplement specific logic here for rotation.

    let secret = crate::config::jwt_secret();
    let now = chrono::Utc::now();
    let expires_in = 24; // Default rotation to 24h
    let Some(exp) = now
        .checked_add_signed(chrono::Duration::hours(expires_in))
        .map(|t| t.timestamp())
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to compute token expiry",
        )
            .into_response();
    };

    let token_id = Uuid::new_v4();
    let tenant_id_str = session.tenant_id.map(|t| t.to_string()).unwrap_or_default();

    let claims = Claims {
        sub: session.user_id.to_string(),
        jti: Some(token_id.to_string()),
        username: session.username.clone(),
        tenant_id: Some(tenant_id_str.clone()),
        role: session.role.clone(),
        exp,
        iat: now.timestamp(),
    };

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(token) => {
            // 2. Store new token
            let token_info = TokenInfo {
                id: token_id,
                tenant_id: session.tenant_id.unwrap_or_default(),
                user_id: session.user_id,
                username: session.username.clone(),
                expires_at: chrono::DateTime::from_timestamp(exp, 0).unwrap_or_default(),
                created_at: Utc::now(),
                is_valid: true,
                token: Some(token.clone()),
            };

            if let Err(e) = store.store_token(token_info).await {
                tracing::warn!("Failed to store rotated token info: {}", e);
            }

            // 3. Revoke the old token. `UserSession` now carries the presenting
            // token's `jti` (B0j), so rotation is a real rotation rather than
            // "issue a second valid token and hope the client forgets the first".
            if let Some(old_token_id) = session.token_id {
                if let Err(e) = store
                    .revoke_token(
                        old_token_id,
                        session.expires_at.max(Utc::now()),
                        Some("Rotated".to_string()),
                    )
                    .await
                {
                    tracing::error!(error = %e, "failed to revoke the rotated-out token");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Issued a new token but could not revoke the old one",
                    )
                        .into_response();
                }
            }

            let response = GenerateTokenResponse {
                token,
                expires_at: chrono::DateTime::from_timestamp(exp, 0)
                    .unwrap_or_default()
                    .to_rfc3339(),
                tenant_id: tenant_id_str,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Token rotation failed: {}", e),
        )
            .into_response(),
    }
}
