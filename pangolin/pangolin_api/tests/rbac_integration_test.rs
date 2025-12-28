use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{post, get},
    Router,
};
use tower::ServiceExt; // for `oneshot`
use pangolin_core::user::{User, UserRole};
use pangolin_core::model::{Catalog, CatalogType};
use pangolin_store::MemoryStore;
use pangolin_store::CatalogStore;
use std::sync::Arc;
use pangolin_api::user_handlers::login;
use pangolin_api::auth_middleware::hash_password;
use pangolin_api::iceberg::{namespaces, types};
use pangolin_api::tests_common::EnvGuard;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_tenant_user_create_namespace_denied() {
    let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "default_secret_for_dev");
    
    // 1. Setup Store
    let store_impl = MemoryStore::new();
    let store: Arc<dyn CatalogStore + Send + Sync> = Arc::new(store_impl);
    
    // Setup Data
    let tenant_id = Uuid::new_v4();
    let catalog_name = "test_catalog".to_string();
    
    // Create Catalog
    let catalog = Catalog {
        id: Uuid::new_v4(),
        name: catalog_name.clone(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: Default::default(),
    };
    store.create_catalog(tenant_id, catalog).await.unwrap();

    // Create Tenant User
    let password = "user_password";
    let hash = hash_password(password).unwrap();
    let user = User {
        id: Uuid::new_v4(),
        username: "tenant_user".to_string(),
        email: "user@tenant.com".to_string(),
        password_hash: Some(hash),
        oauth_provider: None,
        oauth_subject: None,
        tenant_id: Some(tenant_id), // Tenant ID set
        role: UserRole::TenantUser, // Specific Restricted Role
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
    };
    store.create_user(user.clone()).await.unwrap();

    // DEBUG: Verify user exists and password works
    let stored_user = store.get_user_by_username("tenant_user").await.unwrap().expect("User not found in store");
    assert_eq!(stored_user.username, "tenant_user");
    let stored_hash = stored_user.password_hash.as_ref().expect("No password hash");
    let verify_result = pangolin_api::auth_middleware::verify_password(password, stored_hash).expect("Verification error");
    assert!(verify_result, "Password verification failed internally");
    println!("DEBUG: User verified in store with valid password hash");

    // 2. Setup Router with Auth Middleware and Iceberg Routes
    let app = Router::new()
        // Auth Route for Login
        .route("/api/v1/users/login", post(login))
        // Target Route (Protected)
        .route("/v1/:prefix/namespaces", post(namespaces::create_namespace))
        .layer(axum::middleware::from_fn(pangolin_api::auth_middleware::auth_middleware_wrapper))
        .with_state(store.clone());

    // 3. Login to get Token
    let login_req = pangolin_api::user_handlers::LoginRequest {
        username: "tenant_user".to_string(),
        password: password.to_string(),
        tenant_id: Some(tenant_id),  // Tenant-scoped login
    };
    
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/users/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    if response.status() != StatusCode::OK {
         let status = response.status();
         let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
         println!("Login Failed: Status={}, Body={:?}", status, String::from_utf8_lossy(&body));
         panic!("Login failed");
    }
    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = login_res["token"].as_str().unwrap();

    // 4. Attempt Create Namespace (Should Fail)
    let create_req = types::CreateNamespaceRequest {
        namespace: vec!["db1".to_string()],
        properties: None,
    };

    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/{}/namespaces", catalog_name))
        .header("Authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&create_req).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    
    // Assert 403 Forbidden
    if response.status() != StatusCode::FORBIDDEN {
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("Response: {:?}", String::from_utf8_lossy(&body));
        panic!("Expected 403 Forbidden, got {}", status);
    }
}
