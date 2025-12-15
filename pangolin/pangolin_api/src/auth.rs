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
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    
    // Check if NO_AUTH mode is enabled (must be exactly "true" for security)
    let no_auth_enabled = std::env::var("PANGOLIN_NO_AUTH")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    if no_auth_enabled {
        // No-auth mode: bypass all authentication and use default tenant
        let default_tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")
            .expect("Failed to parse default tenant UUID");
        tracing::debug!("Running in NO_AUTH mode, using default tenant: {}", default_tenant_id);
        request.extensions_mut().insert(TenantId(default_tenant_id));
        return Ok(next.run(request).await);
    }

    let headers = request.headers();
    
    // Whitelist: Allow /v1/config and /v1/:prefix/config without auth
    if path == "/v1/config" || path.ends_with("/config") {
        tracing::debug!("Allowing unauthenticated access to config endpoint: {}", path);
        return Ok(next.run(request).await);
    }
    
    // 1. Check for Basic Auth (root user)
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
                                return Ok(next.run(request).await);
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
                        let mut final_tenant_id: Option<Uuid> = None;
                        
                        // 1. From Token
                        if let Some(tid_str) = token_data.claims.tenant_id {
                            if let Ok(tid) = Uuid::parse_str(&tid_str) {
                                final_tenant_id = Some(tid);
                            }
                        }

                        // 2. From Header (Override)
                        // Allows Root users to switch context
                        if let Some(tenant_header) = headers.get("X-Pangolin-Tenant") {
                            if let Ok(tenant_str) = tenant_header.to_str() {
                                if let Ok(tenant_uuid) = Uuid::parse_str(tenant_str) {
                                    final_tenant_id = Some(tenant_uuid);
                                }
                            }
                        }

                        if let Some(tid) = final_tenant_id {
                             request.extensions_mut().insert(TenantId(tid));
                        }
                        
                        // If token has Root role, also insert RootUser extension
                        if token_data.claims.roles.contains(&Role::Root) {
                            request.extensions_mut().insert(RootUser);
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
    tracing::debug!("All headers: {:?}", headers);
    
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
    
    // 3. Fallback: No Auth provided
    if request.extensions().get::<RootUser>().is_none() && request.extensions().get::<TenantId>().is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
