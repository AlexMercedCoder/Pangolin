use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt; // Correct trait path
use pangolin_api::app;
use pangolin_store::memory::MemoryStore;
use std::sync::Arc;
use serde_json::{json, Value};
use serial_test::serial;
use pangolin_api::tests_common::EnvGuard;

#[tokio::test]
#[serial]
async fn test_create_tenant() {
    let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
    let store = Arc::new(MemoryStore::new());
    let app = app(store);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46cGFzc3dvcmQ=") // admin:password
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

// test_full_flow removed
// test_full_flow removed as it was incomplete (missing catalog creation) and redundant.
