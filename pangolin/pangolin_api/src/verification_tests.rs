    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt; // for `oneshot`
    use urlencoding::encode;
    use pangolin_store::memory::MemoryStore;
    use std::sync::Arc;
    use serde_json::json;
    use crate::tests_common::EnvGuard;
    use uuid::Uuid;

    // Helper to generic test app
    fn app() -> Router {
        let store = Arc::new(MemoryStore::new());
        crate::app(store)
    }

    // Helper for Authenticated Requests
    async fn get_auth_token(app: &Router, username: &str, password: &str) -> String {
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/users/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "username": username,
                        "password": password
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        json["token"].as_str().unwrap().to_string()
    }

    // 1. Authentication Tests
    #[tokio::test]
    async fn test_auth_root_login() {
        let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin_test");
        let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "pass_test");
        
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/users/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "username": "admin_test",
                        "password": "pass_test"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_failure() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/users/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "username": "admin",
                        "password": "wrongpassword"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // 2. User Restriction Tests
    #[tokio::test]
    async fn test_root_cannot_create_warehouse() {
        let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let app = app();
        let token = get_auth_token(&app, "admin", "password").await;

        // Try to create warehouse as Root
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/warehouses")
                    .header("Authorization", format!("Bearer {}", token))
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "name": "root-warehouse",
                        "use_sts": false,
                         "storage_config": {
                            "type": "s3",
                            "bucket": "warehouse"
                        },
                        "vending_strategy": null
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
            
        // Expect Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    // 3. Full Flow: Create Tenant Admin -> Create Warehouse -> Create Catalog -> PyIceberg Config
    #[tokio::test]
    async fn test_verified_flow_regression() {
        let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let app = app();
        let root_token = get_auth_token(&app, "admin", "password").await;

        // A. Create Tenant (+ Tenant Admin) automatically? No, create User with Tenant.
        // First, create a Tenant
        let response = app.clone().oneshot(
             Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Authorization", format!("Bearer {}", root_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": "test-tenant",
                    "organization": "TestOrg"
                }).to_string())).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tenant_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let tenant_id = tenant_json["id"].as_str().unwrap();

        // B. Create Tenant Admin User
        let response = app.clone().oneshot(
             Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("Authorization", format!("Bearer {}", root_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "username": "tenant_admin",
                    "email": "ta@test.com",
                    "password": "Password123",
                    "role": "TenantAdmin",
                    "tenant_id": tenant_id
                }).to_string())).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // C. Login as Tenant Admin
        let ta_token = get_auth_token(&app, "tenant_admin", "Password123").await;

        // D. Create Warehouse (Success)
        let response = app.clone().oneshot(
             Request::builder()
                .method("POST")
                .uri("/api/v1/warehouses")
                .header("Authorization", format!("Bearer {}", ta_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": "success-warehouse",
                    "use_sts": false,
                    "storage_config": {
                        "type": "s3",
                        "bucket": "warehouse"
                    }
                }).to_string())).unwrap()
        ).await.unwrap();
        // Should be Created
        assert_eq!(response.status(), StatusCode::CREATED);

        // E. Create Catalog
        let response = app.clone().oneshot(
             Request::builder()
                .method("POST")
                .uri("/api/v1/catalogs")
                .header("Authorization", format!("Bearer {}", ta_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": "test-catalog",
                    "warehouse_name": "success-warehouse",
                    "type": "pangolin"
                }).to_string())).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // F. PyIceberg Config Check (v1/{catalog}/config)
        // ...
        
        let response = app.clone().oneshot(
             Request::builder()
                .method("GET")
                .uri("/v1/test-catalog/v1/config")
                .header("Authorization", format!("Bearer {}", ta_token))
                .body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // ...
    }

    // 4. Namespace Creation (Iceberg)
    #[tokio::test]
    async fn test_iceberg_namespace_creation() {
         let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let app = app();
        
        let root_token = get_auth_token(&app, "admin", "password").await;
        
        // Tenant
        let t_res = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tenants").header("Authorization", format!("Bearer {}", root_token)).header("Content-Type", "application/json").body(Body::from(json!({"name":"t1", "organization":"o1"}).to_string())).unwrap()).await.unwrap();
        assert_eq!(t_res.status(), StatusCode::CREATED);
        let t_id = serde_json::from_slice::<serde_json::Value>(&to_bytes(t_res.into_body(), usize::MAX).await.unwrap()).unwrap()["id"].as_str().unwrap().to_string();

        // User
         let u_res = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/users").header("Authorization", format!("Bearer {}", root_token)).header("Content-Type", "application/json").body(Body::from(json!({"username":"ta","email":"t@a.com","password":"pw","role":"TenantAdmin","tenant_id":t_id}).to_string())).unwrap()).await.unwrap();
         assert_eq!(u_res.status(), StatusCode::CREATED);
        let ta_token = get_auth_token(&app, "ta", "pw").await;

        // Warehouse
        let w_res = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/warehouses").header("Authorization", format!("Bearer {}", ta_token)).header("Content-Type", "application/json").body(Body::from(json!({"name":"wh","use_sts":false,"storage_config":{"type":"s3","bucket":"bh"}}).to_string())).unwrap()).await.unwrap();
        assert_eq!(w_res.status(), StatusCode::CREATED);

        // Catalog
        let c_res = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/catalogs").header("Authorization", format!("Bearer {}", ta_token)).header("Content-Type", "application/json").body(Body::from(json!({"name":"cat","warehouse_name":"wh","type":"pangolin"}).to_string())).unwrap()).await.unwrap();
        assert_eq!(c_res.status(), StatusCode::CREATED);

        // Test Endpoint: Create Namespace
        let response = app.clone().oneshot(
             Request::builder()
                .method("POST")
                .uri("/v1/cat/v1/namespaces")
                .header("Authorization", format!("Bearer {}", ta_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "namespace": ["ns1"]
                }).to_string())).unwrap()
        ).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }

