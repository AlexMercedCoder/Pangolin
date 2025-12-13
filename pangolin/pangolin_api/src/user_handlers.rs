use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_store::CatalogStore;
use std::sync::Arc;

/// Request to create a new user
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
}

/// Request to update a user
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub active: Option<bool>,
}

/// User login request
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response with JWT token
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: String,
}

/// Public user information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
    pub oauth_provider: Option<OAuthProvider>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            tenant_id: user.tenant_id,
            role: user.role,
            oauth_provider: user.oauth_provider,
        }
    }
}

/// Create a new user
pub async fn create_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Json(req): Json<CreateUserRequest>,
) -> Response {
    // TODO: Check permissions - only root or tenant admin can create users
    
    // Hash password if provided
    let password_hash = if let Some(password) = req.password {
        // TODO: Use bcrypt to hash password
        Some(password) // Placeholder
    } else {
        None
    };
    
    let user = match req.role {
        UserRole::Root => {
            if password_hash.is_none() {
                return (StatusCode::BAD_REQUEST, "Root user requires password").into_response();
            }
            User::new_root(req.username, req.email, password_hash.unwrap())
        }
        UserRole::TenantAdmin => {
            let tenant_id = req.tenant_id.ok_or("Tenant admin requires tenant_id").unwrap();
            if password_hash.is_none() {
                return (StatusCode::BAD_REQUEST, "Tenant admin requires password").into_response();
            }
            User::new_tenant_admin(req.username, req.email, password_hash.unwrap(), tenant_id)
        }
        UserRole::TenantUser => {
            let tenant_id = req.tenant_id.ok_or("Tenant user requires tenant_id").unwrap();
            if password_hash.is_none() {
                return (StatusCode::BAD_REQUEST, "Tenant user requires password").into_response();
            }
            User::new_tenant_user(req.username, req.email, password_hash.unwrap(), tenant_id)
        }
    };
    
    // TODO: Store user in database
    // store.create_user(user).await?;
    
    (StatusCode::CREATED, Json(UserInfo::from(user))).into_response()
}

/// List all users (filtered by permissions)
pub async fn list_users(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
) -> Response {
    // TODO: Implement user listing with permission filtering
    // Root can see all, tenant admin can see their tenant users
    
    let users: Vec<UserInfo> = vec![]; // Placeholder
    
    (StatusCode::OK, Json(users)).into_response()
}

/// Get user by ID
pub async fn get_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(user_id): Path<Uuid>,
) -> Response {
    // TODO: Check permissions and fetch user
    
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet").into_response()
}

/// Update user
pub async fn update_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Response {
    // TODO: Check permissions and update user
    
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet").into_response()
}

/// Delete user
pub async fn delete_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(user_id): Path<Uuid>,
) -> Response {
    // TODO: Check permissions and delete user
    
    (StatusCode::NO_CONTENT).into_response()
}

/// User login
pub async fn login(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Json(req): Json<LoginRequest>,
) -> Response {
    // Fetch user by username
    let user_opt = store.get_user_by_username(&req.username).await.unwrap_or(None);
    let user = match user_opt {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    if let Some(hash) = &user.password_hash {
        match crate::auth_middleware::verify_password(&req.password, hash) {
            Ok(true) => (),
            _ => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
        }
    } else {
        return (StatusCode::UNAUTHORIZED, "Account uses external authentication").into_response();
    }

    // Create session
    let session = crate::auth_middleware::create_session(
        user.id,
        user.username.clone(),
        user.tenant_id,
        user.role.clone(),
        86400, // 24 hours
    );

    // Generate token
    let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "default_secret_for_dev".to_string());
    let token = match crate::auth_middleware::generate_token(session, &secret) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response(),
    };

    let response = LoginResponse {
        token,
        user: UserInfo::from(user),
        expires_at: (chrono::Utc::now() + chrono::Duration::seconds(86400)).to_rfc3339(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Get current user
pub async fn get_current_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    // TODO: Extract user from JWT token
) -> Response {
    // TODO: Return current user info from token
    
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet").into_response()
}

/// Logout (invalidate token)
pub async fn logout(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
) -> Response {
    // TODO: Implement token invalidation if using token blacklist
    
    (StatusCode::NO_CONTENT).into_response()
}
