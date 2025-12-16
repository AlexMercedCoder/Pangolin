use pangolin_store::{PostgresStore, CatalogStore};
use pangolin_core::model::{Tenant, Catalog, Namespace, Asset, AssetType};
use uuid::Uuid;
use std::collections::HashMap;
use std::env;
use pangolin_core::user::{User, UserRole};
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};

#[tokio::test]
async fn test_postgres_store_flow() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_postgres_store_flow: DATABASE_URL not set");
            return;
        }
    };

    let store = PostgresStore::new(&connection_string).await.expect("Failed to create PostgresStore");

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
        id: Uuid::new_v4(),
        name: "test_catalog".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: Some("test_warehouse".to_string()),
        storage_location: Some("s3://bucket/path".to_string()),
        federated_config: None,
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
        id: Uuid::new_v4(),
        name: "test_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/test".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant_id, "test_catalog", None, namespace.name.clone(), asset.clone()).await.expect("Failed to create asset");

    let fetched_asset = store.get_asset(tenant_id, "test_catalog", None, namespace.name.clone(), "test_table".to_string()).await.expect("Failed to get asset");
    assert_eq!(fetched_asset.unwrap().name, "test_table");

    // 5. List Assets
    let assets = store.list_assets(tenant_id, "test_catalog", None, namespace.name.clone()).await.expect("Failed to list assets");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "test_table");
    assert_eq!(assets[0].name, "test_table");
}

#[tokio::test]
async fn test_postgres_access_requests() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_postgres_access_requests: DATABASE_URL not set");
            return;
        }
    };

    let store = PostgresStore::new(&connection_string).await.expect("Failed to create PostgresStore");

    // Setup: Tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "req_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant.clone()).await.expect("Create tenant");

    // Setup: User
    let user = User {
        id: Uuid::new_v4(),
        username: "req_user".to_string(),
        email: "req_pg@example.com".to_string(),
        password_hash: None,
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant.id),
        role: UserRole::TenantUser,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        active: true,
    };
    store.create_user(user.clone()).await.expect("Create user");

    // Setup: Asset
    // Create Catalog/Namespace first to respect foreign keys if strict
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "req_catalog".to_string(),
        catalog_type: pangolin_core::model::CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, catalog.clone()).await.expect("Create catalog");

    let namespace = Namespace {
        name: vec!["default".to_string()],
        properties: HashMap::new(),
    };
    store.create_namespace(tenant_id, "req_catalog", namespace.clone()).await.expect("Create namespace");

    let asset = Asset {
        id: Uuid::new_v4(),
        name: "secret_table".to_string(),
        kind: AssetType::IcebergTable,
        location: "s3://bucket/sec".to_string(),
        properties: HashMap::new(),
    };
    store.create_asset(tenant_id, "req_catalog", None, vec!["default".to_string()], asset.clone()).await.expect("Create asset");
    
    // 1. Create Request
    let mut request = AccessRequest::new(user.id, asset.id, Some("Access please".to_string()));
    store.create_access_request(request.clone()).await.expect("Create request");

    // 2. Get Request
    let fetched = store.get_access_request(request.id).await.expect("Get request");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().status, RequestStatus::Pending);

    // 3. List
    let list = store.list_access_requests(tenant_id).await.expect("List requests");
    assert!(!list.is_empty());

    // 4. Update
    request.status = RequestStatus::Approved;
    request.reviewed_by = Some(Uuid::new_v4()); // Dummy reviewer
    request.reviewed_at = Some(chrono::Utc::now());
    store.update_access_request(request.clone()).await.expect("Update request");

    let updated = store.get_access_request(request.id).await.expect("Get updated");
    assert_eq!(updated.unwrap().status, RequestStatus::Approved);
}
