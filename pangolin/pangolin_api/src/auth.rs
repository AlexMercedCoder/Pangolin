use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Subject (User ID)
    exp: usize,
    // Custom claims
    tenant_id: Option<String>,
    roles: Option<Vec<String>>,
}

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Check for Authorization Header (Bearer Token)
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                
                // For MVP, we use a hardcoded secret or env var.
                // In production, this should be loaded from config/KMS.
                let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
                
                let mut validation = Validation::new(Algorithm::HS256);
                // Disable exp check for dev if needed, but better to keep it.
                
                match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
                    Ok(token_data) => {
                        // If token has tenant_id, use it.
                        if let Some(tid_str) = token_data.claims.tenant_id {
                            if let Ok(tid) = Uuid::parse_str(&tid_str) {
                                request.extensions_mut().insert(TenantId(tid));
                                return Ok(next.run(request).await);
                            }
                        }
                    },
                    Err(_) => {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            }
        }
    }

    // 2. Fallback: Check for X-Pangolin-Tenant header (Dev/Legacy)
    if let Some(tenant_header) = headers.get("X-Pangolin-Tenant") {
        if let Ok(tenant_str) = tenant_header.to_str() {
            if let Ok(tenant_uuid) = Uuid::parse_str(tenant_str) {
                request.extensions_mut().insert(TenantId(tenant_uuid));
                return Ok(next.run(request).await);
            }
        }
    }
    
    // 3. Fallback: Nil Tenant (Dev only, warn)
    tracing::warn!("No Auth or Tenant header found, using nil tenant.");
    request.extensions_mut().insert(TenantId(Uuid::nil()));
    Ok(next.run(request).await)
}
