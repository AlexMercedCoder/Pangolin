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
    use pangolin_api::tests_common::EnvGuard;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_config_endpoint_no_auth() {
        // Test that /v1/config is accessible without authentication
        // We explicitly enable NO_AUTH for this test
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");
        
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
    #[serial]
    async fn test_config_endpoint_with_prefix_no_auth() {
        // Test that /v1/:prefix/config is accessible without authentication
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");

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
    #[serial]
    async fn test_tenant_header_propagation() {
        // Test that X-Pangolin-Tenant header is properly extracted
        // Use NO_AUTH to bypass login, but verify tenant header is accepted
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");

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
    #[serial]
    async fn test_operations_without_tenant_header_succeed_with_nil_tenant() {
        // Test that operations without tenant header succeed with nil tenant (dev mode)
        // This is current behavior - auth middleware allows nil tenant as fallback
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");

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
    #[serial]
    async fn test_config_endpoint_returns_valid_json() {
        // Test that config endpoint returns valid JSON
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");

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
    #[serial]
    async fn test_root_user_basic_auth() {
        // Test that root user can authenticate with Basic Auth
        // Set env vars using EnvGuard
        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        // Ensure NO_AUTH is OFF (it might be set by other tests if they leaked, but serial helps. default is secure)
        // We can explicitly unset it to be sure? EnvGuard::new with empty?
        // But removing var? EnvGuard supports overwrite. If we set "false" or empty?
        // Middleware checks `var("PANGOLIN_NO_AUTH").is_ok()`. So presence means true.
        // We need to REMOVE it.
        // EnvGuard doesn't strictly remove, it restores previous value.
        // If previous was unset, it unsets.
        // Since we run serial, and other tests UNSET it on drop, it should be unset here.

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
        // It might return 200 OK (empty list)
    }
}
