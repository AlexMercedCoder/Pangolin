use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`
use pangolin_api::app;
use pangolin_core::model::{Catalog, Tenant, Asset, AssetType};
use std::collections::HashMap;
use serde_json::{json, Value};
use uuid::Uuid;
use std::sync::Arc;
use pangolin_store::memory::MemoryStore;
use serial_test::serial;
use pangolin_api::tests_common::EnvGuard;
use pangolin_store::CatalogStore;

#[tokio::test]
#[serial]
async fn test_merge_branch_flow() {
    // 1. Initialize App
    let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
    let _ = tracing_subscriber::fmt::try_init();
    let store = Arc::new(MemoryStore::new());
    let app = app(store.clone());

    // 2. Create Tenant (Using NIL UUID to avoid potential mismatch issues for now, though root should work with any)
    let tenant_id = Uuid::nil();
    let create_tenant_req = Request::builder()
        .method("POST")
        .uri("/api/v1/tenants")
        .header("Content-Type", "application/json")
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=") // admin:password
        .body(Body::from(json!({
            "name": "MergeTestTenant",
            "id": tenant_id.to_string()
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_tenant_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 2.1 Create 'default' Catalog directly in store
    store.create_catalog(tenant_id, Catalog {
            catalog_type: pangolin_core::model::CatalogType::Local,
        id: Uuid::new_v4(),
        name: "default".to_string(),
        warehouse_name: None,
        storage_location: None,
        properties: HashMap::new(),
            federated_config: None,
    }).await.unwrap();

    // 2.5 Create Namespace
    let create_ns_req = Request::builder()
        .method("POST")
        .uri("/v1/default/namespaces")
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "namespace": ["default"]
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_ns_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Create Asset on 'main'
    let create_asset_req = Request::builder()
        .method("POST")
        .uri(format!("/v1/default/namespaces/default/tables?branch=main"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "table1",
            "location": "s3://bucket/table1",
            "properties": {"v": "1"}
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_asset_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Create 'dev' branch from 'main'
    let create_branch_req = Request::builder()
        .method("POST")
        .uri("/api/v1/branches")
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "dev",
            "from_branch": "main",
            "assets": ["default.table1"]
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_branch_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 5. Create NEW Asset on 'dev' (Simulate feature work)
    let create_asset_2_req = Request::builder()
        .method("POST")
        .uri(format!("/v1/default/namespaces/default/tables?branch=dev"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "table2",
            "location": "s3://bucket/table2",
            "properties": {"v": "2"}
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_asset_2_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Verify 'main' does NOT have table2 yet
    let get_main_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/default/namespaces/default/tables/table2?branch=main"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(get_main_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // 7. Merge 'dev' into 'main'
    let merge_req = Request::builder()
        .method("POST")
        .uri("/api/v1/branches/merge")
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "source_branch": "dev",
            "target_branch": "main"
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(merge_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 8. Verify 'main' now has table2
    let get_main_req_2 = Request::builder()
        .method("GET")
        .uri(format!("/v1/default/namespaces/default/tables/table2?branch=main"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(get_main_req_2).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
