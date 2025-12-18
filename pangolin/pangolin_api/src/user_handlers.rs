use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use pangolin_core::user::{User, UserRole, OAuthProvider, UserSession};
use pangolin_store::CatalogStore;
use std::sync::Arc;
use utoipa::ToSchema;

/// Request to create a new user
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
}

/// Request to update a user
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub active: Option<bool>,
}

/// User login request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response with JWT token
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: String,
}

/// Public user information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

/// App configuration (public)
#[derive(Serialize)]
pub struct AppConfig {
    pub auth_enabled: bool,
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserInfo),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Json(req): Json<CreateUserRequest>,
) -> Response {
    // Check permissions - only root or tenant admin can create users
    
    // Hash password if provided
    let password_hash = if let Some(password) = req.password {
        match crate::auth_middleware::hash_password(&password) {
            Ok(hash) => Some(hash),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
        }
    } else {
        None
    };
    
    let user = match req.role {
        UserRole::Root => {
            return (StatusCode::FORBIDDEN, "Cannot create another Root user").into_response();
        }
        UserRole::TenantAdmin => {
            // Root can create Tenant Admin
            if session.role != UserRole::Root && session.role != UserRole::TenantAdmin {
                 return (StatusCode::FORBIDDEN, "Only Root or Tenant Admin can create Tenant Admins").into_response();
            }

            let tenant_id = match req.tenant_id {
                Some(id) => id,
                None => return (StatusCode::BAD_REQUEST, "Tenant admin requires tenant_id").into_response(),
            };
            if password_hash.is_none() {
                return (StatusCode::BAD_REQUEST, "Tenant admin requires password").into_response();
            }
            User::new_tenant_admin(req.username, req.email, password_hash.unwrap(), tenant_id)
        }
        UserRole::TenantUser => {
            // Root cannot create Tenant User
            if session.role == UserRole::Root {
                return (StatusCode::FORBIDDEN, "Root user cannot create Tenant Users. Login as Tenant Admin.").into_response();
            }

            let tenant_id = match req.tenant_id {
                Some(id) => id,
                None => return (StatusCode::BAD_REQUEST, "Tenant user requires tenant_id").into_response(),
            };
            
            // Ensure Tenant Admin is creating user for THEIR tenant
            if let Some(session_tid) = session.tenant_id {
                if session_tid != tenant_id {
                     return (StatusCode::FORBIDDEN, "Cannot create user for another tenant").into_response();
                }
            }

            if password_hash.is_none() {
                return (StatusCode::BAD_REQUEST, "Tenant user requires password").into_response();
            }
            User::new_tenant_user(req.username, req.email, password_hash.unwrap(), tenant_id)
        }
    };
    
    // Store user in database
    if let Err(_) = store.create_user(user.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response();
    }
    
    (StatusCode::CREATED, Json(UserInfo::from(user))).into_response()
}

/// List all users (filtered by permissions)
use crate::auth::TenantId;

/// List all users (filtered by permissions)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    responses(
        (status = 200, description = "List of users", body = Vec<UserInfo>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_users(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(tenant): Extension<TenantId>,
    Extension(_session): Extension<UserSession>,
) -> Response {
    // Determine tenant_id to list for
    // Extension(TenantId) provides the effective tenant (from token or header fallback)
    let tenant_id = tenant.0;

    match store.list_users(Some(tenant_id)).await {
        Ok(users) => {
             let infos: Vec<UserInfo> = users.into_iter().map(UserInfo::from).collect();
             (StatusCode::OK, Json(infos)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list users: {}", e)).into_response(),
    }
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = UserInfo),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    // Permission check: Users can view themselves, or admins can view anyone
    if session.user_id != user_id && !crate::authz::is_admin(&session.role) {
        return (StatusCode::FORBIDDEN, "Cannot view other users").into_response();
    }
    
    match store.get_user(user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(UserInfo::from(user))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get user: {}", e)).into_response(),
    }
}

/// Update user
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserInfo),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Response {
    // Permission check: Users can update themselves (limited fields), admins can update anyone
    let is_self = session.user_id == user_id;
    let is_admin = crate::authz::is_admin(&session.role);
    
    if !is_self && !is_admin {
        return (StatusCode::FORBIDDEN, "Cannot update other users").into_response();
    }
    
    // Check if user exists
    let existing_user = match store.get_user(user_id).await {
         Ok(Some(u)) => u,
         Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
         Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch user: {}", e)).into_response(),
    };
    
    let mut updated_user = existing_user;
    
    if let Some(email) = req.email {
        updated_user.email = email;
    }
    if let Some(password) = req.password {
        match crate::auth_middleware::hash_password(&password) {
            Ok(hash) => updated_user.password_hash = Some(hash),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
        }
    }
    if let Some(active) = req.active {
        if !is_admin {
             return (StatusCode::FORBIDDEN, "Only admins can change user status").into_response();
        }
        updated_user.active = active;
    }
    
    match store.update_user(updated_user).await {
        Ok(_) => (StatusCode::OK, "User updated").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update user: {}", e)).into_response(),
    }
}

/// Delete user
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 400, description = "Cannot delete yourself"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(user_id): Path<Uuid>,
) -> Response {
    // Permission check: Only admins can delete users
    if !crate::authz::is_admin(&session.role) {
        return (StatusCode::FORBIDDEN, "Only admins can delete users").into_response();
    }
    
    // Cannot delete yourself
    if session.user_id == user_id {
        return (StatusCode::BAD_REQUEST, "Cannot delete yourself").into_response();
    }
    
    match store.delete_user(user_id).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete user: {}", e)).into_response(),
    }
}

/// User login
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    tag = "Users",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Json(req): Json<LoginRequest>,
) -> Response {
    // Fetch user by username
    // Fetch user by username
    let user_opt = store.get_user_by_username(&req.username).await.unwrap_or(None);
    let user = match user_opt {
        Some(u) => {
            // Verify password for DB user
            if let Some(hash) = &u.password_hash {
                match crate::auth_middleware::verify_password(&req.password, hash) {
                    Ok(true) => u,
                    _ => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
                }
            } else {
                return (StatusCode::UNAUTHORIZED, "Account uses external authentication").into_response();
            }
        },
        None => {
            // Check for Root User via Environment Variables
            let root_user = std::env::var("PANGOLIN_ROOT_USER").unwrap_or_else(|_| "admin".to_string());
            let root_pass = std::env::var("PANGOLIN_ROOT_PASSWORD").unwrap_or_else(|_| "password".to_string());

            if !root_user.is_empty() && req.username == root_user && req.password == root_pass {
                // Create a temporary User object for the root user session
                // Use a deterministic UUID for root to ensure consistency across restarts
                let root_id = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap(); 
                User {
                    id: root_id,
                    username: root_user,
                    email: "root@pangolin.local".to_string(), // Dummy email
                    password_hash: None, // No hash needed since we verified plaintext
                    tenant_id: None, // Root has no tenant
                    role: UserRole::Root,
                    oauth_provider: None,
                    oauth_subject: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_login: None,
                    active: true,
                }
            } else {
                return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
            }
        }
    };

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
    State(_store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
) -> Response {
    let user_info = UserInfo {
        id: session.user_id,
        username: session.username,
        email: "".to_string(), 
        tenant_id: session.tenant_id,
        role: session.role,
        oauth_provider: None,
    };
    
    (StatusCode::OK, Json(user_info)).into_response()
}

/// Logout (invalidate token)
pub async fn logout(
    State(_store): State<Arc<dyn CatalogStore + Send + Sync>>,
) -> Response {
    // TODO: Implement token invalidation if using token blacklist
    
    (StatusCode::NO_CONTENT).into_response()
}

pub async fn get_app_config() -> Response {
    // Check if NO_AUTH mode is enabled (must be exactly "true" for security)
    let no_auth = std::env::var("PANGOLIN_NO_AUTH")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    let auth_enabled = !no_auth;
    (StatusCode::OK, Json(AppConfig { auth_enabled })).into_response()
}
