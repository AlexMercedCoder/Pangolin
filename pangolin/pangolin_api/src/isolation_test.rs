
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
    use crate::user_handlers;
    use crate::warehouse_handlers::{list_warehouses, create_warehouse, CreateWarehouseRequest, WarehouseResponse};

    // Helper to get app router
    fn setup_app() -> Router {
        let store = Arc::new(MemoryStore::new());
        // We cast to AppState type equivalent
        let state: Arc<dyn CatalogStore + Send + Sync> = store;
        
        Router::new()
            .route("/api/v1/users/login", post(user_handlers::login))
            .route("/api/v1/warehouses", get(list_warehouses).post(create_warehouse))
            // Use the wrapper which is stateless and easier to test if it works
            .layer(middleware::from_fn(crate::auth_middleware::auth_middleware_wrapper))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_warehouse_isolation() {
        std::env::set_var("PANGOLIN_JWT_SECRET", "test_secret_for_isolation");
        
        // 0. Setup store with admin user
        let store = Arc::new(MemoryStore::new());
        let hash = crate::auth_middleware::hash_password("password").unwrap();
        let admin_user = pangolin_core::user::User::new_root(
            "admin".to_string(),
            "admin@example.com".to_string(),
            hash,
        );
        store.create_user(admin_user).await.unwrap();
        
        // Define state
        let state: Arc<dyn CatalogStore + Send + Sync> = store;

        let app = Router::new()
            .route("/api/v1/users/login", post(user_handlers::login))
            .route("/api/v1/warehouses", get(list_warehouses).post(create_warehouse))
            .layer(middleware::from_fn(crate::auth_middleware::auth_middleware_wrapper))
            .with_state(state);

        // 1. Login to get token
        let login_req = Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username":"admin","password":"password"}"#))
            .unwrap();
            
        let login_res: Response = app.clone().oneshot(login_req).await.unwrap();
        let body_bytes = axum::body::to_bytes(login_res.into_body(), usize::MAX).await.unwrap();
        let login_json: Value = serde_json::from_slice(&body_bytes).unwrap();
        let token = login_json["token"].as_str().unwrap();

        // 2. Create Tenant A Context
        let tenant_a_id = Uuid::new_v4();

        // Create Warehouse in Tenant A
        let create_res: Response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/warehouses")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .header("X-Pangolin-Tenant", tenant_a_id.to_string())
                .body(Body::from(serde_json::to_string(&CreateWarehouseRequest {
                    name: "warehouse-a".to_string(),
                    use_sts: None,
                    storage_config: None,
                }).unwrap()))
                .unwrap()
        ).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        // 3. Create Tenant B Context
        let tenant_b_id = Uuid::new_v4();

        // 4. List in Tenant A (Should see it)
        let list_a_res: Response = app.clone().oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/warehouses")
                .header("Authorization", format!("Bearer {}", token))
                .header("X-Pangolin-Tenant", tenant_a_id.to_string())
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();
        
        let body_a = axum::body::to_bytes(list_a_res.into_body(), usize::MAX).await.unwrap();
        let list_a: Vec<WarehouseResponse> = serde_json::from_slice(&body_a).unwrap();
        assert_eq!(list_a.len(), 1);
        assert_eq!(list_a[0].name, "warehouse-a");

        // 5. List in Tenant B (Should NOT see it)
        let list_b_res: Response = app.clone().oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/warehouses")
                .header("Authorization", format!("Bearer {}", token))
                .header("X-Pangolin-Tenant", tenant_b_id.to_string())
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        let body_b = axum::body::to_bytes(list_b_res.into_body(), usize::MAX).await.unwrap();
        let list_b: Vec<WarehouseResponse> = serde_json::from_slice(&body_b).unwrap();
        assert_eq!(list_b.len(), 0, "Tenant B should not see Tenant A's warehouse: {:?}", list_b.iter().map(|w| &w.name).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn test_catalog_isolation() {
        std::env::set_var("PANGOLIN_JWT_SECRET", "test_secret_for_isolation");
        
        let store = Arc::new(MemoryStore::new());
        let hash = crate::auth_middleware::hash_password("password").unwrap();
        let admin_user = pangolin_core::user::User::new_root(
            "admin".to_string(),
            "admin@example.com".to_string(),
            hash,
        );
        store.create_user(admin_user).await.unwrap();
        
        let state: Arc<dyn CatalogStore + Send + Sync> = store;

        let app = Router::new()
            .route("/api/v1/users/login", post(user_handlers::login))
            .route("/api/v1/catalogs", get(crate::pangolin_handlers::list_catalogs).post(crate::pangolin_handlers::create_catalog))
            .layer(middleware::from_fn(crate::auth_middleware::auth_middleware_wrapper))
            .with_state(state);

        // Login
        let login_req = Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username":"admin","password":"password"}"#))
            .unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        let body_bytes = axum::body::to_bytes(login_res.into_body(), usize::MAX).await.unwrap();
        let token = serde_json::from_slice::<Value>(&body_bytes).unwrap()["token"].as_str().unwrap().to_string();

        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();

        // Create Catalog in Tenant A
        let create_res = app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/catalogs")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Pangolin-Tenant", tenant_a.to_string())
            .body(Body::from(r#"{"name":"cat-a","warehouse_name":null,"storage_location":"s3://bucket"}"#))
            .unwrap()
        ).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        // List A -> Should see 1
        let list_a = app.clone().oneshot(Request::builder()
            .method("GET")
            .uri("/api/v1/catalogs")
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Pangolin-Tenant", tenant_a.to_string())
            .body(Body::empty())
            .unwrap()
        ).await.unwrap();
        let body_a = axum::body::to_bytes(list_a.into_body(), usize::MAX).await.unwrap();
        let list_a_json: Value = serde_json::from_slice(&body_a).unwrap();
        assert_eq!(list_a_json.as_array().unwrap().len(), 1);

        // List B -> Should see 0
        let list_b = app.clone().oneshot(Request::builder()
            .method("GET")
            .uri("/api/v1/catalogs")
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Pangolin-Tenant", tenant_b.to_string())
            .body(Body::empty())
            .unwrap()
        ).await.unwrap();
        let body_b = axum::body::to_bytes(list_b.into_body(), usize::MAX).await.unwrap();
        let list_b_json: Value = serde_json::from_slice(&body_b).unwrap();
        assert_eq!(list_b_json.as_array().unwrap().len(), 0);
    }
}
