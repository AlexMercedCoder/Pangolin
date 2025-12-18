use pangolin_api::federated_proxy::FederatedCatalogProxy;
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use pangolin_core::model::{Tenant, Catalog, CatalogType, FederatedCatalogConfig};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_federated_catalog_identification() {
    let store = Arc::new(MemoryStore::new());
    
    // Create tenant
    let tenant_id = Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        name: "test_tenant".to_string(),
        properties: HashMap::new(),
    };
    store.create_tenant(tenant).await.unwrap();
    
    // Create local catalog
    let local_catalog = Catalog {
        id: Uuid::new_v4(),
        name: "local_catalog".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("test_warehouse".to_string()),
        storage_location: Some("s3://bucket/path".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, local_catalog).await.unwrap();
    
    // Create federated catalog
    let federated_catalog = Catalog {
        id: Uuid::new_v4(),
        name: "federated_catalog".to_string(),
        catalog_type: CatalogType::Federated,
        warehouse_name: None,
        storage_location: None,
        federated_config: Some(FederatedCatalogConfig {
            remote_url: "http://remote-catalog:8080".to_string(),
            auth_token: Some("test-token".to_string()),
            timeout_seconds: 30,
        }),
        properties: HashMap::new(),
    };
    store.create_catalog(tenant_id, federated_catalog).await.unwrap();
    
    // Verify local catalog is not federated
    let local_cat = store.get_catalog(tenant_id, "local_catalog".to_string()).await.unwrap();
    assert!(local_cat.is_some());
    assert_eq!(local_cat.unwrap().catalog_type, CatalogType::Local);
    
    // Verify federated catalog is correctly identified
    let fed_cat = store.get_catalog(tenant_id, "federated_catalog".to_string()).await.unwrap();
    assert!(fed_cat.is_some());
    let fed_cat = fed_cat.unwrap();
    assert_eq!(fed_cat.catalog_type, CatalogType::Federated);
    assert!(fed_cat.federated_config.is_some());
}

#[tokio::test]
async fn test_federated_proxy_url_construction() {
    let config = FederatedCatalogConfig {
        remote_url: "http://remote-catalog:8080".to_string(),
        auth_token: Some("test-token".to_string()),
        timeout_seconds: 30,
    };
    
    let proxy = FederatedCatalogProxy::new(config);
    
    // Test URL construction for different endpoints
    let list_namespaces_url = proxy.build_url("/v1/test_catalog/namespaces");
    assert!(list_namespaces_url.contains("http://remote-catalog:8080"));
    assert!(list_namespaces_url.contains("/v1/test_catalog/namespaces"));
}

#[tokio::test]
async fn test_federated_config_validation() {
    // Valid config
    let valid_config = FederatedCatalogConfig {
        remote_url: "http://valid-url:8080".to_string(),
        auth_token: Some("token".to_string()),
        timeout_seconds: 30,
    };
    assert!(!valid_config.remote_url.is_empty());
    
    // Config without auth token (should still be valid)
    let no_auth_config = FederatedCatalogConfig {
        remote_url: "http://public-catalog:8080".to_string(),
        auth_token: None,
        timeout_seconds: 30,
    };
    assert!(!no_auth_config.remote_url.is_empty());
}
