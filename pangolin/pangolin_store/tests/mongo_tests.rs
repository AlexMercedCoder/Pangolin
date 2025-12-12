use pangolin_store::{MongoStore, CatalogStore};
use pangolin_core::model::{Tenant, Catalog, Namespace, Asset, AssetType, Branch};
use uuid::Uuid;
use std::collections::HashMap;
use std::env;


#[tokio::test]
async fn test_mongo_store_flow() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_mongo_store_flow: DATABASE_URL not set");
            return;
        }
    };

    let store = MongoStore::new(&connection_string).await.expect("Failed to create MongoStore");

    // 1. Create Tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Failed to create tenant");

    let fetched_tenant = store.get_tenant(tenant_id).await.expect("Failed to get tenant");
    assert_eq!(fetched_tenant.unwrap().name, "test_tenant");

    // 2. Create Catalog
    let catalog = Catalog {
        name: "test_catalog".to_string(),
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
        name: "test_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/path".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant_id, "test_catalog", None, namespace.name.clone(), asset.clone()).await.expect("Failed to create asset");

    let fetched_asset = store.get_asset(tenant_id, "test_catalog", None, namespace.name.clone(), "test_table".to_string()).await.expect("Failed to get asset");
    assert_eq!(fetched_asset.unwrap().name, "test_table");

    // 5. List Assets
    let assets = store.list_assets(tenant_id, "test_catalog", None, namespace.name.clone()).await.expect("Failed to list assets");
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
}
