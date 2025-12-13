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
use crate::user_handlers::{create_user, login, get_current_user, CreateUserRequest, LoginRequest};
use crate::auth_middleware::hash_password;

#[tokio::test]
async fn test_auth_flow() {
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
    
    use crate::auth_middleware::auth_middleware;
    use axum::middleware;

    let protected_app = Router::new()
        .route("/me", get(|state: axum::extract::State<Arc<dyn CatalogStore + Send + Sync>>, ext: axum::Extension<pangolin_core::user::UserSession>| async move {
            // Mock handler that returns the session user
            // In real app, get_current_user would do this or look up DB
            Json(ext.0) 
        }))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(store.clone());
        
    // Set JWT secret for test
    std::env::set_var("JWT_SECRET", "default_secret_for_dev");

    let req = Request::builder()
        .method("GET")
        .uri("/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = protected_app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
