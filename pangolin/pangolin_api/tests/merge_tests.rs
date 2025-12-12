use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`
use pangolin_api::app;
use pangolin_core::model::{Asset, AssetType};
use serde_json::{json, Value};
use uuid::Uuid;

use std::sync::Arc;
use pangolin_store::memory::MemoryStore;

#[tokio::test]
async fn test_merge_branch_flow() {
    // 1. Initialize App
    std::env::set_var("PANGOLIN_ROOT_USER", "admin");
    std::env::set_var("PANGOLIN_ROOT_PASSWORD", "password");
    let _ = tracing_subscriber::fmt::try_init();
    let store = Arc::new(MemoryStore::new());
    let app = app(store);

    // 2. Create Tenant
    let tenant_id = Uuid::new_v4();
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

    // 2.5 Create Namespace (and implicitly Catalog? No, need to create Namespace explicitly)
    // Actually, create_table might require namespace to exist.
    let create_ns_req = Request::builder()
        .method("POST")
        .uri("/v1/default/namespaces")
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "namespace": ["default"]
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_ns_req).await.unwrap();
    // Namespace creation might return 200 or 201? Iceberg spec says 200 with CreateNamespaceResponse.
    // Let's check status. If it fails, we know why.
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Create 'main' branch (implicitly done or explicit?)
    // Our system defaults to 'main' existing or being created on first use usually, 
    // but let's create it explicitly to be safe if needed, or just create an asset on 'main'.
    
    // Create Asset on 'main'
    let create_asset_req = Request::builder()
        .method("POST")
        .uri(format!("/v1/default/namespaces/default/tables?branch=main"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
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
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "dev",
            "from_branch": "main",
            "assets": ["default.table1"]
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(create_branch_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 5. Update Asset on 'dev' (Simulate change)
    // We'll just overwrite it with new property
    let update_asset_req = Request::builder()
        .method("POST")
        .uri(format!("/v1/default/namespaces/default/tables?branch=dev"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "table1",
            "location": "s3://bucket/table1",
            "properties": {"v": "2"}
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(update_asset_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Verify 'main' still has v=1
    let get_main_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/default/namespaces/default/tables/table1?branch=main"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(get_main_req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let asset: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(asset["properties"]["v"], "1");

    // 7. Merge 'dev' into 'main'
    let merge_req = Request::builder()
        .method("POST")
        .uri("/api/v1/branches/merge")
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "source_branch": "dev",
            "target_branch": "main"
        }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(merge_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 8. Verify 'main' now has v=2
    let get_main_req_2 = Request::builder()
        .method("GET")
        .uri(format!("/v1/default/namespaces/default/tables/table1?branch=main"))
        .header("X-Pangolin-Tenant", tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(get_main_req_2).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let asset: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(asset["properties"]["v"], "2");
}
