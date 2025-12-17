use crate::CatalogStore;
use pangolin_core::model::*;
use uuid::Uuid;
use std::collections::HashMap;

pub async fn test_asset_update_consistency<S: CatalogStore>(store: &S) {
    let tenant_id = Uuid::new_v4();
    let catalog = "default";
    let namespace = vec!["ns1".to_string()];
    let table = "tbl1".to_string();
    
    // 1. Setup Hierarchy (Tenant -> Warehouse -> Catalog -> Namespace)
    // Some stores enforce referential integrity
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "warehouse1".to_string(),
        tenant_id,
        storage_config: HashMap::new(),
        use_sts: false,
    };
    let _ = store.create_warehouse(tenant_id, warehouse.clone()).await; // Ignore error if exists/unimplemented checking

    let catalog_obj = Catalog {
        id: Uuid::new_v4(),
        name: catalog.to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("warehouse1".to_string()),
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    let _ = store.create_catalog(tenant_id, catalog_obj).await;

    let ns_obj = Namespace {
        name: namespace.clone(),
        properties: HashMap::new(),
    };
    let _ = store.create_namespace(tenant_id, catalog, ns_obj).await;
    
    // 2. Create Asset
    let initial_loc = "s3://bucket/path/v1.json".to_string();
    let asset = Asset {
        id: Uuid::new_v4(),
        name: table.clone(),
        kind: AssetType::IcebergTable,
        location: initial_loc.clone(), 
        properties: HashMap::new(),
    };
    
    store.create_asset(tenant_id, catalog, None, namespace.clone(), asset.clone()).await.expect("Failed to create asset");
    
    // 3. Verify Initial State
    // Check get_metadata_location
    let loc = store.get_metadata_location(tenant_id, catalog, None, namespace.clone(), table.clone()).await.expect("Failed to get metadata location");
    assert_eq!(loc, Some(initial_loc.clone()), "Initial metadata location mismatch");
    
    // 4. Update Metadata Location (CAS Success)
    let new_loc = "s3://bucket/path/v2.json".to_string();
    store.update_metadata_location(tenant_id, catalog, None, namespace.clone(), table.clone(), Some(initial_loc.clone()), new_loc.clone()).await.expect("Failed to update metadata location");
    
    // 5. Verify Updated State
    let updated_loc = store.get_metadata_location(tenant_id, catalog, None, namespace.clone(), table.clone()).await.expect("Failed to get updated metadata location");
    assert_eq!(updated_loc, Some(new_loc.clone()), "Updated metadata location mismatch");
    
    // Verify Asset Properties also updated (Consistency Check)
    let fetched_asset = store.get_asset(tenant_id, catalog, None, namespace.clone(), table.clone()).await.expect("Failed to get asset").expect("Asset not found");
    // Some stores might not update .location field if they prioritize properties, checking both
    if fetched_asset.location != new_loc {
         // If location field is not updated, ensure property IS
         assert_eq!(fetched_asset.properties.get("metadata_location"), Some(&new_loc), "Asset property metadata_location not updated");
    } else {
         assert_eq!(fetched_asset.location, new_loc, "Asset location field not updated");
    }

    // 6. CAS Failure Check
    let wrong_expected = "s3://bucket/path/WRONG.json".to_string();
    let v3_loc = "s3://bucket/path/v3.json".to_string();
    let result = store.update_metadata_location(tenant_id, catalog, None, namespace.clone(), table.clone(), Some(wrong_expected), v3_loc.clone()).await;
    assert!(result.is_err(), "CAS should fail with wrong expected location");
    
    // Verify it didn't change
    let current_loc = store.get_metadata_location(tenant_id, catalog, None, namespace.clone(), table.clone()).await.unwrap();
    assert_eq!(current_loc, Some(new_loc), "Location should not change on CAS failure");
}
