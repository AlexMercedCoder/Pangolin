use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{post, get},
    Router,
    Json,
};
use tower::ServiceExt; // for `oneshot`
use pangolin_core::user::{User, UserRole};
use pangolin_store::MemoryStore;
use pangolin_store::CatalogStore;
use std::sync::Arc;
use pangolin_api::user_handlers::{create_user, login, get_current_user, CreateUserRequest, LoginRequest};
use pangolin_api::auth_middleware::hash_password;
use serial_test::serial;
use pangolin_api::tests_common::EnvGuard;

#[tokio::test]
#[serial]
async fn test_auth_flow() {
    let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "default_secret_for_dev");
    // 1. Setup Store
    let store_impl = MemoryStore::new();
    let store: Arc<dyn CatalogStore + Send + Sync> = Arc::new(store_impl);
    
    // 2. Setup Router
    let app = Router::new()
        .route("/users", post(create_user))
        .route("/login", post(login))
        .route("/me", get(get_current_user))
        .with_state(store.clone());

    // 3. Create Root User manually
    // Actually, let's use the API to create a user if we can, or insert directly into store if needed.
    // Given create_user handler exists but doesn't have auth guard yet, let's use it or insert directly.
    // Better to insert directly to ensure clean slate for login test.
    
    let password = "secure_password";
    let hash = hash_password(password).unwrap();
    let root_user = User::new_root(
        "admin".to_string(),
        "admin@example.com".to_string(),
        hash,
    );
    store.create_user(root_user.clone()).await.unwrap();

    // 4. Test Login Success
    let login_req = LoginRequest {
        username: "admin".to_string(),
        password: password.to_string(),
        tenant_id: None,  // Root login
    };
    
    let req = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    let token = login_res["token"].as_str().unwrap();
    assert!(!token.is_empty());
    assert_eq!(login_res["user"]["username"], "admin");

    // 5. Test Login Failure (Wrong Password)
    let bad_login_req = LoginRequest {
        username: "admin".to_string(),
        password: "wrong_password".to_string(),
        tenant_id: None,  // Root login
    };
    
    let req = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&bad_login_req).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 6. Test Protected Route (Get Current User)
    // Note: To test middleware, we'd need to attach it to the router. 
    // The snippet above didn't attach auth_middleware.
    // Let's create a router WITH middleware for the /me endpoint.
    
    use pangolin_api::auth_middleware::auth_middleware;
    use axum::middleware;
    use axum::Extension;
    use pangolin_core::user::UserSession;

    let protected_app = Router::new()
        .route("/me", get(|Extension(session): Extension<UserSession>| async move {
            // Mock handler that returns the session user
            Json(session)
        }))
        .layer(middleware::from_fn(pangolin_api::auth_middleware::auth_middleware_wrapper))
        .with_state(store.clone());
        
    // Set JWT secret for test
    // Set JWT secret for test
    // Environment variable set at top of function
    // let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "default_secret_for_dev");

    let req = Request::builder()
        .method("GET")
        .uri("/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = protected_app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
