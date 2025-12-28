use crate::CatalogStore;
use pangolin_core::model::*;
use uuid::Uuid;
use std::collections::HashMap;

pub async fn test_bulk_ops_and_ancestry<S: CatalogStore>(store: &S) {
    let tenant_id = Uuid::new_v4();
    let catalog_name = "bulk_test_cat";
    
    // 0. Setup Tenant
    let tenant = Tenant {
        id: tenant_id,
        name: format!("tenant_{}", tenant_id),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.expect("Failed to create tenant");
    
    // 1. Setup Warehouse & Catalog
    let warehouse = Warehouse {
        id: Uuid::new_v4(),
        name: "wh_bulk".to_string(),
        tenant_id,
        storage_config: HashMap::new(),
        use_sts: false,
        vending_strategy: None,
    };
    let _ = store.create_warehouse(tenant_id, warehouse).await; // Ignore if exists
    
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("wh_bulk".to_string()),
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog.clone()).await.expect("Failed to create catalog");
    
    // 2. Create Namespace
    let ns = Namespace { name: vec!["default".to_string()], properties: HashMap::new() };
    store.create_namespace(tenant_id, catalog_name, ns).await.expect("Failed to create namespace");

    // 3. Create 5 Assets in "main"
    let src_branch = "main";
    for i in 0..5 {
        let asset = Asset {
            id: Uuid::new_v4(),
            name: format!("table_{}", i),
            kind: AssetType::IcebergTable,
            location: format!("s3://bucket/table_{}", i),
            properties: HashMap::new(),
        };
        store.create_asset(tenant_id, catalog_name, Some(src_branch.to_string()), vec!["default".to_string()], asset).await.expect("Failed to create asset");
        
        // Creating asset should create commit? 
        // Note: Generic create_asset doesn't necessarily create commits in all stores unless explicitly implemented.
        // But we will create dummy commits to test ancestry if needed.
    }
    
    // 4. Test Bulk Copy
    let dest_branch = "dev";
    
    // Ensure dest branch exists (MemoryStore creates on fly, but standardized parity might require it)
    let branch_obj = Branch {
        name: dest_branch.to_string(),
        head_commit_id: None,
        branch_type: BranchType::Experimental,
        assets: vec![],
    };
    store.create_branch(tenant_id, catalog_name, branch_obj).await.expect("Failed to create dest branch");
    
    let copied = store.copy_assets_bulk(tenant_id, catalog_name, src_branch, dest_branch, None).await.expect("Bulk copy failed");
    assert_eq!(copied, 5, "Should copy 5 assets");
    
    // Verify assets in destination
    let dest_assets = store.list_assets(tenant_id, catalog_name, Some(dest_branch.to_string()), vec!["default".to_string()], None).await.expect("Failed list_assets");
    assert_eq!(dest_assets.len(), 5, "Destination should have 5 assets");
    
    // 5. Test Commit Ancestry
    // We need to manually inject linked commits to test traversal
    // Create 3 linked commits
    let c1_id = Uuid::new_v4();
    let c2_id = Uuid::new_v4();
    let c3_id = Uuid::new_v4();
    
    let c1 = Commit {
        id: c1_id,
        parent_id: None,
        timestamp: 100,
        author: "tester".to_string(),
        message: "init".to_string(),
        operations: vec![],
    };
    let c2 = Commit {
        id: c2_id,
        parent_id: Some(c1_id),
        timestamp: 101,
        author: "tester".to_string(),
        message: "second".to_string(),
        operations: vec![],
    };
    let c3 = Commit {
        id: c3_id,
        parent_id: Some(c2_id),
        timestamp: 102,
        author: "tester".to_string(),
        message: "third".to_string(),
        operations: vec![],
    };
    
    store.create_commit(tenant_id, c1).await.expect("Failed c1");
    store.create_commit(tenant_id, c2).await.expect("Failed c2");
    store.create_commit(tenant_id, c3).await.expect("Failed c3");
    
    // Test Ancestry from Head (c3)
    let ancestry = store.get_commit_ancestry(tenant_id, c3_id, 10).await.expect("Failed ancestry");
    assert_eq!(ancestry.len(), 3, "Ancestry should have 3 commits");
    assert_eq!(ancestry[0].id, c3_id);
    assert_eq!(ancestry[1].id, c2_id);
    assert_eq!(ancestry[2].id, c1_id);
    
    // Test Limit
    let ancestry_limited = store.get_commit_ancestry(tenant_id, c3_id, 2).await.expect("Failed ancestry limit");
    assert_eq!(ancestry_limited.len(), 2, "Limited ancestry should have 2 commits");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryStore;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_memory_store_bulk_ops() {
        let store = MemoryStore::new();
        test_bulk_ops_and_ancestry(&store).await;
    }

    #[tokio::test]
    async fn test_sqlite_store_bulk_ops() {
        use crate::sqlite::SqliteStore;
        let store = SqliteStore::new("sqlite://:memory:").await.unwrap();
        store.apply_schema(include_str!("../../sql/sqlite_schema.sql")).await.unwrap();
        test_bulk_ops_and_ancestry(&store).await;
    }
}
