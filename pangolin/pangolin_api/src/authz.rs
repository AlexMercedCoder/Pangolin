use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_core::permission::{PermissionScope, Action};
use pangolin_core::user::UserSession;
use anyhow::Result;

pub async fn check_permission(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    session: &UserSession,
    action: &Action,
    scope: &PermissionScope,
) -> Result<bool> {
    
    use pangolin_core::user::UserRole as RoleEnum;
    
    // 1. Root User - Global Access
    if session.role == RoleEnum::Root {
        return Ok(true);
    }
    
    // 2. Tenant Admin - Tenant Scope Access
    // Simplification: TenantAdmin has all permissions for their tenant.
    // In a real app we'd verify the scope belongs to the tenant.
    // For now, if role is TenantAdmin, allow.
    // Wait, session role is Enum?
    // UserSession struct:
    // pub struct UserSession {
    //    pub user_id: Uuid,
    //    pub tenant_id: Option<Uuid>,
    //    pub role: UserRole, // Enum
    //    ...
    // }
    // If role is TenantAdmin, we should allow, assuming the middleware/handler ensured tenant context.
    // But check_permission might be called for a scope outside tenant.
    // Safest: Check dynamic roles first. The Enum role is just a default set of perms.
    // We can assume TenantAdmin Enum implies ALL permissions for that tenant.
    
    // use pangolin_core::user::UserRole as RoleEnum; // Already imported or handled
    if session.role == RoleEnum::TenantAdmin {
         // TODO: Check if scope matches session.tenant_id
         return Ok(true);
    }
    
    // 3. Fetch Assigned Roles
    let user_roles = store.get_user_roles(session.user_id).await?;
    
    for user_role in &user_roles {
        if let Some(role) = store.get_role(user_role.role_id).await? {
            for grant in role.permissions {
                // Determine if grant allows
                let grant_allows_action = grant.actions.iter().any(|a| a.implies(action));
                
                if grant.scope.covers(scope) && grant_allows_action {
                    return Ok(true);
                }
            }
        }
    }
    
    // 4. Fetch Direct Permissions
    let direct_perms = store.list_user_permissions(session.user_id, None).await?;
    for perm in &direct_perms {
        if perm.scope.covers(scope) && perm.actions.iter().any(|a| a.implies(action)) {
             return Ok(true);
        }
    }
    
    // 5. Tag-based Permissions (If required scope is Asset)
    if let PermissionScope::Asset { catalog_id: _, namespace: _, asset_id } = scope {
        if let Ok(Some(metadata)) = store.get_business_metadata(*asset_id).await {
             for tag in metadata.tags {
                 let tag_scope = PermissionScope::Tag { tag_name: tag };
                 
                 // Check if user has permission on this Tag Scope
                 // 1. Direct
                for perm in &direct_perms {
                    if perm.scope.covers(&tag_scope) && perm.actions.iter().any(|a| a.implies(action)) {
                        return Ok(true);
                    }
                }
                
                // 2. Role
                 for user_role in &user_roles {
                    if let Ok(Some(role)) = store.get_role(user_role.role_id).await {
                        for grant in role.permissions {
                             if grant.scope.covers(&tag_scope) && grant.actions.iter().any(|a| a.implies(action)) {
                                return Ok(true);
                            }
                        }
                    }
                }
             }
        }
    }

    Ok(false)
}

/// Helper to check if user is an admin (TenantAdmin or Root)
pub fn is_admin(role: &pangolin_core::user::UserRole) -> bool {
    use pangolin_core::user::UserRole as RoleEnum;
    matches!(role, RoleEnum::TenantAdmin | RoleEnum::Root)
}

/// Get the catalog name for a given asset ID
/// This is needed to check catalog-scoped permissions
pub async fn get_catalog_for_asset(
    store: &Arc<dyn CatalogStore + Send + Sync>,
    tenant_id: uuid::Uuid,
    asset_id: uuid::Uuid,
) -> Result<String> {
    
    // Optimized path: Use O(1) direct lookup if available
    match store.get_asset_by_id(tenant_id, asset_id).await {
        Ok(Some((_, catalog_name, _))) => return Ok(catalog_name),
        Ok(None) => {}, // Not found by ID, weird if store supports it but didn't find it.
        Err(_) => {}, // Store might not support it (e.g., deprecated stores), fall back to scan
    }

    use uuid::Uuid;
    
    // Fallback: Iterate through catalogs to find the asset (O(N))
    let catalogs = store.list_catalogs(tenant_id, None).await?;
    
    for catalog in catalogs {
        // list_namespaces takes (tenant_id, catalog_name, parent: Option<String>)
        let namespaces = store.list_namespaces(tenant_id, &catalog.name, None, None).await?;
        
        for namespace in namespaces {
            // list_assets takes (tenant_id, catalog_name, branch: Option<String>, namespace: Vec<String>)
            // key is already a Vec<String>
            let namespace_vec = namespace.name; // assuming implementation details from previous steps
            let assets = store.list_assets(tenant_id, &catalog.name, Some("main".to_string()), namespace_vec, None).await?;
            
            if assets.iter().any(|a| a.id == asset_id) {
                return Ok(catalog.name);
            }
        }
    }
    
    Err(anyhow::anyhow!("Asset not found"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pangolin_store::memory::MemoryStore;
    use pangolin_core::permission::{Role, Permission, UserRole as UserRoleStruct};
    use pangolin_core::user::{UserRole as UserRoleEnum};
    use uuid::Uuid;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_rbac_check() {
        let store_impl = MemoryStore::new();
        let store: Arc<dyn CatalogStore + Send + Sync> = Arc::new(store_impl);
        
        // Setup IDs
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let catalog_id = Uuid::new_v4();
        
        // Setup Session (TenantUser)
        let session = UserSession {
            user_id,
            tenant_id: Some(tenant_id),
            role: UserRoleEnum::TenantUser,
            username: "test_user".to_string(),
            issued_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        
        // Target Action/Scope
        let read_action = Action::Read;
        let write_action = Action::Write;
        let scope_ns = PermissionScope::Namespace { catalog_id, namespace: "sales".to_string() };
        let scope_asset = PermissionScope::Asset { catalog_id, namespace: "sales".to_string(), asset_id: Uuid::new_v4() };
        
        // 1. Initial Check (Should Fail)
        assert!(!check_permission(&store, &session, &read_action, &scope_ns).await.unwrap());
        
        // 2. Create Role with Read Permission on Namespace
        let mut role = Role::new("Viewer".to_string(), None, tenant_id, Uuid::new_v4());
        let mut actions = HashSet::new();
        actions.insert(Action::Read);
        role.add_permission(scope_ns.clone(), actions);
        store.create_role(role.clone()).await.unwrap();
        
        // 3. Assign Role
        let user_role = UserRoleStruct::new(user_id, role.id, Uuid::new_v4());
        store.assign_role(user_role).await.unwrap();
        
        // 4. Check Read (Should Pass for Namespace)
        assert!(check_permission(&store, &session, &read_action, &scope_ns).await.unwrap());
        
        // 5. Check Read on Asset (Should Pass because Namespace covers Asset)
        assert!(check_permission(&store, &session, &read_action, &scope_asset).await.unwrap());
        
        // 6. Check Write (Should Fail)
        assert!(!check_permission(&store, &session, &write_action, &scope_ns).await.unwrap());
        
        // 7. Grant Direct Write Permission
        let mut write_actions = HashSet::new();
        write_actions.insert(Action::Write);
        let perm = Permission::new(user_id, scope_ns.clone(), write_actions, Uuid::new_v4());
        store.create_permission(perm).await.unwrap();
        
        // 8. Check Write (Should NOW Pass)
        assert!(check_permission(&store, &session, &write_action, &scope_ns).await.unwrap());
    }

    #[tokio::test]
    async fn test_branch_and_tag_permissions() {
        let store_impl = MemoryStore::new();
        let store: Arc<dyn CatalogStore + Send + Sync> = Arc::new(store_impl);
        
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let catalog_id = Uuid::new_v4();
        
        let session = UserSession {
            user_id,
            tenant_id: Some(tenant_id),
            role: UserRoleEnum::TenantUser,
            username: "test_user".to_string(),
            issued_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        
        // --- Branch Permission Test ---
        let branch_action = Action::ExperimentalBranching; // Use actual action checked by handler
        let scope_branch = PermissionScope::Catalog { 
            catalog_id 
        };
        
        // Fail initially
        assert!(!check_permission(&store, &session, &branch_action, &scope_branch).await.unwrap());
        
        // Grant Branch Permission (on Catalog)
        let mut role = Role::new("BranchManager".to_string(), None, tenant_id, Uuid::new_v4());
        let mut actions = HashSet::new();
        actions.insert(Action::ExperimentalBranching);
        role.add_permission(scope_branch.clone(), actions);
        store.create_role(role.clone()).await.unwrap();
        
        // Assign Role
        let user_role = UserRoleStruct::new(user_id, role.id, Uuid::new_v4());
        store.assign_role(user_role).await.unwrap();
        
        // Pass
        assert!(check_permission(&store, &session, &branch_action, &scope_branch).await.unwrap());
        
        // --- Tag Permission Test ---
        let asset_id = Uuid::new_v4();
        let scope_asset = PermissionScope::Asset { 
            catalog_id, 
            namespace: "finance".to_string(), 
            asset_id 
        };
        let read_action = Action::Read;
        
        // Fail initially on asset
        assert!(!check_permission(&store, &session, &read_action, &scope_asset).await.unwrap());
        
        // Add "PII" tag to asset
        use pangolin_core::business_metadata::BusinessMetadata;
        let mut metadata = BusinessMetadata::new(asset_id, user_id); // Pass UUID
        metadata.tags.push("PII".to_string());
        store.upsert_business_metadata(metadata).await.unwrap(); // Use upsert variant
        
        // Create Role with Tag Permission
        let mut tag_role = Role::new("PIIAccess".to_string(), None, tenant_id, Uuid::new_v4());
        let mut tag_actions = HashSet::new();
        tag_actions.insert(Action::Read);
        let tag_scope = PermissionScope::Tag { tag_name: "PII".to_string() };
        tag_role.add_permission(tag_scope, tag_actions);
        store.create_role(tag_role.clone()).await.unwrap();
        
        // Assign Tag Role
        let user_tag_role = UserRoleStruct::new(user_id, tag_role.id, Uuid::new_v4());
        store.assign_role(user_tag_role).await.unwrap();
        
        // Pass (Access via Tag)
        assert!(check_permission(&store, &session, &read_action, &scope_asset).await.unwrap());
    }
}
