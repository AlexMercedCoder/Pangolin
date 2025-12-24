use pangolin_core::permission::{Permission, PermissionScope, Action};
use pangolin_core::model::{Catalog, Namespace, Asset};
use pangolin_core::user::UserRole;
use uuid::Uuid;

/// Check if a user has access to a catalog based on their permissions
/// 
/// Checks for Read or Discoverable actions on:
/// - Exact catalog scope
/// - Tenant-wide scope
pub fn has_catalog_access(
    catalog_id: Uuid,
    permissions: &[Permission],
    required_actions: &[Action],
) -> bool {
    permissions.iter().any(|perm| {
        // Check if permission scope covers this catalog
        let scope_matches = matches!(
            &perm.scope,
            PermissionScope::Catalog { catalog_id: cid } if *cid == catalog_id
        ) || matches!(&perm.scope, PermissionScope::Tenant);
        
        // Check if permission has any of the required actions
        let has_action = required_actions.iter().any(|action| {
            perm.actions.iter().any(|a| a.implies(action))
        });
        
        scope_matches && has_action
    })
}

/// Check if a user has access to a namespace based on their permissions
/// 
/// Checks for Read or Discoverable actions on:
/// - Exact namespace scope
/// - Parent catalog scope
/// - Tenant-wide scope
pub fn has_namespace_access(
    catalog_id: Uuid,
    namespace: &str,
    permissions: &[Permission],
    required_actions: &[Action],
) -> bool {
    permissions.iter().any(|perm| {
        // Check if permission scope covers this namespace
        let scope_matches = match &perm.scope {
            PermissionScope::Namespace { catalog_id: cid, namespace: ns } => {
                *cid == catalog_id && ns == namespace
            },
            PermissionScope::Catalog { catalog_id: cid } => *cid == catalog_id,
            PermissionScope::Tenant => true,
            _ => false,
        };
        
        // Check if permission has any of the required actions
        let has_action = required_actions.iter().any(|action| {
            perm.actions.iter().any(|a| a.implies(action))
        });
        
        scope_matches && has_action
    })
}

/// Check if a user has access to an asset based on their permissions
/// 
/// Checks for Read or Discoverable actions on:
/// - Exact asset scope
/// - Parent namespace scope
/// - Parent catalog scope
/// - Tenant-wide scope
pub fn has_asset_access(
    catalog_id: Uuid,
    namespace: &str,
    asset_id: Uuid,
    permissions: &[Permission],
    required_actions: &[Action],
) -> bool {
    permissions.iter().any(|perm| {
        // Check if permission scope covers this asset
        let scope_matches = match &perm.scope {
            PermissionScope::Asset { catalog_id: cid, namespace: ns, asset_id: aid } => {
                *cid == catalog_id && ns == namespace && *aid == asset_id
            },
            PermissionScope::Namespace { catalog_id: cid, namespace: ns } => {
                *cid == catalog_id && ns == namespace
            },
            PermissionScope::Catalog { catalog_id: cid } => *cid == catalog_id,
            PermissionScope::Tenant => true,
            _ => false,
        };
        
        // Check if permission has any of the required actions
        let has_action = required_actions.iter().any(|action| {
            perm.actions.iter().any(|a| a.implies(action))
        });
        
        scope_matches && has_action
    })
}

/// Filter catalogs based on user permissions
/// 
/// Returns only catalogs the user has Read or Discoverable access to.
/// Root and TenantAdmin users bypass filtering.
pub fn filter_catalogs(
    catalogs: Vec<Catalog>,
    permissions: &[Permission],
    user_role: UserRole,
) -> Vec<Catalog> {
    // Root and TenantAdmin see everything
    if matches!(user_role, UserRole::Root | UserRole::TenantAdmin) {
        return catalogs;
    }
    
    let required_actions = vec![Action::Read, Action::ManageDiscovery];
    
    catalogs.into_iter()
        .filter(|catalog| has_catalog_access(catalog.id, permissions, &required_actions))
        .collect()
}

/// Filter namespaces based on user permissions
/// 
/// Returns only namespaces the user has Read or Discoverable access to.
/// Root and TenantAdmin users bypass filtering.
pub fn filter_namespaces(
    namespaces: Vec<(Namespace, String)>,
    permissions: &[Permission],
    user_role: UserRole,
    catalog_id_map: &std::collections::HashMap<String, Uuid>,
) -> Vec<(Namespace, String)> {
    // Root and TenantAdmin see everything
    if matches!(user_role, UserRole::Root | UserRole::TenantAdmin) {
        return namespaces;
    }
    
    let required_actions = vec![Action::Read, Action::ManageDiscovery];
    
    namespaces.into_iter()
        .filter(|(namespace, catalog_name)| {
            // Get catalog ID from the map
            if let Some(&catalog_id) = catalog_id_map.get(catalog_name) {
                let namespace_str = namespace.name.join(".");
                has_namespace_access(catalog_id, &namespace_str, permissions, &required_actions)
            } else {
                false
            }
        })
        .collect()
}

/// Filter assets based on user permissions
/// 
/// Returns only assets the user has Read or Discoverable access to.
/// Root and TenantAdmin users bypass filtering.
pub fn filter_assets(
    assets: Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)>,
    permissions: &[Permission],
    user_role: UserRole,
    catalog_id_map: &std::collections::HashMap<String, Uuid>,
) -> Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)> {
    // Root and TenantAdmin see everything
    if matches!(user_role, UserRole::Root | UserRole::TenantAdmin) {
        return assets;
    }
    
    let required_actions = vec![Action::Read, Action::ManageDiscovery];
    
    assets.into_iter()
        .filter(|(asset, metadata, catalog_name, namespace)| {
            // Check discoverable flag - if discoverable, anyone can see it
            if let Some(meta) = metadata {
                if meta.discoverable {
                    return true;
                }
            }

            // Get catalog ID from the map
            if let Some(&catalog_id) = catalog_id_map.get(catalog_name) {
                let namespace_str = namespace.join(".");
                has_asset_access(catalog_id, &namespace_str, asset.id, permissions, &required_actions)
            } else {
                false
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use chrono::Utc;

    #[test]
    fn test_has_catalog_access_with_catalog_permission() {
        let catalog_id = Uuid::new_v4();
        let mut actions = HashSet::new();
        actions.insert(Action::Read);
        
        let permission = Permission {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            scope: PermissionScope::Catalog { catalog_id },
            actions,
            granted_by: Uuid::new_v4(),
            granted_at: Utc::now(),
        };
        
        assert!(has_catalog_access(catalog_id, &[permission], &[Action::Read]));
    }

    #[test]
    fn test_has_catalog_access_with_tenant_permission() {
        let catalog_id = Uuid::new_v4();
        let mut actions = HashSet::new();
        actions.insert(Action::Read);
        
        let permission = Permission {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            scope: PermissionScope::Tenant,
            actions,
            granted_by: Uuid::new_v4(),
            granted_at: Utc::now(),
        };
        
        assert!(has_catalog_access(catalog_id, &[permission], &[Action::Read]));
    }

    #[test]
    fn test_has_catalog_access_without_permission() {
        let catalog_id = Uuid::new_v4();
        let other_catalog_id = Uuid::new_v4();
        let mut actions = HashSet::new();
        actions.insert(Action::Read);
        
        let permission = Permission {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            scope: PermissionScope::Catalog { catalog_id: other_catalog_id },
            actions,
            granted_by: Uuid::new_v4(),
            granted_at: Utc::now(),
        };
        
        assert!(!has_catalog_access(catalog_id, &[permission], &[Action::Read]));
    }

    #[test]
    fn test_filter_catalogs_as_root() {
        let catalogs = vec![
            Catalog {
                id: Uuid::new_v4(),
                name: "catalog1".to_string(),
                catalog_type: pangolin_core::model::CatalogType::Local,
                warehouse_name: None,
                storage_location: None,
                federated_config: None,
                properties: std::collections::HashMap::new(),
            }
        ];
        
        let filtered = filter_catalogs(catalogs.clone(), &[], UserRole::Root);
        assert_eq!(filtered.len(), catalogs.len());
    }

    #[test]
    fn test_filter_catalogs_as_tenant_user_with_permission() {
        let catalog_id = Uuid::new_v4();
        let catalogs = vec![
            Catalog {
                id: catalog_id,
                name: "catalog1".to_string(),
                catalog_type: pangolin_core::model::CatalogType::Local,
                warehouse_name: None,
                storage_location: None,
                federated_config: None,
                properties: std::collections::HashMap::new(),
            }
        ];
        
        let mut actions = HashSet::new();
        actions.insert(Action::Read);
        
        let permission = Permission {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            scope: PermissionScope::Catalog { catalog_id },
            actions,
            granted_by: Uuid::new_v4(),
            granted_at: Utc::now(),
        };
        
        let filtered = filter_catalogs(catalogs.clone(), &[permission], UserRole::TenantUser);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_catalogs_as_tenant_user_without_permission() {
        let catalog_id = Uuid::new_v4();
        let catalogs = vec![
            Catalog {
                id: catalog_id,
                name: "catalog1".to_string(),
                catalog_type: pangolin_core::model::CatalogType::Local,
                warehouse_name: None,
                storage_location: None,
                federated_config: None,
                properties: std::collections::HashMap::new(),
            }
        ];
        
        let filtered = filter_catalogs(catalogs, &[], UserRole::TenantUser);
        assert_eq!(filtered.len(), 0);
    }
}
