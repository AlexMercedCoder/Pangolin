use pangolin_store::{MongoStore, CatalogStore};
use pangolin_core::model::{Tenant, Catalog, Namespace, Asset, AssetType, Branch, Warehouse};
use uuid::Uuid;
use std::collections::HashMap;
use std::env;

use pangolin_core::user::{User, UserRole};
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};

#[tokio::test]
async fn test_mongo_store_flow() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_mongo_store_flow: DATABASE_URL not set");
            return;
        }
    };

    let store = MongoStore::new(&connection_string, "admin").await.expect("Failed to create MongoStore");

    // Test Tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.unwrap();

    // Test Warehouse
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "test_warehouse".to_string(),
        tenant_id,
        storage_config: HashMap::new(),
        use_sts: false,
        vending_strategy: None,
    };
    store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();

    // 2. Create Catalog
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("test_warehouse".to_string()),
        storage_location: Some("s3://bucket/path".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog.clone()).await.expect("Failed to create catalog");

    // 3. Create Namespace
    let namespace = Namespace {
        name: vec!["db".to_string(), "schema".to_string()],
        properties: HashMap::new(),
    };
    store.create_namespace(tenant_id, "test_catalog", namespace.clone()).await.expect("Failed to create namespace");

    // 4. Create Asset
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "test_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/test".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant_id, "test_catalog", None, namespace.name.clone(), asset.clone()).await.expect("Failed to create asset");

    let fetched_asset = store.get_asset(tenant_id, "test_catalog", None, namespace.name.clone(), "test_table".to_string()).await.expect("Failed to get asset");
    assert_eq!(fetched_asset.unwrap().name, "test_table");

    // 5. List Assets
    let assets = store.list_assets(tenant_id, "test_catalog", None, namespace.name.clone(), None).await.expect("Failed to list assets");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "test_table");

    // 6. Update Namespace Properties
    let mut new_props = HashMap::new();
    new_props.insert("owner".to_string(), "data_team".to_string());
    store.update_namespace_properties(tenant_id, "test_catalog", namespace.name.clone(), new_props).await.expect("Failed to update namespace properties");
    let updated_namespace = store.get_namespace(tenant_id, "test_catalog", namespace.name.clone()).await.expect("Failed to get namespace").unwrap();
    assert_eq!(updated_namespace.properties.get("owner").unwrap(), "data_team");

    // 7. Create Branch
    let branch = Branch {
        name: "dev".to_string(),
        head_commit_id: None,
        branch_type: pangolin_core::model::BranchType::Experimental,
        assets: vec![],
    };
    store.create_branch(tenant_id, "test_catalog", branch.clone()).await.expect("Failed to create branch");
    let fetched_branch = store.get_branch(tenant_id, "test_catalog", "dev".to_string()).await.expect("Failed to get branch");
    assert_eq!(fetched_branch.unwrap().name, "dev");

    // 8. Rename Asset
    let dest_namespace = vec!["db".to_string(), "new_schema".to_string()];
    let new_namespace = Namespace {
        name: dest_namespace.clone(),
        properties: HashMap::new(),
    };
    store.create_namespace(tenant_id, "test_catalog", new_namespace.clone()).await.expect("Failed to create new namespace");
    
    store.rename_asset(tenant_id, "test_catalog", None, namespace.name.clone(), "test_table".to_string(), dest_namespace.clone(), "renamed_table".to_string()).await.expect("Failed to rename asset");
    
    let old_asset = store.get_asset(tenant_id, "test_catalog", None, namespace.name.clone(), "test_table".to_string()).await.expect("Failed to get old asset");
    assert!(old_asset.is_none());
    
    let new_asset = store.get_asset(tenant_id, "test_catalog", None, dest_namespace.clone(), "renamed_table".to_string()).await.expect("Failed to get new asset");
    assert_eq!(new_asset.unwrap().name, "renamed_table");

    // 9. Delete Asset
    store.delete_asset(tenant_id, "test_catalog", None, dest_namespace.clone(), "renamed_table".to_string()).await.expect("Failed to delete asset");
    let deleted_asset = store.get_asset(tenant_id, "test_catalog", None, dest_namespace.clone(), "renamed_table".to_string()).await.expect("Failed to get deleted asset");
    assert!(deleted_asset.is_none());

    // 10. Delete Namespace
    store.delete_namespace(tenant_id, "test_catalog", namespace.name.clone()).await.expect("Failed to delete namespace");
    let deleted_namespace = store.get_namespace(tenant_id, "test_catalog", namespace.name.clone()).await.expect("Failed to get deleted namespace");
    assert!(deleted_namespace.is_none());
    let deleted_namespace = store.get_namespace(tenant_id, "test_catalog", namespace.name.clone()).await.expect("Failed to get deleted namespace");
    assert!(deleted_namespace.is_none());
}

#[tokio::test]
async fn test_mongo_access_requests() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_mongo_access_requests: DATABASE_URL not set");
            return;
        }
    };

    let store = MongoStore::new(&connection_string, "admin_test").await.expect("Failed to create MongoStore");

    // Setup: Tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "req_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Create tenant");

    // Setup: User
    let user = User {
        id: Uuid::new_v4(),
        username: "req_user_mongo".to_string(),
        email: "req_mongo@example.com".to_string(),
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
    // Note: MongoStore needs `create_user` implementation.
    // I need to check if MongoStore implements create_user/get_user first!
    // existing mongo.rs had `users()` collection helper, but I didn't verify `create_user` method in `CatalogStore` impl or `MongoStore` impl.
    // Wait, CatalogStore DOES NOT have user methods.
    // User methods are usually separate or extension?
    // In `sqlite.rs`, `impl CatalogStore for SqliteStore` does NOT have `create_user`.
    // Use `impl SqliteStore` methods directly.
    // So I need to call `store.create_user`.
    // Does `MongoStore` have `create_user`?
    // I saw `fn users(&self)` helper.
    // I should verify `create_user` exists in `MongoStore` impl.
    // If not, I can't run this test fully. I might need to implement it.
    // FOR NOW, I will assume it exists or comment it out if generic, but Access Request relies on it.
    // `sqlite.rs` had `create_user` inside `impl SqliteStore`.
    
    // I'll proceed assuming I need to add `create_user` if missing.
    // The test logic relies on `AccessRequest` pointing to `user_id`.
    // In Mongo, relations are weak, so it might work without creating user if no FK constraint.
    // BUT `list_access_requests` uses lookup on `users`. So user MUST exist.
    
    // I will add `create_user` to `MongoStore` in `mongo.rs` if missing.
    // I'll assume I'll add it in next step if compilation fails, OR blindly add it now.
    // Let's add the test first.
    
    // Setup: Asset
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "req_catalog".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog.clone()).await.expect("Create catalog");

    let namespace = Namespace {
        name: vec!["default".to_string()],
        properties: HashMap::new(),
    };
    store.create_namespace(tenant_id, "req_catalog", namespace.clone()).await.expect("Create namespace");

    let asset = Asset {
        id: Uuid::new_v4(),
        name: "secret_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/sec".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant_id, "req_catalog", None, vec!["default".to_string()], asset.clone()).await.expect("Create asset");
    
    // 1. Create Request (Mocking user existence for now if create_user missing, but lookup needs it)
    // Actually, I should IMPLEMENT `create_user` in mongo.rs first to be safe.
    // I will finish this replacement first.
}
