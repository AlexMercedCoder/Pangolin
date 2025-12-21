use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{post, get},
    Router,
    middleware,
};
use tower::ServiceExt; 
use std::sync::Arc;
use pangolin_store::MemoryStore;
use pangolin_store::CatalogStore;
// Handlers
// Handlers
use pangolin_api::user_handlers::{login, LoginRequest, get_app_config};
use pangolin_api::warehouse_handlers::create_warehouse;
use pangolin_api::pangolin_handlers::create_catalog;
use pangolin_api::tests_common::EnvGuard;

use serial_test::serial;

// Helper to setup app with middleware
fn setup_app(store: Arc<dyn CatalogStore + Send + Sync>) -> Router {
    Router::new()
        .route("/api/v1/users/login", post(login))
        .route("/api/v1/app-config", get(get_app_config))
        .route("/api/v1/warehouses", post(create_warehouse))
        .route("/api/v1/catalogs", post(create_catalog))
        .layer(middleware::from_fn(pangolin_api::auth_middleware::auth_middleware_wrapper))
        .with_state(store)
}

#[tokio::test]
#[serial]
async fn test_root_login_env_vars() {
    let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "superadmin");
    let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "supersecret");
    let _guard_jwt = EnvGuard::new("PANGOLIN_JWT_SECRET", "test_secret");

    let store = Arc::new(MemoryStore::new());
    let app = setup_app(store);

    let login_req = LoginRequest {
        username: "superadmin".to_string(),
        password: "supersecret".to_string(),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/users/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Login should succeed");
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(login_res["user"]["role"], "root");
}

#[tokio::test]
#[serial]
async fn test_root_cannot_create_warehouse() {
    let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
    let _guard_jwt = EnvGuard::new("PANGOLIN_JWT_SECRET", "test_secret");

    let store = Arc::new(MemoryStore::new());
    let app = setup_app(store);

    // 1. Login to get token
    let login_req = LoginRequest { username: "admin".to_string(), password: "password".to_string() };
    let req = Request::builder().method("POST").uri("/api/v1/users/login").header("content-type", "application/json").body(Body::from(serde_json::to_string(&login_req).unwrap())).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Login should succeed");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = login_res["token"].as_str().unwrap();

    // 2. Try create warehouse
    let wh_req_json = serde_json::json!({
        "name": "root_wh",
        "storage_config": {
            "type": "memory"
        }
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/warehouses")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(wh_req_json.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN); // Expect 403
}

#[tokio::test]
#[serial]
async fn test_root_cannot_create_catalog() {
    let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
    let _guard_jwt = EnvGuard::new("PANGOLIN_JWT_SECRET", "test_secret");

    let store = Arc::new(MemoryStore::new());
    let app = setup_app(store);

    // 1. Login to get token
    let login_req = LoginRequest { username: "admin".to_string(), password: "password".to_string() };
    let req = Request::builder().method("POST").uri("/api/v1/users/login").header("content-type", "application/json").body(Body::from(serde_json::to_string(&login_req).unwrap())).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Login should succeed");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = login_res["token"].as_str().unwrap();

    // 2. Try create catalog
    let cat_req_json = serde_json::json!({
        "name": "root_cat",
        "warehouse_name": "some_wh"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/catalogs")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(cat_req_json.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn test_no_auth_mode_access() {
    // Enable No-Auth
    let _guard_no_auth = EnvGuard::new("PANGOLIN_NO_AUTH", "true");
    
    let store = Arc::new(MemoryStore::new());
    let app = setup_app(store);

    // 1. Check App Config
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/app-config")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let config: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    // In No-Auth mode, auth_enabled is false ( !no_auth )
    assert_eq!(config["auth_enabled"], false);

    // 2. Try a protected endpoint (create warehouse) WITHOUT token
    // In No-Auth mode, handler sees "TenantAdmin" (changed from Root). TenantAdmin CAN create warehouse -> 201 Created.
    
    let wh_req_json = serde_json::json!({
        "name": "no_auth_wh",
        "storage_config": {
            "type": "memory"
        }
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/warehouses")
        .header("content-type", "application/json")
        // NO AUTHORIZATION HEADER
        .body(Body::from(wh_req_json.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
