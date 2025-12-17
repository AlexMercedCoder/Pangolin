use pangolin_store::{CatalogStore, MemoryStore};
use pangolin_core::model::{Tenant, Warehouse, Catalog, CatalogType, Namespace, Asset, AssetType};
use uuid::Uuid;
use std::collections::HashMap;
use std::env;

use pangolin_store::SqliteStore;
use pangolin_store::PostgresStore;
use pangolin_store::MongoStore;

// Helper to run tests against a store
async fn run_store_compliance_tests<S: CatalogStore>(store: &S, store_name: &str) {
    println!("Running compliance tests for: {}", store_name);
    
    // 1. Basic CRUD
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: format!("test_tenant_{}", Uuid::new_v4()),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("create_tenant failed");
    
    let retrieved = store.get_tenant(tenant_id).await.expect("get_tenant failed");
    assert_eq!(retrieved.unwrap().name, tenant.name, "Tenant name mismatch");

    // 2. Metadata Location Logic (Regression Check)
    // Create Catalog/Namespace
    let catalog_name = "test_catalog".to_string();
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.clone(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: Some("s3://bucket/cat".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog).await.expect("create_catalog failed");
    
    let ns = Namespace { name: vec!["db".to_string()], properties: HashMap::new() };
    store.create_namespace(tenant_id, &catalog_name, ns.clone()).await.expect("create_namespace failed");
    
    // Create Asset with Metadata Location Property
    let table_name = "test_table".to_string();
    let metadata_path = "s3://bucket/cat/db/test_table/metadata/v1.json".to_string();
    
    let mut props = HashMap::new();
    props.insert("metadata_location".to_string(), metadata_path.clone());
    
    let asset = Asset {
        id: Uuid::new_v4(),
        name: table_name.clone(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/cat/db/test_table".to_string(),
        properties: props,
    };
    
    // Testing Metadata Location Population Logic
    store.create_asset(tenant_id, &catalog_name, None, vec!["db".to_string()], asset.clone()).await.expect("create_asset failed");
    
    // Verify metadata_location was correctly populated in the backend
    // This requires inspecting the asset, but `get_asset` returns the struct which we populated.
    // However, `update_metadata_location` relies on the backend having the value correctly.
    // If we call update_metadata_location with expected=Some(metadata_path), it should succeed.
    // If the backend stored `location` instead of `metadata_location` (the bug), this update will fail (lock mismatch).
    
    let new_metadata_path = "s3://bucket/cat/db/test_table/metadata/v2.json".to_string();
    
    store.update_metadata_location(
        tenant_id, 
        &catalog_name, 
        None, 
        vec!["db".to_string()], 
        table_name.clone(), 
        Some(metadata_path.clone()), // Expected existing
        new_metadata_path.clone()    // New
    ).await.expect("update_metadata_location failed - Optimistic Locking/Metadata Location Regression!");

    // 3. File IO Check (Write/Read)
    // Only if the store supports it (Memory, Postgres, Sqlite updated)
    // We can test by writing a small file
    let file_path = format!("s3://bucket/test_{}.txt", Uuid::new_v4());
    let content = b"Hello World";
    
    match store.write_file(&file_path, content.to_vec()).await {
        Ok(_) => {
             let read_content = store.read_file(&file_path).await.expect("read_file failed");
             assert_eq!(read_content, content, "File content mismatch");
        },
        Err(e) => {
            println!("Store {} does not support File IO or failed: {}", store_name, e);
            // Verify if it SHOULD support it
            if store_name == "MemoryStore" || store_name == "PostgresStore" {
                 panic!("File IO should be supported for {}", store_name);
            }
        }
    }
    
    println!("{} passed compliance tests.", store_name);
}

#[tokio::test]
async fn test_memory_store_compliance() {
    let store = MemoryStore::new();
    run_store_compliance_tests(&store, "MemoryStore").await;
}


#[tokio::test]
async fn test_sqlite_store_compliance() {
    let db_path = format!("file:memdb_{}?mode=memory&cache=shared", Uuid::new_v4());
    let store = SqliteStore::new(&db_path).await.expect("Failed to create SqliteStore");
    
    // Apply schema
    let schema = include_str!("../sql/sqlite_schema.sql");
    store.apply_schema(schema).await.expect("Failed to apply schema");

    run_store_compliance_tests(&store, "SqliteStore").await;
}

#[tokio::test]
async fn test_mongo_store_compliance() {
    // Requires mongodb instance. Set DATABASE_URL to mongodb://... to run.
    let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
    if !db_url.starts_with("mongodb://") {
        println!("Skipping MongoStore tests: DATABASE_URL not set to mongodb://");
        return;
    }
    
    // Use a unique DB name for test isolation
    let db_name = format!("pangolin_test_{}", Uuid::new_v4());
    
    let store = MongoStore::new(&db_url, &db_name).await.expect("Failed to create MongoStore");
    // No schema migration needed for Mongo
    
    run_store_compliance_tests(&store, "MongoStore").await;
}

#[tokio::test]
async fn test_postgres_store_compliance() {
    // Only run if DATABASE_URL is set
    if let Ok(url) = env::var("DATABASE_URL") {
        match PostgresStore::new(&url).await {
            Ok(store) => run_store_compliance_tests(&store, "PostgresStore").await,
            Err(e) => println!("Skipping Postgres compliance test: {}", e),
        }
    } else {
        println!("Skipping Postgres compliance test (DATABASE_URL not set)");
    }
}
