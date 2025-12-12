use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt; // Correct trait path
use pangolin_api::app;
use pangolin_store::memory::MemoryStore;
use std::sync::Arc;
use serde_json::{json, Value};

#[tokio::test]
async fn test_create_tenant() {
    let store = Arc::new(MemoryStore::new());
    let app = app(store);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": "integration_test_tenant",
                    "properties": {}
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["name"], "integration_test_tenant");
}

#[tokio::test]
async fn test_full_flow() {
    let store = Arc::new(MemoryStore::new());
    let app = app(store);

    // 1. Create Tenant
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/tenants")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "flow_tenant",
            "properties": {}
        }).to_string()))
        .unwrap();
    
    // Clone app for each request because oneshot consumes it
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let tenant_id = body["id"].as_str().unwrap();

    // 2. Create Namespace
    let req = Request::builder()
        .method("POST")
        .uri("/v1/default/namespaces")
        .header("Content-Type", "application/json")
        .header("X-Pangolin-Tenant", tenant_id)
        .body(Body::from(json!({
            "namespace": ["ns_flow"]
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Create Table
    let req = Request::builder()
        .method("POST")
        .uri("/v1/default/namespaces/ns_flow/tables")
        .header("Content-Type", "application/json")
        .header("X-Pangolin-Tenant", tenant_id)
        .body(Body::from(json!({
            "name": "tbl_flow"
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Get Table
    let req = Request::builder()
        .method("GET")
        .uri("/v1/default/namespaces/ns_flow/tables/tbl_flow")
        .header("X-Pangolin-Tenant", tenant_id)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Rename Table
    let req = Request::builder()
        .method("POST")
        .uri("/v1/default/tables/rename")
        .header("Content-Type", "application/json")
        .header("X-Pangolin-Tenant", tenant_id)
        .body(Body::from(json!({
            "source": {"namespace": ["ns_flow"], "name": "tbl_flow"},
            "destination": {"namespace": ["ns_flow"], "name": "tbl_flow_renamed"}
        }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 6. Delete Table
    let req = Request::builder()
        .method("DELETE")
        .uri("/v1/default/namespaces/ns_flow/tables/tbl_flow_renamed")
        .header("X-Pangolin-Tenant", tenant_id)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
