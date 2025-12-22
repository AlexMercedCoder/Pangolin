
use crate::common::{setup_test_app, TestApp};
use pangolin_core::model::{Catalog, CatalogType};
use pangolin_api::dashboard_handlers::DashboardStats;
use uuid::Uuid;
use std::collections::HashMap;

mod common;

#[tokio::test]
async fn test_dashboard_stats_counts() {
    let app = setup_test_app().await;
    let client = &app.client;

    // 1. Setup Tenant Admin session
    let tenant_id = app.tenant_id;
    let auth_headers = app.get_auth_headers();

    // 2. Initial Stats (Should be empty)
    let resp = client.get(&format!("{}/api/v1/dashboard/stats", app.address))
        .headers(auth_headers.clone())
        .send()
        .await
        .expect("Failed to get stats");
    
    assert_eq!(resp.status(), 200);
    let stats: DashboardStats = resp.json().await.expect("Failed to parse stats");
    assert_eq!(stats.catalogs_count, 0);
    assert_eq!(stats.namespaces_count, 0);
    assert_eq!(stats.tables_count, 0);

    // 3. Create Catalog
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: "stats_test_cat".to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: Some("test_wh".to_string()),
        storage_location: Some("s3://bucket/stats".to_string()),
        federated_config: None,
        properties: HashMap::new(),
    };

    client.post(&format!("{}/api/v1/catalogs", app.address))
        .headers(auth_headers.clone())
        .json(&catalog)
        .send()
        .await
        .expect("Failed to create catalog");

    // 4. Create Namespace
    let ns_payload = serde_json::json!({
        "name": ["db1"],
        "properties": {}
    });
    client.post(&format!("{}/api/v1/catalogs/stats_test_cat/namespaces", app.address))
        .headers(auth_headers.clone())
        .json(&ns_payload)
        .send()
        .await
        .expect("Failed to create namespace");
    
    // 5. Create Asset (Table)
    let asset_payload = serde_json::json!({
        "name": "table1",
        "kind": "IcebergTable",
        "location": "s3://bucket/stats/db1/table1",
        "properties": {}
    });
    client.post(&format!("{}/api/v1/catalogs/stats_test_cat/namespaces/db1/assets", app.address))
        .headers(auth_headers.clone())
        .json(&asset_payload)
        .send()
        .await
        .expect("Failed to create asset");

    // 6. Verify Stats Updated
    let resp = client.get(&format!("{}/api/v1/dashboard/stats", app.address))
        .headers(auth_headers.clone())
        .send()
        .await
        .expect("Failed to get stats");
    
    assert_eq!(resp.status(), 200);
    let stats: DashboardStats = resp.json().await.expect("Failed to parse stats");
    
    println!("Stats: Catalogs={}, Namespaces={}, Tables={}", 
        stats.catalogs_count, stats.namespaces_count, stats.tables_count);

    assert_eq!(stats.catalogs_count, 1);
    assert_eq!(stats.namespaces_count, 1);
    assert_eq!(stats.tables_count, 1);
}
