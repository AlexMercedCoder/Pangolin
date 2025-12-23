use pangolin_store::{PostgresStore, CatalogStore};
use pangolin_core::model::{Tenant, Warehouse, Catalog, CatalogType, Namespace, Asset, AssetType};
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_core::permission::{Role, Permission, Action, PermissionScope, UserRole as UserRoleAssignment, PermissionGrant};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use std::env;
use chrono::{DateTime, Utc};

const TEST_DB_URL: &str = "postgresql://pangolin:pangolin_dev_password@localhost:5433/pangolin_test";

#[tokio::test]
async fn test_postgres_tenant_crud() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    
    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");
    
    // Get tenant
    let retrieved = store.get_tenant(tenant_id).await.expect("Failed to get tenant");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, tenant.name);
    
    // List tenants
    let tenants = store.list_tenants().await.expect("Failed to list tenants");
    assert!(tenants.iter().any(|t| t.id == tenant_id));
}

#[tokio::test]
async fn test_postgres_warehouse_crud() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    
    // Create tenant first
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.expect("Failed to create tenant");
    
    // Create warehouse
    let warehouse_name = format!("test_warehouse_{}", Uuid::new_v4());
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        tenant_id,
        name: "test_wh".to_string(),
        use_sts: false,
        storage_config: From::from([("s3.bucket".to_string(), "b".to_string())]),
        vending_strategy: None,
    };
    
    store.create_warehouse(tenant_id, warehouse.clone()).await.expect("Failed to create warehouse");
    
    // Get warehouse
    let retrieved = store.get_warehouse(tenant_id, warehouse_name.clone()).await.expect("Failed to get warehouse");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().name, warehouse_name);
    assert_eq!(retrieved.as_ref().unwrap().use_sts, false);
    
    // List warehouses
    let warehouses = store.list_warehouses(tenant_id).await.expect("Failed to list warehouses");
    assert!(warehouses.iter().any(|w| w.name == warehouse_name));
    
    // Delete warehouse
    store.delete_warehouse(tenant_id, warehouse_name.clone()).await.expect("Failed to delete warehouse");
    
    // Verify deletion
    let deleted = store.get_warehouse(tenant_id, warehouse_name).await.expect("Failed to get warehouse after delete");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_postgres_catalog_crud() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    
    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.expect("Failed to create tenant");
    
    // Create catalog
    let catalog_name = format!("test_catalog_{}", Uuid::new_v4());
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.clone(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: Some("s3://test-bucket/catalog/".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    
    store.create_catalog(tenant_id, catalog.clone()).await.expect("Failed to create catalog");
    
    // Get catalog
    let retrieved = store.get_catalog(tenant_id, catalog_name.clone()).await.expect("Failed to get catalog");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().name, catalog_name);
    
    // List catalogs
    let catalogs = store.list_catalogs(tenant_id).await.expect("Failed to list catalogs");
    assert!(catalogs.iter().any(|c| c.name == catalog_name));
    
    // Delete catalog
    store.delete_catalog(tenant_id, catalog_name.clone()).await.expect("Failed to delete catalog");
    
    // Verify deletion
    let deleted = store.get_catalog(tenant_id, catalog_name).await.expect("Failed to get catalog after delete");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_postgres_namespace_operations() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    
    // Setup tenant and catalog
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    let catalog_name = format!("test_catalog_{}", Uuid::new_v4());
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.clone(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: Some("s3://test/".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog).await.unwrap();
    
    // Create namespace
    let namespace = Namespace {
        name: vec!["db".to_string(), "schema".to_string()],
        properties: HashMap::from([("owner".to_string(), "test".to_string())]),
    };
    
    store.create_namespace(tenant_id, &catalog_name, namespace.clone()).await.expect("Failed to create namespace");
    
    // Get namespace
    let retrieved = store.get_namespace(tenant_id, &catalog_name, namespace.name.clone()).await.expect("Failed to get namespace");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().name, namespace.name);
    
    // List namespaces
    let namespaces = store.list_namespaces(tenant_id, &catalog_name, None).await.expect("Failed to list namespaces");
    assert!(namespaces.iter().any(|n| n.name == namespace.name));
    
    // Update properties
    let new_props = HashMap::from([("updated".to_string(), "true".to_string())]);
    store.update_namespace_properties(tenant_id, &catalog_name, namespace.name.clone(), new_props).await.expect("Failed to update namespace");
    
    // Delete namespace
    store.delete_namespace(tenant_id, &catalog_name, namespace.name.clone()).await.expect("Failed to delete namespace");
    
    // Verify deletion
    let deleted = store.get_namespace(tenant_id, &catalog_name, namespace.name).await.expect("Failed to get namespace after delete");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_postgres_asset_operations() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    
    // Setup
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    let catalog_name = format!("test_catalog_{}", Uuid::new_v4());
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.clone(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: Some("s3://test/".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog).await.unwrap();
    
    let namespace = Namespace {
        name: vec!["db".to_string()],
        properties: HashMap::new(),
    };
    store.create_namespace(tenant_id, &catalog_name, namespace.clone()).await.unwrap();
    
    // Create asset
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "test_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/table".to_string(),
        properties: HashMap::new(),
    };
    
    store.create_asset(tenant_id, &catalog_name, None, namespace.name.clone(), asset.clone()).await.expect("Failed to create asset");
    
    // Get asset
    let retrieved = store.get_asset(tenant_id, &catalog_name, None, namespace.name.clone(), "test_table".to_string()).await.expect("Failed to get asset");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().name, "test_table");
    
    // List assets
    let assets = store.list_assets(tenant_id, &catalog_name, None, namespace.name.clone()).await.expect("Failed to list assets");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "test_table");
    
    // Delete asset
    store.delete_asset(tenant_id, &catalog_name, None, namespace.name.clone(), "test_table".to_string()).await.expect("Failed to delete asset");
    
    // Verify deletion
    let deleted = store.get_asset(tenant_id, &catalog_name, None, namespace.name, "test_table".to_string()).await.expect("Failed to get asset after delete");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_postgres_multi_tenant_isolation() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    
    // Create two tenants
    let tenant1_id = Uuid::new_v4();
    let tenant1 = Tenant {
        id: tenant1_id,
        name: format!("tenant1_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant1).await.unwrap();
    
    let tenant2_id = Uuid::new_v4();
    let tenant2 = Tenant {
        id: tenant2_id,
        name: format!("tenant2_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant2).await.unwrap();
    
    // Create warehouse for tenant1
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        tenant_id: tenant1_id,
        name: "shared_name".to_string(),
        use_sts: false,
        storage_config: HashMap::new(),
        vending_strategy: None,
    };
    store.create_warehouse(tenant1_id, warehouse).await.unwrap();
    
    // Tenant2 should not see tenant1's warehouse
    let tenant2_warehouses = store.list_warehouses(tenant2_id).await.unwrap();
    assert_eq!(tenant2_warehouses.len(), 0);
    
    // Tenant2 cannot delete tenant1's warehouse
    let delete_result = store.delete_warehouse(tenant2_id, "shared_name".to_string()).await;
    assert!(delete_result.is_err());
    
    // Tenant1's warehouse still exists
    let tenant1_warehouses = store.list_warehouses(tenant1_id).await.unwrap();
    assert_eq!(tenant1_warehouses.len(), 1);
}

#[tokio::test]
async fn test_postgres_catalog_delete_cascade() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");

    // Create Catalog
    let catalog_name = format!("test_catalog_{}", Uuid::new_v4());
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.clone(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog.clone()).await.expect("Failed");

    // Create Namespace
    let ns = Namespace { name: vec!["db".to_string()], properties: HashMap::new() };
    store.create_namespace(tenant_id, &catalog_name, ns.clone()).await.expect("Failed");

    // Create Asset
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "tbl".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://loc".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant_id, &catalog_name, None, vec!["db".to_string()], asset.clone()).await.expect("Failed");

    // Verify existence
    assert_eq!(store.list_namespaces(tenant_id, &catalog_name, None).await.unwrap().len(), 1);
    assert_eq!(store.list_assets(tenant_id, &catalog_name, None, vec!["db".to_string()]).await.unwrap().len(), 1);

    // Delete Catalog
    store.delete_catalog(tenant_id, catalog_name.clone()).await.expect("Failed to delete catalog");

    // Verify Cascading Delete
    let assets = store.list_assets(tenant_id, &catalog_name, None, vec!["db".to_string()]).await.unwrap();
    assert_eq!(assets.len(), 0);
    
    let namespaces = store.list_namespaces(tenant_id, &catalog_name, None).await.unwrap();
    assert_eq!(namespaces.len(), 0);
}

#[tokio::test]
async fn test_postgres_rbac_operations() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");

    // Create User
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: format!("alice_{}", Uuid::new_v4()),
        email: format!("alice_{}@example.com", Uuid::new_v4()),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant_id),
        role: UserRole::TenantAdmin,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user.clone()).await.expect("Failed to create user");

    // Get User
    let fetched = store.get_user(user_id).await.expect("Failed to get user");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().username, user.username);

    // Update User
    let mut updated_user = user.clone();
    updated_user.role = UserRole::TenantUser;
    store.update_user(updated_user.clone()).await.expect("Failed to update");
    
    let fetched_updated = store.get_user(user_id).await.unwrap().unwrap();
    assert_eq!(fetched_updated.role, UserRole::TenantUser);

    // Create Role
    
    let role_id = Uuid::new_v4();
    let role = Role {
        id: role_id,
        tenant_id: tenant_id,
        name: "DataEngineer".to_string(),
        description: Some("Manage tables".to_string()),
        permissions: vec![],
        created_by: user_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    store.create_role(role.clone()).await.expect("Failed to create role");
    
    // Assign Role
    let assignment = pangolin_core::permission::UserRole {
        user_id: user_id,
        role_id: role_id,
        assigned_by: user_id,
        assigned_at: chrono::Utc::now(),
    };
    store.assign_role(assignment).await.expect("Failed to assign role");
    
    // Verify Assignment
    let roles = store.get_user_roles(user_id).await.unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role_id, role_id);
    
    // Delete User
    store.delete_user(user_id).await.expect("Failed to delete user");
    assert!(store.get_user(user_id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_postgres_list_user_permissions_aggregation() {
    let store = PostgresStore::new(TEST_DB_URL).await.expect("Failed to create PostgresStore");
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();

    // 0. Setup Tenant
    let tenant = Tenant {
        id: tenant_id,
        name: format!("tenant_for_agg_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.expect("Failed to create tenant");

    // 0. Setup User
    let user = User {
        id: user_id,
        username: format!("user_for_agg_{}", Uuid::new_v4()),
        email: format!("agg_{}@example.com", Uuid::new_v4()),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant_id),
        role: UserRole::TenantUser,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user).await.expect("Failed to create user");

    // 1. Create a Role with permissions
    let mut role = Role::new("test-role-agg".to_string(), None, tenant_id, admin_id);
    let role_scope = PermissionScope::Catalog { catalog_id: Uuid::new_v4() };
    let mut role_actions = HashSet::new();
    role_actions.insert(Action::Read);
    role.add_permission(role_scope.clone(), role_actions);
    store.create_role(role.clone()).await.unwrap();

    // 2. Assign role to user
    let user_role = UserRoleAssignment::new(user_id, role.id, admin_id);
    store.assign_role(user_role).await.unwrap();

    // 3. Create a direct permission
    let direct_scope = PermissionScope::Catalog { catalog_id: Uuid::new_v4() };
    let mut direct_actions = HashSet::new();
    direct_actions.insert(Action::Write);
    let direct_perm = Permission::new(user_id, direct_scope.clone(), direct_actions, admin_id);
    store.create_permission(direct_perm.clone()).await.unwrap();

    // 4. List user permissions and verify aggregation
    let aggregated_perms = store.list_user_permissions(user_id).await.unwrap();
    
    assert_eq!(aggregated_perms.len(), 2, "Should have 2 permissions (1 direct, 1 from role)");
    
    let has_direct = aggregated_perms.iter().any(|p| p.scope == direct_scope && p.actions.contains(&Action::Write));
    let has_role_based = aggregated_perms.iter().any(|p| p.scope == role_scope && p.actions.contains(&Action::Read));
    
    assert!(has_direct, "Aggregated permissions should include direct permission");
    assert!(has_role_based, "Aggregated permissions should include role-based permission");
}
