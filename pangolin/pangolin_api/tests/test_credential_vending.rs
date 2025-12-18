use pangolin_api::signing_handlers::*;
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Tenant, Warehouse, Catalog, CatalogType, VendingStrategy};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_aws_sts_vending_strategy() {
    let store = Arc::new(MemoryStore::new());
    
    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    // Create warehouse with STS vending
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        tenant_id,
        name: "sts_warehouse".to_string(),
        use_sts: true,
        storage_config: HashMap::from([
            ("s3.bucket".to_string(), "test-bucket".to_string()),
            ("s3.region".to_string(), "us-east-1".to_string()),
        ]),
        vending_strategy: Some(VendingStrategy::AwsSts),
    };
    store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();
    
    // Verify warehouse configuration
    let retrieved = store.get_warehouse(tenant_id, "sts_warehouse".to_string()).await.unwrap();
    assert!(retrieved.is_some());
    let wh = retrieved.unwrap();
    assert_eq!(wh.vending_strategy, Some(VendingStrategy::AwsSts));
    assert_eq!(wh.use_sts, true);
}

#[tokio::test]
async fn test_aws_static_vending_strategy() {
    let store = Arc::new(MemoryStore::new());
    
    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    // Create warehouse with static credentials
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        tenant_id,
        name: "static_warehouse".to_string(),
        use_sts: false,
        storage_config: HashMap::from([
            ("s3.bucket".to_string(), "test-bucket".to_string()),
            ("s3.access-key-id".to_string(), "AKIAIOSFODNN7EXAMPLE".to_string()),
            ("s3.secret-access-key".to_string(), "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string()),
        ]),
        vending_strategy: Some(VendingStrategy::AwsStatic),
    };
    store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();
    
    // Verify warehouse configuration
    let retrieved = store.get_warehouse(tenant_id, "static_warehouse".to_string()).await.unwrap();
    assert!(retrieved.is_some());
    let wh = retrieved.unwrap();
    assert_eq!(wh.vending_strategy, Some(VendingStrategy::AwsStatic));
    assert_eq!(wh.use_sts, false);
}

#[tokio::test]
async fn test_bucket_resolution_from_warehouse() {
    let store = Arc::new(MemoryStore::new());
    
    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    // Create warehouse with explicit bucket
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        tenant_id,
        name: "bucket_warehouse".to_string(),
        use_sts: false,
        storage_config: HashMap::from([
            ("s3.bucket".to_string(), "my-specific-bucket".to_string()),
        ]),
        vending_strategy: None,
    };
    store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();
    
    // Verify bucket is correctly stored
    let retrieved = store.get_warehouse(tenant_id, "bucket_warehouse".to_string()).await.unwrap();
    assert!(retrieved.is_some());
    let wh = retrieved.unwrap();
    assert_eq!(wh.storage_config.get("s3.bucket"), Some(&"my-specific-bucket".to_string()));
}
