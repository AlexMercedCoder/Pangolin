#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;
    use pangolin_store::memory::MemoryStore;
    use pangolin_api::app;
    use std::sync::Arc;
    use base64::Engine;

    #[tokio::test]
    async fn test_config_endpoint_no_auth() {
        // Test that /v1/config is accessible without authentication
        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_config_endpoint_with_prefix_no_auth() {
        // Test that /v1/:prefix/config is accessible without authentication
        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/main_warehouse/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_tenant_header_propagation() {
        // Test that X-Pangolin-Tenant header is properly extracted
        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        
        // Create a test tenant first
        let tenant_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        store.create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: "test_tenant".to_string(),
            properties: std::collections::HashMap::new(),
        }).await.unwrap();

        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/main_warehouse/namespaces")
                    .header("X-Pangolin-Tenant", "00000000-0000-0000-0000-000000000001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 200 (or 404 if warehouse doesn't exist, but not 401)
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_operations_without_tenant_header_succeed_with_nil_tenant() {
        // Test that operations without tenant header succeed with nil tenant (dev mode)
        // This is current behavior - auth middleware allows nil tenant as fallback
        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/main_warehouse/namespaces")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should succeed (200 or 404 for missing warehouse, but not 401)
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_config_endpoint_returns_valid_json() {
        // Test that config endpoint returns valid JSON
        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        // Should have defaults and overrides keys
        assert!(json.get("defaults").is_some());
        assert!(json.get("overrides").is_some());
    }

    #[tokio::test]
    async fn test_root_user_basic_auth() {
        // Test that root user can authenticate with Basic Auth
        std::env::set_var("PANGOLIN_ROOT_USER", "admin");
        std::env::set_var("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        let app = app(store);

        let credentials = base64::engine::general_purpose::STANDARD.encode("admin:password");
        let auth_header = format!("Basic {}", credentials);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tenants")
                    .header(header::AUTHORIZATION, auth_header)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should not return 401
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
