use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashSet;
use pangolin_core::permission::{Role, Permission, PermissionScope, Action, UserRole};
use pangolin_core::user::{UserSession, UserRole as AuthRole};
use pangolin_store::CatalogStore;

/// Request to create a new role
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Uuid,
}

/// Request to assign a role to a user
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

/// Request to grant a permission
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GrantPermissionRequest {
    pub user_id: Uuid,
    pub scope: PermissionScope,
    pub actions: HashSet<Action>,
}

/// Create a new role
pub async fn create_role(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Json(req): Json<CreateRoleRequest>,
) -> Response {
    // Strict permission check
    if session.role != AuthRole::Root && session.role != AuthRole::TenantAdmin {
        return (StatusCode::FORBIDDEN, "Only admins can create roles").into_response();
    }
    
    let role = Role::new(
        req.name,
        req.description,
        req.tenant_id,
        session.user_id,
    );
    
    if let Err(e) = store.create_role(role.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create role: {}", e)).into_response();
    }
    
    (StatusCode::CREATED, Json(role)).into_response()
}

/// List roles
pub async fn list_roles(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
) -> Response {
    let tenant_id = session.tenant_id.unwrap_or_default(); 
    
    // For now list for session tenant
    let roles = match store.list_roles(tenant_id).await {
        Ok(roles) => roles,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list roles: {}", e)).into_response(),
    };
    
    (StatusCode::OK, Json(roles)).into_response()
}

/// Get role details
pub async fn get_role(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(role_id): Path<Uuid>,
) -> Response {
    match store.get_role(role_id).await {
        Ok(Some(role)) => (StatusCode::OK, Json(role)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Role not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get role: {}", e)).into_response(),
    }
}

/// Update role
pub async fn update_role(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(role_id): Path<Uuid>,
    Json(mut role): Json<Role>,
) -> Response {
    if session.role != AuthRole::Root && session.role != AuthRole::TenantAdmin {
        return (StatusCode::FORBIDDEN, "Only admins can update roles").into_response();
    }

    if role.id != role_id {
         return (StatusCode::BAD_REQUEST, "Role ID mismatch").into_response();
    }

    match store.update_role(role).await {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update role: {}", e)).into_response(),
    }
}

/// Delete role
pub async fn delete_role(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(role_id): Path<Uuid>,
) -> Response {
    if session.role != AuthRole::Root && session.role != AuthRole::TenantAdmin {
        return (StatusCode::FORBIDDEN, "Only admins can delete roles").into_response();
    }

    match store.delete_role(role_id).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete role: {}", e)).into_response(),
    }
}

/// Assign role to user
pub async fn assign_role(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path(target_user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> Response {
    if session.role != AuthRole::Root && session.role != AuthRole::TenantAdmin {
        return (StatusCode::FORBIDDEN, "Only admins can assign roles").into_response();
    }

    let user_role = UserRole::new(
        target_user_id,
        req.role_id,
        session.user_id,
    );
    
    match store.assign_role(user_role).await {
        Ok(_) => (StatusCode::CREATED).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to assign role: {}", e)).into_response(),
    }
}

/// Get user roles
pub async fn get_user_roles(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(target_user_id): Path<Uuid>,
) -> Response {
    match store.get_user_roles(target_user_id).await {
        // We probably want to return full Role objects, but store returns UserRole (mapping).
        // The frontend expects Role[].
        // So we need to fetch the roles from the store for each UserRole.
        Ok(user_roles) => {
             let mut roles = Vec::new();
             for ur in user_roles {
                 if let Ok(Some(role)) = store.get_role(ur.role_id).await {
                     roles.push(role);
                 }
             }
             (StatusCode::OK, Json(roles)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get user roles: {}", e)).into_response(),
    }
}

/// Revoke role from user
pub async fn revoke_role(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Path((target_user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Response {
    if session.role != AuthRole::Root && session.role != AuthRole::TenantAdmin {
        return (StatusCode::FORBIDDEN, "Only admins can revoke roles").into_response();
    }

    match store.revoke_role(target_user_id, role_id).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to revoke role: {}", e)).into_response(),
    }
}

/// Grant permission to user
pub async fn grant_permission(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Extension(session): Extension<UserSession>,
    Json(req): Json<GrantPermissionRequest>,
) -> Response {
    if session.role == pangolin_core::user::UserRole::Root {
        return (StatusCode::FORBIDDEN, "Root user cannot grant granular permissions. Please login as Tenant Admin.").into_response();
    }

    let permission = Permission::new(
        req.user_id,
        req.scope,
        req.actions,
        session.user_id,
    );
    
    match store.create_permission(permission.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(permission)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to grant permission: {}", e)).into_response(),
    }
}

/// Revoke permission
pub async fn revoke_permission(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(permission_id): Path<Uuid>,
) -> Response {
    match store.revoke_permission(permission_id).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to revoke permission: {}", e)).into_response(),
    }
}

/// Get user permissions
pub async fn get_user_permissions(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(target_user_id): Path<Uuid>,
) -> Response {
    match store.list_user_permissions(target_user_id).await {
        Ok(perms) => (StatusCode::OK, Json(perms)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list permissions: {}", e)).into_response(),
    }
}
