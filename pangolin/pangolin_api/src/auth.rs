use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

use base64::{Engine as _, engine::general_purpose::STANDARD};

#[derive(Clone, Copy, Debug)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    Root,
    Admin,
    User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub tenant_id: Option<String>,
    pub roles: Vec<Role>,
}

#[derive(Clone, Copy, Debug)]
pub struct RootUser;

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Whitelist: Allow public access to config endpoints (Iceberg REST spec requirement)
    let path = request.uri().path().to_string();
    if path == "/v1/config" || path.ends_with("/config") {
        return Ok(next.run(request).await);
    }
    
    // 0. Check for Root User (Basic Auth)
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                let token = &auth_str[6..];
                if let Ok(decoded) = STANDARD.decode(token) {
                    if let Ok(cred_str) = String::from_utf8(decoded) {
                        if let Some((username, password)) = cred_str.split_once(':') {
                            let root_user = std::env::var("PANGOLIN_ROOT_USER").unwrap_or_default();
                            let root_pass = std::env::var("PANGOLIN_ROOT_PASSWORD").unwrap_or_default();
                            
                            if !root_user.is_empty() && username == root_user && password == root_pass {
                                request.extensions_mut().insert(RootUser);
                                request.extensions_mut().insert(vec![Role::Root]);
                            }
                        }
                    }
                }
            }
        }
    }

    // 1. Check for Authorization Header (Bearer Token)
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                
                let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
                let mut validation = Validation::new(Algorithm::HS256);
                
                match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
                    Ok(token_data) => {
                        if let Some(tid_str) = token_data.claims.tenant_id {
                            if let Ok(tid) = Uuid::parse_str(&tid_str) {
                                request.extensions_mut().insert(TenantId(tid));
                            }
                        }
                        request.extensions_mut().insert(token_data.claims.roles);
                        return Ok(next.run(request).await);
                    },
                    Err(_) => {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            }
        }
    }

    // 2. Fallback: Check for X-Pangolin-Tenant header (Dev/Legacy)
    tracing::debug!("Checking X-Pangolin-Tenant header for path: {}", path);
    if let Some(tenant_header) = headers.get("X-Pangolin-Tenant") {
        tracing::debug!("Found X-Pangolin-Tenant header: {:?}", tenant_header);
        if let Ok(tenant_str) = tenant_header.to_str() {
            tracing::debug!("Parsed tenant header value: {}", tenant_str);
            if let Ok(tenant_uuid) = Uuid::parse_str(tenant_str) {
                tracing::info!("Successfully authenticated with tenant: {}", tenant_uuid);
                request.extensions_mut().insert(TenantId(tenant_uuid));
                return Ok(next.run(request).await);
            } else {
                tracing::warn!("Failed to parse tenant UUID from: {}", tenant_str);
            }
        } else {
            tracing::warn!("Failed to convert tenant header to string");
        }
    } else {
        tracing::debug!("No X-Pangolin-Tenant header found. Available headers: {:?}", headers.keys().collect::<Vec<_>>());
    }
    
    // 3. Fallback: Nil Tenant (Dev only, warn)
    // Only use nil tenant if NOT a root user. If root user, they might not need a tenant for global ops.
    if request.extensions().get::<RootUser>().is_none() && request.extensions().get::<TenantId>().is_none() {
        tracing::warn!("No Auth or Tenant header found, using nil tenant.");
        request.extensions_mut().insert(TenantId(Uuid::nil()));
    }

    Ok(next.run(request).await)
}
