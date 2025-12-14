use axum::http::StatusCode;
use pangolin_api::*;
use pangolin_core::model::{Catalog, CatalogType, FederatedAuthType, FederatedCatalogConfig, FederatedCredentials, Tenant};
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Test cross-tenant federation: Tenant A connects to Tenant B's catalog
#[tokio::test]
async fn test_cross_tenant_federation() {
    let store = Arc::new(MemoryStore::new());

    // Create Tenant A
    let tenant_a = Tenant {
        id: Uuid::new_v4(),
        name: "tenant_a".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant_a.clone()).await.unwrap();

    // Create Tenant B
    let tenant_b = Tenant {
        id: Uuid::new_v4(),
        name: "tenant_b".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant_b.clone()).await.unwrap();

    // Create a local catalog in Tenant B
    let tenant_b_catalog = Catalog {
        id: Uuid::new_v4(),
        name: "production".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("default".to_string()),
        storage_location: Some("s3://bucket/tenant_b".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store
        .create_catalog(tenant_b.id, tenant_b_catalog.clone())
        .await
        .unwrap();

    // Create a federated catalog in Tenant A pointing to Tenant B
    // In a real scenario, this would point to Tenant B's Pangolin instance
    // For this test, we'll simulate it pointing to localhost
    let federated_catalog = Catalog {
        id: Uuid::new_v4(),
        name: "partner_production".to_string(),
        catalog_type: CatalogType::Federated,
        warehouse_name: None,
        storage_location: None,
        federated_config: Some(FederatedCatalogConfig {
            base_url: "http://localhost:8080".to_string(), // Would be Tenant B's URL
            auth_type: FederatedAuthType::ApiKey,
            credentials: Some(FederatedCredentials {
                username: None,
                password: None,
                token: None,
                api_key: Some("tenant_b_service_user_key_xyz123".to_string()),
            }),
            timeout_seconds: 30,
        }),
        properties: HashMap::new(),
    };
    store
        .create_catalog(tenant_a.id, federated_catalog.clone())
        .await
        .unwrap();

    // Verify catalogs were created
    let tenant_a_catalogs = store.list_catalogs(tenant_a.id).await.unwrap();
    assert_eq!(tenant_a_catalogs.len(), 1);
    assert_eq!(tenant_a_catalogs[0].name, "partner_production");
    assert_eq!(tenant_a_catalogs[0].catalog_type, CatalogType::Federated);

    let tenant_b_catalogs = store.list_catalogs(tenant_b.id).await.unwrap();
    assert_eq!(tenant_b_catalogs.len(), 1);
    assert_eq!(tenant_b_catalogs[0].name, "production");
    assert_eq!(tenant_b_catalogs[0].catalog_type, CatalogType::Local);

    // Verify federated catalog configuration
    let retrieved_federated = store
        .get_catalog(tenant_a.id, "partner_production")
        .await
        .unwrap()
        .unwrap();
    assert!(retrieved_federated.federated_config.is_some());
    let config = retrieved_federated.federated_config.unwrap();
    assert_eq!(config.base_url, "http://localhost:8080");
    assert_eq!(config.auth_type, FederatedAuthType::ApiKey);
    assert_eq!(config.timeout_seconds, 30);

    println!("✅ Cross-tenant federation test passed!");
    println!("   - Tenant A created with federated catalog 'partner_production'");
    println!("   - Tenant B created with local catalog 'production'");
    println!("   - Federated catalog configured to connect to Tenant B");
    println!("   - In production, requests to 'partner_production' would be forwarded to Tenant B");
}

/// Test federated catalog CRUD operations
#[tokio::test]
async fn test_federated_catalog_crud() {
    let store = Arc::new(MemoryStore::new());

    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();

    // Create federated catalog
    let federated_catalog = Catalog {
        id: Uuid::new_v4(),
        name: "external_catalog".to_string(),
        catalog_type: CatalogType::Federated,
        warehouse_name: None,
        storage_location: None,
        federated_config: Some(FederatedCatalogConfig {
            base_url: "https://external.example.com".to_string(),
            auth_type: FederatedAuthType::BearerToken,
            credentials: Some(FederatedCredentials {
                username: None,
                password: None,
                token: Some("jwt_token_xyz".to_string()),
                api_key: None,
            }),
            timeout_seconds: 60,
        }),
        properties: HashMap::new(),
    };

    // Create
    store
        .create_catalog(tenant_id, federated_catalog.clone())
        .await
        .unwrap();

    // Read
    let retrieved = store
        .get_catalog(tenant_id, "external_catalog")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.name, "external_catalog");
    assert_eq!(retrieved.catalog_type, CatalogType::Federated);

    // List
    let catalogs = store.list_catalogs(tenant_id).await.unwrap();
    assert_eq!(catalogs.len(), 1);
    assert_eq!(catalogs[0].catalog_type, CatalogType::Federated);

    // Delete
    store
        .delete_catalog(tenant_id, "external_catalog")
        .await
        .unwrap();
    let deleted = store
        .get_catalog(tenant_id, "external_catalog")
        .await
        .unwrap();
    assert!(deleted.is_none());

    println!("✅ Federated catalog CRUD test passed!");
}
