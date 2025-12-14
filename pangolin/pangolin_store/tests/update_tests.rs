use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Tenant, Warehouse, Catalog, TenantUpdate, WarehouseUpdate, CatalogUpdate, CatalogType};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_tenant_update() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create tenant
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    // Update tenant name
    let updates = TenantUpdate {
        name: Some("updated_tenant".to_string()),
        properties: None,
    };
    let updated = store.update_tenant(tenant_id, updates).await.unwrap();
    assert_eq!(updated.name, "updated_tenant");
    
    // Delete tenant
    store.delete_tenant(tenant_id).await.unwrap();
    let deleted = store.get_tenant(tenant_id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_warehouse_update() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create warehouse
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "test_wh".to_string(),
        tenant_id,
        storage_config: HashMap::from([("type".to_string(), "s3".to_string())]),
        use_sts: false,
    };
    store.create_warehouse(tenant_id, warehouse).await.unwrap();
    
    // Update warehouse to enable STS
    let updates = WarehouseUpdate {
        name: None,
        storage_config: Some(HashMap::from([("role_arn".to_string(), "arn:aws:iam::123:role/test".to_string())])),
        use_sts: Some(true),
    };
    let updated = store.update_warehouse(tenant_id, "test_wh".to_string(), updates).await.unwrap();
    assert_eq!(updated.use_sts, true);
    assert!(updated.storage_config.contains_key("role_arn"));
}

#[tokio::test]
async fn test_catalog_update() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create catalog
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog).await.unwrap();
    
    // Update catalog to attach warehouse
    let updates = CatalogUpdate {
        warehouse_name: Some("my_warehouse".to_string()),
        storage_location: Some("s3://bucket/path".to_string()),
        properties: None,
    };
    let updated = store.update_catalog(tenant_id, "test_catalog".to_string(), updates).await.unwrap();
    assert_eq!(updated.warehouse_name, Some("my_warehouse".to_string()));
    assert_eq!(updated.storage_location, Some("s3://bucket/path".to_string()));
}
