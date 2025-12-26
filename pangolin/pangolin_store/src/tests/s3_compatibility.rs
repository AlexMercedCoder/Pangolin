#[cfg(test)]
mod tests {
    use crate::{CatalogStore, MemoryStore};
    use pangolin_core::model::Warehouse;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_memory_store_s3_fallback_logic() {
        let store = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        
        // Create a warehouse with "s3.bucket" instead of just "bucket"
        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "s3".to_string());
        storage_config.insert("s3.bucket".to_string(), "my-test-bucket".to_string());
        storage_config.insert("s3.region".to_string(), "us-east-1".to_string());
        storage_config.insert("s3.path-style-access".to_string(), "true".to_string());

        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            tenant_id,
            name: "test_warehouse".to_string(),
            storage_config,
            use_sts: false,
            vending_strategy: None,
        };

        store.create_warehouse(tenant_id, warehouse.clone()).await.expect("Failed to create warehouse");

        // Test 1: Lookup using a path that contains the bucket name
        // The logic searches for warehouses where config has 'bucket'/'s3.bucket' that is contained in the location string.
        let location = "s3://my-test-bucket/some/path/metadata.json";
        
        let found_warehouse = store.get_warehouse_for_location(location);
        
        assert!(found_warehouse.is_some(), "Should find warehouse based on s3.bucket match");
        let w = found_warehouse.unwrap();
        assert_eq!(w.name, "test_warehouse");
        
        // Verify s3.path-style-access is preserved
        assert_eq!(w.storage_config.get("s3.path-style-access").map(|s| s.as_str()), Some("true"));
    }

    #[tokio::test]
    async fn test_memory_store_legacy_bucket_logic() {
        let store = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        
        // Create a warehouse with standard "bucket"
        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "s3".to_string());
        storage_config.insert("bucket".to_string(), "legacy-bucket".to_string());

        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            tenant_id,
            name: "legacy_warehouse".to_string(),
            storage_config,
            use_sts: false,
            vending_strategy: None,
        };

        store.create_warehouse(tenant_id, warehouse.clone()).await.expect("Failed to create warehouse");

        let location = "s3://legacy-bucket/data/file.parquet";
        let found_warehouse = store.get_warehouse_for_location(location);
        
        assert!(found_warehouse.is_some(), "Should find warehouse based on legacy bucket match");
        assert_eq!(found_warehouse.unwrap().name, "legacy_warehouse");
    }
}
