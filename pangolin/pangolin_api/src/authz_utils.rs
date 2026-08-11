use pangolin_core::model::{Asset, Catalog, Namespace};
use pangolin_core::permission::{Action, Permission, PermissionScope};
use pangolin_core::user::UserRole;
use uuid::Uuid;

/// Does a `Tenant`-scoped grant apply to a resource in `resource_tenant_id`?
///
/// B0i: all three access checks below used a bare `PermissionScope::Tenant =>
/// true`, never comparing the grant's own `tenant_id` against the resource's.
/// A tenant-wide grant issued in tenant A therefore satisfied access checks for
/// resources in tenant B. Nothing exploited it today only because callers
/// pre-scope their store queries to one tenant - but every path that can
/// surface cross-tenant rows (root impersonation, search, dashboards) leaked
/// through it, and the invariant was one refactor away from mattering.
fn tenant_grant_applies(perm: &Permission, resource_tenant_id: Uuid) -> bool {
    perm.tenant_id == resource_tenant_id
}

/// Check if a user has access to a catalog based on their permissions
///
/// Checks for Read or Discoverable actions on:
/// - Exact catalog scope
/// - Tenant-wide scope, when the grant belongs to the resource's tenant
pub fn has_catalog_access(
    resource_tenant_id: Uuid,
    catalog_id: Uuid,
    permissions: &[Permission],
    required_actions: &[Action],
) -> bool {
    permissions.iter().any(|perm| {
        // Check if permission scope covers this catalog
        let scope_matches = matches!(
            &perm.scope,
            PermissionScope::Catalog { catalog_id: cid } if *cid == catalog_id
        ) || (matches!(&perm.scope, PermissionScope::Tenant)
            && tenant_grant_applies(perm, resource_tenant_id));

        // Check if permission has any of the required actions
        let has_action = required_actions
            .iter()
            .any(|action| perm.actions.iter().any(|a| a.implies(action)));

        scope_matches && has_action
    })
}

/// Check if a user has access to a namespace based on their permissions
///
/// Checks for Read or Discoverable actions on:
/// - Exact namespace scope
/// - Parent catalog scope
/// - Tenant-wide scope, when the grant belongs to the resource's tenant
pub fn has_namespace_access(
    resource_tenant_id: Uuid,
    catalog_id: Uuid,
    namespace: &str,
    permissions: &[Permission],
    required_actions: &[Action],
) -> bool {
    permissions.iter().any(|perm| {
        // Check if permission scope covers this namespace
        let scope_matches = match &perm.scope {
            PermissionScope::Namespace {
                catalog_id: cid,
                namespace: ns,
            } => *cid == catalog_id && ns == namespace,
            PermissionScope::Catalog { catalog_id: cid } => *cid == catalog_id,
            PermissionScope::Tenant => tenant_grant_applies(perm, resource_tenant_id),
            _ => false,
        };

        // Check if permission has any of the required actions
        let has_action = required_actions
            .iter()
            .any(|action| perm.actions.iter().any(|a| a.implies(action)));

        scope_matches && has_action
    })
}

/// Check if a user has access to an asset based on their permissions
///
/// Checks for Read or Discoverable actions on:
/// - Exact asset scope
/// - Parent namespace scope
/// - Parent catalog scope
/// - Tenant-wide scope, when the grant belongs to the resource's tenant
pub fn has_asset_access(
    resource_tenant_id: Uuid,
    catalog_id: Uuid,
    namespace: &str,
    asset_id: Uuid,
    permissions: &[Permission],
    required_actions: &[Action],
) -> bool {
    permissions.iter().any(|perm| {
        // Check if permission scope covers this asset
        let scope_matches = match &perm.scope {
            PermissionScope::Asset {
                catalog_id: cid,
                namespace: ns,
                asset_id: aid,
            } => *cid == catalog_id && ns == namespace && *aid == asset_id,
            PermissionScope::Namespace {
                catalog_id: cid,
                namespace: ns,
            } => *cid == catalog_id && ns == namespace,
            PermissionScope::Catalog { catalog_id: cid } => *cid == catalog_id,
            PermissionScope::Tenant => tenant_grant_applies(perm, resource_tenant_id),
            _ => false,
        };

        // Check if permission has any of the required actions
        let has_action = required_actions
            .iter()
            .any(|action| perm.actions.iter().any(|a| a.implies(action)));

        scope_matches && has_action
    })
}

/// Filter catalogs based on user permissions
///
/// Returns only catalogs the user has Read or Discoverable access to.
/// Root and TenantAdmin users bypass filtering.
pub fn filter_catalogs(
    resource_tenant_id: Uuid,
    catalogs: Vec<Catalog>,
    permissions: &[Permission],
    user_role: UserRole,
) -> Vec<Catalog> {
    // Root and TenantAdmin see everything
    if matches!(user_role, UserRole::Root | UserRole::TenantAdmin) {
        return catalogs;
    }

    let required_actions = vec![Action::Read, Action::ManageDiscovery];

    catalogs
        .into_iter()
        .filter(|catalog| {
            has_catalog_access(
                resource_tenant_id,
                catalog.id,
                permissions,
                &required_actions,
            )
        })
        .collect()
}

/// Filter namespaces based on user permissions
///
/// Returns only namespaces the user has Read or Discoverable access to.
/// Root and TenantAdmin users bypass filtering.
pub fn filter_namespaces(
    resource_tenant_id: Uuid,
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

    namespaces
        .into_iter()
        .filter(|(namespace, catalog_name)| {
            // Get catalog ID from the map
            if let Some(&catalog_id) = catalog_id_map.get(catalog_name) {
                let namespace_str = namespace.name.join(".");
                has_namespace_access(
                    resource_tenant_id,
                    catalog_id,
                    &namespace_str,
                    permissions,
                    &required_actions,
                )
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
    resource_tenant_id: Uuid,
    assets: Vec<(
        Asset,
        Option<pangolin_core::business_metadata::BusinessMetadata>,
        String,
        Vec<String>,
    )>,
    permissions: &[Permission],
    user_role: UserRole,
    catalog_id_map: &std::collections::HashMap<String, Uuid>,
) -> Vec<(
    Asset,
    Option<pangolin_core::business_metadata::BusinessMetadata>,
    String,
    Vec<String>,
)> {
    // Root and TenantAdmin see everything
    if matches!(user_role, UserRole::Root | UserRole::TenantAdmin) {
        return assets;
    }

    let required_actions = vec![Action::Read, Action::ManageDiscovery];

    assets
        .into_iter()
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
                has_asset_access(
                    resource_tenant_id,
                    catalog_id,
                    &namespace_str,
                    asset.id,
                    permissions,
                    &required_actions,
                )
            } else {
                false
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashSet;

    /// Build a permission with a single `Read` action.
    fn read_permission(tenant_id: Uuid, scope: PermissionScope) -> Permission {
        let mut actions = HashSet::new();
        actions.insert(Action::Read);
        Permission {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tenant_id,
            scope,
            actions,
            granted_by: Uuid::new_v4(),
            granted_at: Utc::now(),
        }
    }

    #[test]
    fn test_has_catalog_access_with_catalog_permission() {
        let tenant_id = Uuid::new_v4();
        let catalog_id = Uuid::new_v4();
        let permission = read_permission(tenant_id, PermissionScope::Catalog { catalog_id });

        assert!(has_catalog_access(
            tenant_id,
            catalog_id,
            &[permission],
            &[Action::Read]
        ));
    }

    #[test]
    fn test_has_catalog_access_with_tenant_permission() {
        let tenant_id = Uuid::new_v4();
        let catalog_id = Uuid::new_v4();
        let permission = read_permission(tenant_id, PermissionScope::Tenant);

        assert!(has_catalog_access(
            tenant_id,
            catalog_id,
            &[permission],
            &[Action::Read]
        ));
    }

    /// Regression test for B0i: a `Tenant`-scoped grant issued in tenant A must
    /// not satisfy access for a resource in tenant B.
    #[test]
    fn tenant_scoped_grant_does_not_cross_tenants() {
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let catalog_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let permission = read_permission(tenant_a, PermissionScope::Tenant);
        let grants = [permission];

        assert!(has_catalog_access(
            tenant_a,
            catalog_id,
            &grants,
            &[Action::Read]
        ));
        assert!(
            !has_catalog_access(tenant_b, catalog_id, &grants, &[Action::Read]),
            "a tenant-A grant must not authorize a tenant-B catalog"
        );
        assert!(
            !has_namespace_access(tenant_b, catalog_id, "sales", &grants, &[Action::Read]),
            "a tenant-A grant must not authorize a tenant-B namespace"
        );
        assert!(
            !has_asset_access(
                tenant_b,
                catalog_id,
                "sales",
                asset_id,
                &grants,
                &[Action::Read]
            ),
            "a tenant-A grant must not authorize a tenant-B asset"
        );
    }

    #[test]
    fn test_has_catalog_access_without_permission() {
        let tenant_id = Uuid::new_v4();
        let catalog_id = Uuid::new_v4();
        let other_catalog_id = Uuid::new_v4();
        let permission = read_permission(
            tenant_id,
            PermissionScope::Catalog {
                catalog_id: other_catalog_id,
            },
        );

        assert!(!has_catalog_access(
            tenant_id,
            catalog_id,
            &[permission],
            &[Action::Read]
        ));
    }

    #[test]
    fn test_filter_catalogs_as_root() {
        let catalogs = vec![Catalog {
            id: Uuid::new_v4(),
            name: "catalog1".to_string(),
            catalog_type: pangolin_core::model::CatalogType::Local,
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }];

        let filtered = filter_catalogs(Uuid::new_v4(), catalogs.clone(), &[], UserRole::Root);
        assert_eq!(filtered.len(), catalogs.len());
    }

    #[test]
    fn test_filter_catalogs_as_tenant_user_with_permission() {
        let catalog_id = Uuid::new_v4();
        let catalogs = vec![Catalog {
            id: catalog_id,
            name: "catalog1".to_string(),
            catalog_type: pangolin_core::model::CatalogType::Local,
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }];

        let tenant_id = Uuid::new_v4();
        let permission = read_permission(tenant_id, PermissionScope::Catalog { catalog_id });

        let filtered = filter_catalogs(
            tenant_id,
            catalogs.clone(),
            &[permission],
            UserRole::TenantUser,
        );
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_catalogs_as_tenant_user_without_permission() {
        let catalog_id = Uuid::new_v4();
        let catalogs = vec![Catalog {
            id: catalog_id,
            name: "catalog1".to_string(),
            catalog_type: pangolin_core::model::CatalogType::Local,
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }];

        let filtered = filter_catalogs(Uuid::new_v4(), catalogs, &[], UserRole::TenantUser);
        assert_eq!(filtered.len(), 0);
    }
}
