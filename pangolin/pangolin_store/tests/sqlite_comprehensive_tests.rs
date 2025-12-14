use pangolin_store::{SqliteStore, CatalogStore};
use pangolin_core::model::*;
use uuid::Uuid;
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
        storage_config: HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "test".to_string()),
        ]),
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
