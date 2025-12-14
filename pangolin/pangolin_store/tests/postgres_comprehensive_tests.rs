use pangolin_store::{PostgresStore, CatalogStore};
use pangolin_core::model::{Tenant, Warehouse, Catalog, CatalogType, Namespace, Asset, AssetType};
use uuid::Uuid;
use std::collections::HashMap;
use std::env;

const TEST_DB_URL: &str = "postgresql://pangolin:pangolin_dev_password@localhost/pangolin_test";

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
        name: warehouse_name.clone(),
        use_sts: false,
        storage_config: HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "test-bucket".to_string()),
        ]),
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
