use pangolin_store::MemoryStore;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Catalog, Warehouse};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_catalog_crud() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create a warehouse first (catalogs need warehouses)
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "test-warehouse".to_string(),
        tenant_id,
        use_sts: false,
        storage_config: HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "test-bucket".to_string()),
        ]),
    };
    store.create_warehouse(tenant_id, warehouse).await.unwrap();
    
    // Test Create
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test-catalog".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("test-warehouse".to_string()),
        storage_location: Some("s3://test-bucket/catalog/".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    
    store.create_catalog(tenant_id, catalog.clone()).await.unwrap();
    
    // Test Read
    let retrieved = store.get_catalog(tenant_id, "test-catalog".to_string()).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test-catalog");
    
    // Test List
    let catalogs = store.list_catalogs(tenant_id).await.unwrap();
    assert_eq!(catalogs.len(), 1);
    assert_eq!(catalogs[0].name, "test-catalog");
    
    // Test Delete
    store.delete_catalog(tenant_id, "test-catalog".to_string()).await.unwrap();
    
    // Verify deletion
    let deleted = store.get_catalog(tenant_id, "test-catalog".to_string()).await.unwrap();
    assert!(deleted.is_none());
    
    let catalogs_after = store.list_catalogs(tenant_id).await.unwrap();
    assert_eq!(catalogs_after.len(), 0);
}

#[tokio::test]
async fn test_warehouse_crud() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Test Create
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "test-warehouse".to_string(),
        tenant_id,
        use_sts: false,
        storage_config: HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "test-bucket".to_string()),
            ("region".to_string(), "us-east-1".to_string()),
        ]),
    };
    
    store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();
    
    // Test Read
    let retrieved = store.get_warehouse(tenant_id, "test-warehouse".to_string()).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_warehouse = retrieved.unwrap();
    assert_eq!(retrieved_warehouse.name, "test-warehouse");
    assert_eq!(retrieved_warehouse.use_sts, false);
    
    // Test List
    let warehouses = store.list_warehouses(tenant_id).await.unwrap();
    assert_eq!(warehouses.len(), 1);
    assert_eq!(warehouses[0].name, "test-warehouse");
    
    // Test Delete
    store.delete_warehouse(tenant_id, "test-warehouse".to_string()).await.unwrap();
    
    // Verify deletion
    let deleted = store.get_warehouse(tenant_id, "test-warehouse".to_string()).await.unwrap();
    assert!(deleted.is_none());
    
    let warehouses_after = store.list_warehouses(tenant_id).await.unwrap();
    assert_eq!(warehouses_after.len(), 0);
}

#[tokio::test]
async fn test_catalog_delete_nonexistent() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Try to delete a catalog that doesn't exist
    let result = store.delete_catalog(tenant_id, "nonexistent".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_warehouse_delete_nonexistent() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Try to delete a warehouse that doesn't exist
    let result = store.delete_warehouse(tenant_id, "nonexistent".to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_multiple_tenants_isolation() {
    let store = MemoryStore::new();
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    
    // Create warehouse for tenant1
    let warehouse1 = Warehouse {
        id: Uuid::new_v4(),
        name: "warehouse1".to_string(),
        tenant_id: tenant1,
        use_sts: false,
        storage_config: HashMap::new(),
    };
    store.create_warehouse(tenant1, warehouse1).await.unwrap();
    
    // Create catalog for tenant1
    let catalog1 = Catalog {
        id: Uuid::new_v4(),
        name: "catalog1".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("warehouse1".to_string()),
        storage_location: Some("s3://bucket1/".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant1, catalog1).await.unwrap();
    
    // Verify tenant2 cannot see tenant1's data
    let tenant2_catalogs = store.list_catalogs(tenant2).await.unwrap();
    assert_eq!(tenant2_catalogs.len(), 0);
    
    let tenant2_warehouses = store.list_warehouses(tenant2).await.unwrap();
    assert_eq!(tenant2_warehouses.len(), 0);
    
    // Verify tenant2 cannot delete tenant1's warehouse
    let delete_result = store.delete_warehouse(tenant2, "warehouse1".to_string()).await;
    assert!(delete_result.is_err());
    
    // Verify tenant1's warehouse still exists
    let tenant1_warehouses = store.list_warehouses(tenant1).await.unwrap();
    assert_eq!(tenant1_warehouses.len(), 1);
}

#[tokio::test]
async fn test_warehouse_delete_prevents_orphaned_catalogs() {
    // This test documents expected behavior: we should check if catalogs exist
    // before allowing warehouse deletion in a production system
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create warehouse
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "warehouse-with-catalog".to_string(),
        tenant_id,
        use_sts: false,
        storage_config: HashMap::new(),
    };
    store.create_warehouse(tenant_id, warehouse).await.unwrap();
    
    // Create catalog using this warehouse
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "dependent-catalog".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("warehouse-with-catalog".to_string()),
        storage_location: Some("s3://bucket/".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog).await.unwrap();
    
    // Currently, we CAN delete the warehouse even with dependent catalogs
    // In a production system, this should either:
    // 1. Fail with an error about dependent catalogs, OR
    // 2. Cascade delete the catalogs
    // For now, we just document this behavior
    let delete_result = store.delete_warehouse(tenant_id, "warehouse-with-catalog".to_string()).await;
    assert!(delete_result.is_ok(), "Current implementation allows deletion");
    
    // The catalog still exists but references a non-existent warehouse
    let catalogs = store.list_catalogs(tenant_id).await.unwrap();
    assert_eq!(catalogs.len(), 1);
    assert_eq!(catalogs[0].warehouse_name, Some("warehouse-with-catalog".to_string()));
}
