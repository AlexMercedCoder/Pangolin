use pangolin_store::{CatalogStore, MemoryStore};
use pangolin_core::model::{Tenant, Catalog, Namespace, Asset, AssetType, Branch, BranchType, Commit, Tag};
use pangolin_core::business_metadata::BusinessMetadata;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

#[tokio::test]
async fn test_memory_store_search() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    
    // Create Tenant
    store.create_tenant(Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    }).await.unwrap();

    // Create a Catalog
    let catalog_id = Uuid::new_v4();
    
    // Create Warehouse first
    let mut storage_config = HashMap::new();
    storage_config.insert("type".to_string(), "s3".to_string());
    storage_config.insert("bucket".to_string(), "bucket".to_string());
    storage_config.insert("region".to_string(), "us-east-1".to_string());

    store.create_warehouse(tenant_id, pangolin_core::model::Warehouse {
        id: Uuid::new_v4(), // Warehouse needs ID
        tenant_id,
        name: "test_wh".to_string(),
        storage_config,
        use_sts: false,
        vending_strategy: None
    }).await.unwrap();

    // Now Create Catalog
    store.create_catalog(tenant_id, Catalog {
        id: catalog_id,
        name: "marketing".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("test_wh".to_string()),
        storage_location: Some("s3://bucket/marketing".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    }).await.unwrap();
    
    store.create_catalog(tenant_id, Catalog {
        id: Uuid::new_v4(),
        name: "engineering".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("test_wh".to_string()),
        storage_location: Some("s3://bucket/engineering".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    }).await.unwrap();

    // Create Namespace
    store.create_namespace(tenant_id, "marketing", Namespace {
        name: vec!["sales".to_string()],
        properties: HashMap::new(), 
    }).await.unwrap();
    
    store.create_namespace(tenant_id, "marketing", Namespace {
        name: vec!["hr".to_string()],
        properties: HashMap::new(),
    }).await.unwrap();

    // Create Asset
    let asset_id = Uuid::new_v4();
    store.create_asset(tenant_id, "marketing", None, vec!["sales".to_string()], Asset {
        id: asset_id,
        name: "revenue_q1".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/marketing/sales/revenue_q1".to_string(),
        properties: HashMap::new(), // Asset still uses HashMap in model.rs view? Wait.
        // Let's check Asset struct in previous view... it wasn't fully shown.
        // But `postgres.rs` used `from_value` for properties?
        // Let's assume HashMap for now, if error, verification will fail.
        // Actually `Asset` properties is often `HashMap<String, String>` in Iceberg.
    }).await.unwrap();

    // Insert Business Metadata
    store.upsert_business_metadata(BusinessMetadata {
        id: Uuid::new_v4(),
        asset_id,
        description: Some("Q1 Revenue Data".to_string()),
        tags: vec!["finance".to_string(), "quarterly".to_string()],
        properties: serde_json::json!({}),
        discoverable: true,
        created_by: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_by: Uuid::new_v4(),
        updated_at: Utc::now(),
    }).await.unwrap();

    // --- VERIFICATION ---

    // 4. Search Assets by Missing Tag
    let results = store.search_assets(tenant_id, "", Some(vec!["missing".to_string()])).await.unwrap();
    assert_eq!(results.len(), 0);

    // 1. Search Assets by Name
    let results = store.search_assets(tenant_id, "revenue", None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "revenue_q1");
    assert!(results[0].1.is_some()); // Metadata present
    assert_eq!(results[0].2, "marketing"); // Catalog correct
    assert_eq!(results[0].3, vec!["sales".to_string()]); // Namespace correct

    // 2. Search Assets by Metadata Description
    let results = store.search_assets(tenant_id, "Revenue Data", None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "revenue_q1");
    // Verifying context again mostly for sanity
    assert_eq!(results[0].2, "marketing");

    // 3. Search Assets by Tag
    let results = store.search_assets(tenant_id, "", Some(vec!["finance".to_string()])).await.unwrap();
    assert_eq!(results.len(), 1);

    // 5. Search Catalogs
    let catalogs = store.search_catalogs(tenant_id, "mark").await.unwrap();
    assert_eq!(catalogs.len(), 1);
    assert_eq!(catalogs[0].name, "marketing");

    // 6. Search Namespaces
    let namespaces = store.search_namespaces(tenant_id, "sale").await.unwrap();
    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces[0].0.name, vec!["sales"]);
    
    // 7. Search Branches (need to create one first)
    // MemoryStore branches map: (Uuid, String, String) -> Branch (tenant, catalog, branch_name)
    // create_branch signature in trait?
    // MemoryStore create_branch(tenant_id, catalog_name, branch)
    store.create_branch(tenant_id, "marketing", Branch {
        name: "dev-feature".to_string(),
        head_commit_id: None,
        branch_type: BranchType::Experimental,
        assets: Vec::new(),
    }).await.unwrap();
    
    let branches = store.search_branches(tenant_id, "dev").await.unwrap();
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].0.name, "dev-feature");

    println!("MemoryStore Search Tests Passed!");
}
