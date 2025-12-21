
#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
        Router,
        middleware,
        response::Response,
        extract::State,
    };
    use tower::ServiceExt; // for `oneshot`
    use serde_json::Value;
    use std::sync::Arc;
    use pangolin_store::{MemoryStore, CatalogStore};
    use uuid::Uuid;
    use pangolin_api::user_handlers;
    use pangolin_api::warehouse_handlers::{list_warehouses, create_warehouse, CreateWarehouseRequest, WarehouseResponse};

    // Helper to get app router
    fn setup_app() -> Router {
        let store = Arc::new(MemoryStore::new());
        // We cast to AppState type equivalent
        let state: Arc<dyn CatalogStore + Send + Sync> = store;
        
        Router::new()
            .route("/api/v1/users/login", post(user_handlers::login))
            .route("/api/v1/warehouses", get(list_warehouses).post(create_warehouse))
            // Use the wrapper which is stateless and easier to test if it works
            .layer(middleware::from_fn(pangolin_api::auth_middleware::auth_middleware_wrapper))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_warehouse_isolation() {
        std::env::set_var("PANGOLIN_JWT_SECRET", "test_secret_for_isolation");
        
        let store = Arc::new(MemoryStore::new());
        
        // 1. Setup Tenant A and User A
        let tenant_a_id = Uuid::new_v4();
        let tenant_a = pangolin_core::model::Tenant {
            id: tenant_a_id,
            name: "tenant-a".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant_a).await.unwrap();

        let hash_a = pangolin_api::auth_middleware::hash_password("password_a").unwrap();
        let user_a = pangolin_core::user::User {
            id: Uuid::new_v4(),
            username: "user_a".to_string(),
            email: "user_a@example.com".to_string(),
            password_hash: Some(hash_a),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: Some(tenant_a_id),
            role: pangolin_core::user::UserRole::TenantAdmin,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        store.create_user(user_a.clone()).await.unwrap();

        // 2. Setup Tenant B and User B
        let tenant_b_id = Uuid::new_v4();
        let tenant_b = pangolin_core::model::Tenant {
            id: tenant_b_id,
            name: "tenant-b".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant_b).await.unwrap();

        let hash_b = pangolin_api::auth_middleware::hash_password("password_b").unwrap();
        let user_b = pangolin_core::user::User {
            id: Uuid::new_v4(),
            username: "user_b".to_string(),
            email: "user_b@example.com".to_string(),
            password_hash: Some(hash_b),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: Some(tenant_b_id),
            role: pangolin_core::user::UserRole::TenantAdmin,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        store.create_user(user_b.clone()).await.unwrap();

        let state: Arc<dyn CatalogStore + Send + Sync> = store;

        let app = Router::new()
            .route("/api/v1/users/login", post(user_handlers::login))
            .route("/api/v1/warehouses", get(list_warehouses).post(create_warehouse))
            .layer(middleware::from_fn(pangolin_api::auth_middleware::auth_middleware_wrapper))
            .with_state(state);

        // 3. Login as User A
        let login_a_req = Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username":"user_a","password":"password_a"}"#))
            .unwrap();
        let login_a_res: Response = app.clone().oneshot(login_a_req).await.unwrap();
        let body_bytes_a = axum::body::to_bytes(login_a_res.into_body(), usize::MAX).await.unwrap();
        let login_json_a: Value = serde_json::from_slice(&body_bytes_a).unwrap();
        let token_a = login_json_a["token"].as_str().unwrap();

        // 4. Create Warehouse as User A
        let create_res: Response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/warehouses")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token_a))
                .body(Body::from(serde_json::to_string(&CreateWarehouseRequest {
                    name: "warehouse-a".to_string(),
                    use_sts: None,
                    storage_config: None,
                    vending_strategy: None,
                }).unwrap()))
                .unwrap()
        ).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        // 5. Login as User B
        let login_b_req = Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username":"user_b","password":"password_b"}"#))
            .unwrap();
        let login_b_res: Response = app.clone().oneshot(login_b_req).await.unwrap();
        let body_bytes_b = axum::body::to_bytes(login_b_res.into_body(), usize::MAX).await.unwrap();
        let login_json_b: Value = serde_json::from_slice(&body_bytes_b).unwrap();
        let token_b = login_json_b["token"].as_str().unwrap();

        // 6. List Warehouses as User B (Should NOT see warehouse-a)
        let list_b_res: Response = app.clone().oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/warehouses")
                .header("Authorization", format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        let body_b = axum::body::to_bytes(list_b_res.into_body(), usize::MAX).await.unwrap();
        let list_b: Vec<WarehouseResponse> = serde_json::from_slice(&body_b).unwrap();
        assert_eq!(list_b.len(), 0, "User B should not see Tenant A's warehouse");
    }

    #[tokio::test]
    async fn test_catalog_isolation() {
        std::env::set_var("PANGOLIN_JWT_SECRET", "test_secret_for_isolation");
        
        let store = Arc::new(MemoryStore::new());
        
        // 1. Setup Tenant A and User A
        let tenant_a_id = Uuid::new_v4();
        let tenant_a = pangolin_core::model::Tenant {
            id: tenant_a_id,
            name: "tenant-a".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant_a).await.unwrap();

        let hash_a = pangolin_api::auth_middleware::hash_password("password_a").unwrap();
        let user_a = pangolin_core::user::User {
            id: Uuid::new_v4(),
            username: "user_a".to_string(),
            email: "user_a@example.com".to_string(),
            password_hash: Some(hash_a),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: Some(tenant_a_id),
            role: pangolin_core::user::UserRole::TenantAdmin,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        store.create_user(user_a.clone()).await.unwrap();

        // 2. Setup Tenant B and User B
        let tenant_b_id = Uuid::new_v4();
        let tenant_b = pangolin_core::model::Tenant {
            id: tenant_b_id,
            name: "tenant-b".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant_b).await.unwrap();

        let hash_b = pangolin_api::auth_middleware::hash_password("password_b").unwrap();
        let user_b = pangolin_core::user::User {
            id: Uuid::new_v4(),
            username: "user_b".to_string(),
            email: "user_b@example.com".to_string(),
            password_hash: Some(hash_b),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: Some(tenant_b_id),
            role: pangolin_core::user::UserRole::TenantAdmin,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        store.create_user(user_b.clone()).await.unwrap();
        
        let state: Arc<dyn CatalogStore + Send + Sync> = store;

        let app = Router::new()
            .route("/api/v1/users/login", post(user_handlers::login))
            .route("/api/v1/catalogs", get(pangolin_api::pangolin_handlers::list_catalogs).post(pangolin_api::pangolin_handlers::create_catalog))
            .layer(middleware::from_fn(pangolin_api::auth_middleware::auth_middleware_wrapper))
            .with_state(state);

        // 3. Login as User A
        let login_a_req = Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username":"user_a","password":"password_a"}"#))
            .unwrap();
        let login_a_res = app.clone().oneshot(login_a_req).await.unwrap();
        let body_bytes_a = axum::body::to_bytes(login_a_res.into_body(), usize::MAX).await.unwrap();
        let token_a = serde_json::from_slice::<Value>(&body_bytes_a).unwrap()["token"].as_str().unwrap().to_string();

        // 4. Create Catalog as User A
        let create_res = app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/catalogs")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token_a))
            .body(Body::from(r#"{"name":"cat-a","warehouse_name":null,"storage_location":"s3://bucket"}"#))
            .unwrap()
        ).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        // 5. Login as User B
        let login_b_req = Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username":"user_b","password":"password_b"}"#))
            .unwrap();
        let login_b_res = app.clone().oneshot(login_b_req).await.unwrap();
        let body_bytes_b = axum::body::to_bytes(login_b_res.into_body(), usize::MAX).await.unwrap();
        let token_b = serde_json::from_slice::<Value>(&body_bytes_b).unwrap()["token"].as_str().unwrap().to_string();

        // 6. List as User A -> Should see 1
        let list_a = app.clone().oneshot(Request::builder()
            .method("GET")
            .uri("/api/v1/catalogs")
            .header("Authorization", format!("Bearer {}", token_a))
            .body(Body::empty())
            .unwrap()
        ).await.unwrap();
        let body_a = axum::body::to_bytes(list_a.into_body(), usize::MAX).await.unwrap();
        let list_a_json: Value = serde_json::from_slice(&body_a).unwrap();
        assert_eq!(list_a_json.as_array().unwrap().len(), 1);

        // 7. List as User B -> Should see 0
        let list_b = app.clone().oneshot(Request::builder()
            .method("GET")
            .uri("/api/v1/catalogs")
            .header("Authorization", format!("Bearer {}", token_b))
            .body(Body::empty())
            .unwrap()
        ).await.unwrap();
        let body_b = axum::body::to_bytes(list_b.into_body(), usize::MAX).await.unwrap();
        let list_b_json: Value = serde_json::from_slice(&body_b).unwrap();
        assert_eq!(list_b_json.as_array().unwrap().len(), 0);
    }
}
