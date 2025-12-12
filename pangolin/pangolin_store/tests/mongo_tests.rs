use pangolin_store::{MongoStore, CatalogStore};
use pangolin_core::model::{Tenant, Catalog, Namespace, Asset, AssetType};
use uuid::Uuid;
use std::collections::HashMap;
use std::env;

use futures::stream::TryStreamExt;

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
}
