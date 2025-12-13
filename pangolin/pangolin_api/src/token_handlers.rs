use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;
use crate::auth::{Claims, Role};
use crate::iceberg_handlers::AppState;

#[derive(Deserialize)]
pub struct GenerateTokenRequest {
    pub tenant_id: String,
    pub username: Option<String>,
    pub roles: Option<Vec<Role>>,
    pub expires_in_hours: Option<u64>,
}

#[derive(Serialize)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub expires_at: String,
    pub tenant_id: String,
}

/// Generate a JWT token for a tenant
/// This endpoint allows generating tokens for testing and development
pub async fn generate_token(
    State(_store): State<AppState>,
    Json(payload): Json<GenerateTokenRequest>,
) -> impl IntoResponse {
    // Validate tenant_id is a valid UUID
    let tenant_uuid = match Uuid::parse_str(&payload.tenant_id) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid tenant_id format").into_response(),
    };
    
    let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let expires_in = payload.expires_in_hours.unwrap_or(24);
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(expires_in as i64))
        .unwrap()
        .timestamp() as usize;
    
    let claims = Claims {
        sub: payload.username.unwrap_or_else(|| "api-user".to_string()),
        exp,
        tenant_id: Some(payload.tenant_id.clone()),
        roles: payload.roles.unwrap_or_else(|| vec![Role::User]),
    };
    
    match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())) {
        Ok(token) => {
            let response = GenerateTokenResponse {
                token,
                expires_at: chrono::DateTime::from_timestamp(exp as i64, 0)
                    .unwrap()
                    .to_rfc3339(),
                tenant_id: payload.tenant_id,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Token generation failed: {}", e)).into_response(),
    }
}
