use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use pangolin_store::CatalogStore;
use bcrypt::verify;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};
use pangolin_core::user::{UserRole, UserSession};
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// JWT claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub tenant_id: Option<String>,
    pub role: UserRole,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
}

impl From<UserSession> for Claims {
    fn from(session: UserSession) -> Self {
        Self {
            sub: session.user_id.to_string(),
            username: session.username,
            tenant_id: session.tenant_id.map(|id| id.to_string()),
            role: session.role,
            exp: session.expires_at.timestamp(),
            iat: session.issued_at.timestamp(),
        }
    }
}

impl Claims {
    pub fn to_session(&self) -> Result<UserSession, String> {
        Ok(UserSession {
            user_id: Uuid::parse_str(&self.sub).map_err(|e| e.to_string())?,
            username: self.username.clone(),
            tenant_id: self.tenant_id.as_ref()
                .map(|id| Uuid::parse_str(id))
                .transpose()
                .map_err(|e| e.to_string())?,
            role: self.role.clone(),
            issued_at: DateTime::from_timestamp(self.iat, 0)
                .ok_or("Invalid issued_at timestamp")?,
            expires_at: DateTime::from_timestamp(self.exp, 0)
                .ok_or("Invalid expires_at timestamp")?,
        })
    }
}

/// Generate JWT token for user session
pub fn generate_token(session: UserSession, secret: &str) -> Result<String, String> {
    let claims = Claims::from(session);
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to generate token: {}", e))
}

/// Verify and decode JWT token
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}

/// Create user session with expiration
pub fn create_session(
    user_id: Uuid,
    username: String,
    tenant_id: Option<Uuid>,
    role: UserRole,
    expiration_secs: u64,
) -> UserSession {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(expiration_secs as i64);
    
    UserSession {
        user_id,
        username,
        tenant_id,
        role,
        issued_at: now,
        expires_at,
    }
}

/// Axum middleware to extract and verify JWT token
pub async fn auth_middleware(
    State(store): State<Arc<dyn CatalogStore>>,
    mut req: Request,
    next: Next,
) -> Response {
    // Check if NO_AUTH mode is enabled
    if std::env::var("PANGOLIN_NO_AUTH").is_ok() {
        // In NO_AUTH mode, create a default root user session
        let session = create_session(
            Uuid::nil(),
            "root".to_string(),
            None,
            UserRole::Root,
            86400,
        );
        req.extensions_mut().insert(session);
        // Also insert TenantId for NO_AUTH mode (default tenant)
        let default_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        req.extensions_mut().insert(crate::auth::TenantId(default_tenant));
        // In NO_AUTH mode, we are effectively root
        req.extensions_mut().insert(crate::auth::RootUser);
        req.extensions_mut().insert(crate::auth::RootUser);
        return next.run(req).await;
    }

    // Check for X-API-Key header (Service User authentication)
    if let Some(api_key_header) = req.headers().get("X-API-Key") {
        if let Ok(api_key) = api_key_header.to_str() {
            // We need to iterate through all service users and verify the API key
            // This is not ideal for performance but works for MVP
            // In production, consider caching or indexing strategies
            
            // Get all tenants and check their service users
            if let Ok(tenants) = store.list_tenants().await {
                for tenant in tenants {
                    if let Ok(service_users) = store.list_service_users(tenant.id).await {
                        for service_user in service_users {
                            // Verify the API key against the stored hash
                            if let Ok(true) = verify(api_key, &service_user.api_key_hash) {
                                // Check if service user is valid (active and not expired)
                                if service_user.is_valid() {
                                    // Create session from service user
                                    let session = create_session(
                                        service_user.id,
                                        service_user.name.clone(),
                                        Some(service_user.tenant_id),
                                        service_user.role.clone(),
                                        86400, // 24 hour session
                                    );
                                    req.extensions_mut().insert(session);
                                    req.extensions_mut().insert(crate::auth::TenantId(service_user.tenant_id));
                                    
                                    // Update last_used timestamp (fire and forget)
                                    let store_clone = store.clone();
                                    let service_user_id = service_user.id;
                                    tokio::spawn(async move {
                                        let _ = store_clone.update_service_user_last_used(service_user_id, Utc::now()).await;
                                    });
                                    
                                    return next.run(req).await;
                                }
                            }
                        }
                    }
                }
            }
            
            // If we get here, API key was invalid
            return (StatusCode::UNAUTHORIZED, "Invalid or expired API key").into_response();
        }
    }

    // Whitelist public endpoints
    let path = req.uri().path();
    if path == "/api/v1/users/login" || 
       path == "/api/v1/app-config" || 
       path == "/v1/config" || 
       path.ends_with("/config") ||
       path.starts_with("/oauth/authorize/") ||
       path.starts_with("/oauth/callback/") {
            return next.run(req).await;
    }
    
    // Extract token from Authorization header or check for Basic Auth
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    
    // Check for Basic Auth first
    if let Some(header_val) = auth_header {
        if header_val.starts_with("Basic ") {
            let token = &header_val[6..];
            if let Ok(decoded) = STANDARD.decode(token) {
                if let Ok(cred_str) = String::from_utf8(decoded) {
                    if let Some((username, password)) = cred_str.split_once(':') {
                        let root_user = std::env::var("PANGOLIN_ROOT_USER").unwrap_or_default();
                        let root_pass = std::env::var("PANGOLIN_ROOT_PASSWORD").unwrap_or_default();
                        
                        if !root_user.is_empty() && username == root_user && password == root_pass {
                             // Create a root session
                            let session = create_session(
                                Uuid::nil(),
                                "root".to_string(),
                                None,
                                UserRole::Root,
                                3600, // 1 hour session for root ops
                            );
                            req.extensions_mut().insert(session);
                            
                            // Insert default tenant ID for root
                            let default_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
                            req.extensions_mut().insert(crate::auth::TenantId(default_tenant));
                            
                            // Insert RootUser extension
                            req.extensions_mut().insert(crate::auth::RootUser);
                            
                            return next.run(req).await;
                        }
                    }
                }
            }
        }
    }

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            &header[7..]
        }
        _ => {
            return (StatusCode::UNAUTHORIZED, "Missing or invalid authorization header").into_response();
        }
    };
    
    // Get JWT secret from environment
    let secret = match std::env::var("PANGOLIN_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not configured").into_response();
        }
    };
    
    // Verify token
    let claims = match verify_token(token, &secret) {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)).into_response();
        }
    };
    
    // Convert claims to session
    let session = match claims.to_session() {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, format!("Invalid session: {}", e)).into_response();
        }
    };
    
    // Check if token is expired
    if session.expires_at < Utc::now() {
        return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
    }
    
    // Add session to request extensions
    req.extensions_mut().insert(session.clone());

    // Inject TenantId for handlers that require it (like iceberg_handlers)
    // If session has a tenant_id, use it. Otherwise default to the nil UUID (system/root)
    let tenant_uuid = session.tenant_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    });
    req.extensions_mut().insert(crate::auth::TenantId(tenant_uuid));
    
    // If role is Root, insert RootUser extension
    if session.role == UserRole::Root {
        req.extensions_mut().insert(crate::auth::RootUser);
    }
    
    next.run(req).await
}

/// Wrapper for middleware that doesn't require state (for backward compatibility)
pub async fn auth_middleware_wrapper(
    mut req: Request,
    next: Next,
) -> Response {
    // For routes that don't have service user support, use the old middleware
    // This is a temporary solution - ideally all routes should use the new middleware
    
    // Check if NO_AUTH mode is enabled
    if std::env::var("PANGOLIN_NO_AUTH").is_ok() {
        let session = create_session(
            Uuid::nil(),
            "root".to_string(),
            None,
            UserRole::Root,
            86400,
        );
        req.extensions_mut().insert(session);
        let default_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        req.extensions_mut().insert(crate::auth::TenantId(default_tenant));
        req.extensions_mut().insert(crate::auth::RootUser);
        return next.run(req).await;
    }

    // Whitelist public endpoints
    let path = req.uri().path();
    if path == "/api/v1/users/login" || 
       path == "/api/v1/app-config" || 
       path == "/v1/config" || 
       path.ends_with("/config") ||
       path.starts_with("/oauth/authorize/") ||
       path.starts_with("/oauth/callback/") {
            return next.run(req).await;
    }
    
    // Extract token from Authorization header or check for Basic Auth
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    
    // Check for Basic Auth first
    if let Some(header_val) = auth_header {
        if header_val.starts_with("Basic ") {
            let token = &header_val[6..];
            if let Ok(decoded) = STANDARD.decode(token) {
                if let Ok(cred_str) = String::from_utf8(decoded) {
                    if let Some((username, password)) = cred_str.split_once(':') {
                        let root_user = std::env::var("PANGOLIN_ROOT_USER").unwrap_or_default();
                        let root_pass = std::env::var("PANGOLIN_ROOT_PASSWORD").unwrap_or_default();
                        
                        if !root_user.is_empty() && username == root_user && password == root_pass {
                            let session = create_session(
                                Uuid::nil(),
                                "root".to_string(),
                                None,
                                UserRole::Root,
                                3600,
                            );
                            req.extensions_mut().insert(session);
                            let default_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
                            req.extensions_mut().insert(crate::auth::TenantId(default_tenant));
                            req.extensions_mut().insert(crate::auth::RootUser);
                            return next.run(req).await;
                        }
                    }
                }
            }
        }
    }

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            &header[7..]
        }
        _ => {
            return (StatusCode::UNAUTHORIZED, "Missing or invalid authorization header").into_response();
        }
    };
    
    let secret = match std::env::var("PANGOLIN_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not configured").into_response();
        }
    };
    
    let claims = match verify_token(token, &secret) {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)).into_response();
        }
    };
    
    let session = match claims.to_session() {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, format!("Invalid session: {}", e)).into_response();
        }
    };
    
    if session.expires_at < Utc::now() {
        return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
    }
    
    req.extensions_mut().insert(session.clone());
    let tenant_uuid = session.tenant_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    });
    req.extensions_mut().insert(crate::auth::TenantId(tenant_uuid));
    
    if session.role == UserRole::Root {
        req.extensions_mut().insert(crate::auth::RootUser);
    }
    
    next.run(req).await
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash)
        .map_err(|e| format!("Failed to verify password: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let secret = "test_secret_key_12345";
        let session = create_session(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            UserRole::Root,
            3600,
        );
        
        let token = generate_token(session.clone(), secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        let decoded_session = claims.to_session().unwrap();
        
        assert_eq!(decoded_session.user_id, session.user_id);
        assert_eq!(decoded_session.username, session.username);
        assert_eq!(decoded_session.role, session.role);
    }

    #[test]
    fn test_jwt_invalid_secret() {
        let secret = "test_secret_key_12345";
        let wrong_secret = "wrong_secret_key";
        let session = create_session(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            UserRole::Root,
            3600,
        );
        
        let token = generate_token(session, secret).unwrap();
        assert!(verify_token(&token, wrong_secret).is_err());
    }

    #[test]
    fn test_session_creation() {
        let session = create_session(
            Uuid::new_v4(),
            "testuser".to_string(),
            Some(Uuid::new_v4()),
            UserRole::TenantAdmin,
            7200,
        );
        
        assert_eq!(session.username, "testuser");
        assert_eq!(session.role, UserRole::TenantAdmin);
        assert!(session.tenant_id.is_some());
        
        let duration = session.expires_at - session.issued_at;
        assert_eq!(duration.num_seconds(), 7200);
    }
}
