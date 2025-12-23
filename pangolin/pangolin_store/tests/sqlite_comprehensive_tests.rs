use pangolin_store::{SqliteStore, CatalogStore};
use pangolin_core::model::*;
use uuid::Uuid;
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};
use pangolin_core::user::{User, UserRole};
use std::collections::HashMap;

async fn setup_store() -> SqliteStore {
    // Use in-memory SQLite database for tests
    let store = SqliteStore::new("sqlite::memory:").await.expect("Failed to create store");
    
    // Apply schema
    let schema = include_str!("../sql/sqlite_schema.sql");
    store.apply_schema(schema).await.expect("Failed to apply schema");
    
    store
}

#[tokio::test]
async fn test_sqlite_tenant_crud() {
    let store = setup_store().await;
    
    // Create tenant
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");
    
    // Get tenant
    let retrieved = store.get_tenant(tenant.id).await.expect("Failed to get tenant");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_tenant");
    
    // List tenants
    let tenants = store.list_tenants().await.expect("Failed to list tenants");
    assert_eq!(tenants.len(), 1);
}

#[tokio::test]
async fn test_sqlite_warehouse_crud() {
    let store = setup_store().await;
    
    // Create tenant first
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");
    
    // Create warehouse
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        tenant_id: tenant.id,
        name: "test_warehouse".to_string(),
        use_sts: false,
        storage_config: From::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "test-bucket".to_string()),
        ]),
        vending_strategy: None,
    };
    
    store.create_warehouse(tenant.id, warehouse.clone()).await.expect("Failed to create warehouse");
    
    // Get warehouse
    let retrieved = store.get_warehouse(tenant.id, "test_warehouse".to_string()).await.expect("Failed to get warehouse");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_warehouse");
    
    // List warehouses
    let warehouses = store.list_warehouses(tenant.id).await.expect("Failed to list warehouses");
    assert_eq!(warehouses.len(), 1);
    
    // Delete warehouse
    store.delete_warehouse(tenant.id, "test_warehouse".to_string()).await.expect("Failed to delete warehouse");
    let warehouses = store.list_warehouses(tenant.id).await.expect("Failed to list warehouses after delete");
    assert_eq!(warehouses.len(), 0);
}

#[tokio::test]
async fn test_sqlite_catalog_crud() {
    let store = setup_store().await;
    
    // Create tenant
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");
    
    // Create catalog
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("test_warehouse".to_string()),
        storage_location: Some("s3://test-bucket/catalog".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    
    store.create_catalog(tenant.id, catalog.clone()).await.expect("Failed to create catalog");
    
    // Get catalog
    let retrieved = store.get_catalog(tenant.id, "test_catalog".to_string()).await.expect("Failed to get catalog");
    assert!(retrieved.is_some());
    let retrieved_catalog = retrieved.unwrap();
    assert_eq!(retrieved_catalog.name, "test_catalog");
    assert_eq!(retrieved_catalog.id, catalog.id);
    
    // List catalogs
    let catalogs = store.list_catalogs(tenant.id).await.expect("Failed to list catalogs");
    assert_eq!(catalogs.len(), 1);
    
    // Delete catalog
    store.delete_catalog(tenant.id, "test_catalog".to_string()).await.expect("Failed to delete catalog");
    let catalogs = store.list_catalogs(tenant.id).await.expect("Failed to list catalogs after delete");
    assert_eq!(catalogs.len(), 0);
}

#[tokio::test]
async fn test_sqlite_namespace_operations() {
    let store = setup_store().await;
    
    // Create tenant and catalog
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");
    
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("test_warehouse".to_string()),
        storage_location: Some("s3://test-bucket/catalog".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant.id, catalog.clone()).await.expect("Failed to create catalog");
    
    // Create namespace
    let namespace = Namespace {
        name: vec!["db1".to_string(), "schema1".to_string()],
        properties: HashMap::new(),
    };
    
    store.create_namespace(tenant.id, "test_catalog", namespace.clone()).await.expect("Failed to create namespace");
    
    // Get namespace
    let retrieved = store.get_namespace(tenant.id, "test_catalog", vec!["db1".to_string(), "schema1".to_string()]).await.expect("Failed to get namespace");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, vec!["db1", "schema1"]);
    
    // List namespaces
    let namespaces = store.list_namespaces(tenant.id, "test_catalog", None).await.expect("Failed to list namespaces");
    assert_eq!(namespaces.len(), 1);
    
    // Update namespace properties
    let mut props = HashMap::new();
    props.insert("key1".to_string(), "value1".to_string());
    store.update_namespace_properties(tenant.id, "test_catalog", vec!["db1".to_string(), "schema1".to_string()], props).await.expect("Failed to update namespace properties");
    
    let updated = store.get_namespace(tenant.id, "test_catalog", vec!["db1".to_string(), "schema1".to_string()]).await.expect("Failed to get updated namespace");
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().properties.get("key1"), Some(&"value1".to_string()));
    
    // Delete namespace
    store.delete_namespace(tenant.id, "test_catalog", vec!["db1".to_string(), "schema1".to_string()]).await.expect("Failed to delete namespace");
    let namespaces = store.list_namespaces(tenant.id, "test_catalog", None).await.expect("Failed to list namespaces after delete");
    assert_eq!(namespaces.len(), 0);
}

#[tokio::test]
async fn test_sqlite_asset_operations() {
    let store = setup_store().await;
    
    // Create tenant, catalog, and namespace
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");
    
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("test_warehouse".to_string()),
        storage_location: Some("s3://test-bucket/catalog".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant.id, catalog.clone()).await.expect("Failed to create catalog");
    
    let namespace = Namespace {
        name: vec!["db1".to_string()],
        properties: HashMap::new(),
    };
    store.create_namespace(tenant.id, "test_catalog", namespace.clone()).await.expect("Failed to create namespace");
    
    // Create asset
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "test_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://test-bucket/table".to_string(),
        properties: HashMap::new(),
    };
    
    store.create_asset(tenant.id, "test_catalog", None, vec!["db1".to_string()], asset.clone()).await.expect("Failed to create asset");
    
    // Get asset
    let retrieved = store.get_asset(tenant.id, "test_catalog", None, vec!["db1".to_string()], "test_table".to_string()).await.expect("Failed to get asset");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_table");
    
    // List assets
    let assets = store.list_assets(tenant.id, "test_catalog", None, vec!["db1".to_string()]).await.expect("Failed to list assets");
    assert_eq!(assets.len(), 1);
    
    // Delete asset
    store.delete_asset(tenant.id, "test_catalog", None, vec!["db1".to_string()], "test_table".to_string()).await.expect("Failed to delete asset");
    let assets = store.list_assets(tenant.id, "test_catalog", None, vec!["db1".to_string()]).await.expect("Failed to list assets after delete");
    assert_eq!(assets.len(), 0);
}

#[tokio::test]
async fn test_sqlite_multi_tenant_isolation() {
    let store = setup_store().await;
    
    // Create two tenants
    let tenant1 = Tenant {
        id: Uuid::new_v4(),
        name: "tenant1".to_string(),
        properties: HashMap::new(),
    };
    let tenant2 = Tenant {
        id: Uuid::new_v4(),
        name: "tenant2".to_string(),
        properties: HashMap::new(),
    };
    
    store.create_tenant(tenant1.clone()).await.expect("Failed to create tenant1");
    store.create_tenant(tenant2.clone()).await.expect("Failed to create tenant2");
    
    // Create catalog for tenant1
    let catalog1 = Catalog {
        id: Uuid::new_v4(),
        name: "catalog1".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("warehouse1".to_string()),
        storage_location: Some("s3://bucket1/catalog".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant1.id, catalog1.clone()).await.expect("Failed to create catalog for tenant1");
    
    // Verify tenant2 cannot see tenant1's catalog
    let tenant2_catalogs = store.list_catalogs(tenant2.id).await.expect("Failed to list catalogs for tenant2");
    assert_eq!(tenant2_catalogs.len(), 0);
    
    // Verify tenant1 can see their catalog
    let tenant1_catalogs = store.list_catalogs(tenant1.id).await.expect("Failed to list catalogs for tenant1");
    assert_eq!(tenant1_catalogs.len(), 1);
}

#[tokio::test]
async fn test_sqlite_catalog_delete_cascade() {
    let store = setup_store().await;
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");

    // Create Catalog
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant.id, catalog.clone()).await.expect("Failed");

    // Create Namespace
    let ns = Namespace { name: vec!["db".to_string()], properties: HashMap::new() };
    store.create_namespace(tenant.id, "test_catalog", ns.clone()).await.expect("Failed");

    // Create Asset
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "tbl".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://loc".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant.id, "test_catalog", None, vec!["db".to_string()], asset.clone()).await.expect("Failed");

    // Verify existence
    assert_eq!(store.list_namespaces(tenant.id, "test_catalog", None).await.unwrap().len(), 1);
    assert_eq!(store.list_assets(tenant.id, "test_catalog", None, vec!["db".to_string()]).await.unwrap().len(), 1);

    // Delete Catalog
    store.delete_catalog(tenant.id, "test_catalog".to_string()).await.expect("Failed to delete catalog");

    // Verify Cascading Delete (Simulated in Sqlite via manual delete in code, or foreign keys if table dropped?
    // In our code `delete_catalog` manually deletes children.
    // Check if assets are gone.
    // Note: `list_assets` requires catalog to exist logically for some backends, but here query is specific.
    // However, since we deleted catalog, strict foreign keys might have deleted them too if we didn't do it manually.
    // But our manual implementation ensures it.
    
    // Check direct DB query or re-create catalog and check empty?
    // Let's check DB directly if possible or assert list returns empty.
    
    // In Sqlite, foreign keys ON DELETE CASCADE is enabled. 
    // And our code ALSO runs manual deletes. 
    // So they should definitely be gone.
    
    // We can't list assets for a non-existent catalog easily via API usually, but `list_assets` just queries by catalog_name string.
    let assets = store.list_assets(tenant.id, "test_catalog", None, vec!["db".to_string()]).await.unwrap();
    assert_eq!(assets.len(), 0);
    
    let namespaces = store.list_namespaces(tenant.id, "test_catalog", None).await.unwrap();
    assert_eq!(namespaces.len(), 0);
}

#[tokio::test]
async fn test_sqlite_rbac_operations() {
    let store = setup_store().await;
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");

    use pangolin_core::user::{User, UserRole, OAuthProvider};
    
    // Create User
    let user = User {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant.id),
        role: UserRole::TenantAdmin,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user.clone()).await.expect("Failed to create user");

    // Get User
    let fetched = store.get_user(user.id).await.expect("Failed to get user");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().username, "alice");

    // Update User
    let mut updated_user = user.clone();
    updated_user.role = UserRole::TenantUser;
    store.update_user(updated_user.clone()).await.expect("Failed to update");
    
    let fetched_updated = store.get_user(user.id).await.unwrap().unwrap();
    assert_eq!(fetched_updated.role, UserRole::TenantUser);

    // Create Role
    use pangolin_core::permission::{Role, Permission, Action, PermissionScope};
    use std::collections::HashSet;
    
    let role = Role {
        id: Uuid::new_v4(),
        tenant_id: tenant.id,
        name: "DataEngineer".to_string(),
        description: Some("Manage tables".to_string()),
        permissions: vec![],
        created_by: user.id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    store.create_role(role.clone()).await.expect("Failed to create role");
    
    // Assign Role
    let assignment = pangolin_core::permission::UserRole {
        user_id: user.id,
        role_id: role.id,
        assigned_by: user.id,
        assigned_at: chrono::Utc::now(),
    };
    store.assign_role(assignment).await.expect("Failed to assign role");
    
    // Verify Assignment
    let roles = store.get_user_roles(user.id).await.unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role_id, role.id);
    
    // Delete User
    store.delete_user(user.id).await.expect("Failed to delete user");
    assert!(store.get_user(user.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_sqlite_access_requests() {
    let store = setup_store().await;

    // Setup: Tenant
    let tenant = Tenant {
        id: Uuid::new_v4(),
        name: "request_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Create tenant");

    // Setup: User (Fully Initialized)
    let user = User {
        id: Uuid::new_v4(),
        username: "request_user".to_string(),
        email: "req@example.com".to_string(),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant.id),
        role: UserRole::TenantUser,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user.clone()).await.expect("Create user");

    // Setup: Asset & Hierarchy
    let catalog = Catalog {
        id: Uuid::new_v4(),
        // tenant_id passed to method
        name: "def".to_string(),
        warehouse_name: None,
        storage_location: None,
        catalog_type: CatalogType::Local,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant.id, catalog.clone()).await.expect("Create catalog");

    let namespace = Namespace {
        // id, tenant_id, catalog_name not in struct
        name: vec!["default".to_string()],
        properties: HashMap::new(),
    };
    store.create_namespace(tenant.id, "def", namespace.clone()).await.expect("Create namespace");

    let asset = Asset {
        id: Uuid::new_v4(),
        name: "secret_asset".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/secret".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant.id, "def", None, vec!["default".to_string()], asset.clone())
        .await
        .expect("Create asset");

    // 1. Create Request
    let mut request = AccessRequest::new(tenant.id, user.id, asset.id, Some("Need access".to_string()));
    store.create_access_request(request.clone()).await.expect("Create request");

    // 2. Get Request
    let fetched = store.get_access_request(request.id).await.expect("Get request");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().status, RequestStatus::Pending);

    // 3. List Requests
    let list = store.list_access_requests(tenant.id).await.expect("List requests");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, request.id);

    // 4. Update Request (Approve)
    let admin_id = Uuid::new_v4();
    request.approve(admin_id, Some("Granted".to_string()));
    store.update_access_request(request.clone()).await.expect("Update request");

    let updated = store.get_access_request(request.id).await.expect("Get updated");
    assert_eq!(updated.unwrap().status, RequestStatus::Approved);
}

#[tokio::test]
async fn test_sqlite_list_user_permissions_aggregation() {
    let store = setup_store().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();

    // 0. Setup Tenant
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.expect("Failed to create tenant");

    // 0. Setup User (Foreign key requirement)
    let user = User {
        id: user_id,
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
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

    use pangolin_core::permission::{Action, PermissionScope, Role, UserRole as UserRoleAssignment, Permission};
    use std::collections::HashSet;

    // 1. Create a Role with permissions
    let mut role = Role::new("test-role".to_string(), None, tenant_id, admin_id);
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

