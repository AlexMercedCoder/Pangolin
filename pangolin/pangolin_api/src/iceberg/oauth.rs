use axum::{
    extract::{State, Form},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use pangolin_store::CatalogStore;
// Removed Signer import as we implement signing locally
use pangolin_core::user::{User, UserRole, UserSession, ServiceUser};
use chrono::{Utc, Duration};
use anyhow::Context;
use uuid::Uuid;
use bcrypt::verify;
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::auth::Claims;

// Internal imports
use crate::error::ApiError;
use crate::iceberg::AppState;

#[derive(Deserialize)]
pub struct OAuthTokenRequest {
    grant_type: String,
    client_id: String,
    client_secret: String,
    scope: Option<String>,
}

#[derive(Serialize)]
pub struct OAuthTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    issued_token_type: String,
}

/// Handler for the standard OAuth2 client_credentials flow
/// 
/// This endpoint accepts `application/x-www-form-urlencoded` data
/// to support standard libraries like PyIceberg/REST Catalog.
/// 
/// It maps:
/// - `client_id` -> Service User ID (UUID)
/// - `client_secret` -> Service User API Key
/// 
/// If valid, it returns a standard Pangolin JWT signed by the server key.
pub async fn handle_oauth_token(
    State(store): State<AppState>,
    Form(payload): Form<OAuthTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Validate Grant Type
    if payload.grant_type != "client_credentials" {
        return Err(ApiError::bad_request("Unsupported grant_type. exact 'client_credentials' required."));
    }

    // 2. Parse Client ID as UUID
    let service_user_id = Uuid::parse_str(&payload.client_id)
        .map_err(|_| ApiError::bad_request("Invalid client_id format. Must be a valid UUID."))?;

    // 3. Retrieve Service User
    let store_ref = &*store;
    let service_user_result: anyhow::Result<Option<ServiceUser>> = store_ref.get_service_user(service_user_id).await;
    let service_user = service_user_result
        .map_err(|e| ApiError::InternalError(e))?
        .ok_or_else(|| ApiError::unauthorized("Invalid client_id"))?;

    // 4. Verify Active Status
    if !service_user.active {
        return Err(ApiError::unauthorized("Client is inactive"));
    }

    // 5. Verify Secret (API Key)
    // The stored hash is bcrypt
    let valid_secret = verify(&payload.client_secret, &service_user.api_key_hash)
        .map_err(|e| ApiError::InternalError(anyhow::anyhow!("Crypto failure: {}", e)))?;

    if !valid_secret {
        return Err(ApiError::unauthorized("Invalid client_secret"));
    }
    
    // 6. Generate Session/Token
    // We reuse the standard JWT generation logic used for users
    let now = Utc::now();
    let expires_in_seconds = 3600; // 1 hour default
    let expires_at = now + Duration::seconds(expires_in_seconds as i64);

    let token_id = Uuid::new_v4();
    let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "default_secret_for_dev".to_string());
    
    let claims = Claims {
        sub: service_user.id.to_string(), 
        jti: Some(token_id.to_string()),
        username: service_user.name.clone(), 
        tenant_id: Some(service_user.tenant_id.to_string()),
        role: service_user.role.clone(), 
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    // We need to sign this. 
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| ApiError::InternalError(anyhow::anyhow!("Token generation failed: {}", e)))?;
        
    // 7. Update Last Used
    // Best effort - don't fail auth if this fails
    let _ = store_ref.update_service_user_last_used(service_user.id, now).await;

    // 8. Return Response
    Ok(Json(OAuthTokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: expires_in_seconds,
        issued_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
    }))
}
