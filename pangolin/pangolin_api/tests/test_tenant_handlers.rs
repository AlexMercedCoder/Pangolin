#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use axum::body::to_bytes;
    use tower::ServiceExt;
    use pangolin_store::{memory::MemoryStore, CatalogStore};
    use std::sync::Arc;
    use serial_test::serial;
    use pangolin_api::tests_common::EnvGuard;

    #[tokio::test]
    #[serial]
    async fn test_tenant_creation_blocked_in_no_auth_mode() {
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "1");

        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tenants")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name": "test-tenant"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should be forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Check error message
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "Cannot create additional tenants in NO_AUTH mode");
        assert!(json["message"].as_str().unwrap().contains("evaluation and testing"));
        assert!(json["hint"].as_str().unwrap().contains("PANGOLIN_NO_AUTH"));
    }

    #[tokio::test]
    #[serial]
    async fn test_tenant_creation_allowed_with_auth() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tenants")
            .header("Authorization", format!("Basic {}", credentials))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name": "test-tenant"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should succeed
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[serial]
    async fn test_list_tenants_works_in_no_auth_mode() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let _no_auth_guard = EnvGuard::new("PANGOLIN_NO_AUTH", "1");
        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        
        // Create default tenant
        let default_tenant = pangolin_core::model::Tenant {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(default_tenant).await.unwrap();

        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
