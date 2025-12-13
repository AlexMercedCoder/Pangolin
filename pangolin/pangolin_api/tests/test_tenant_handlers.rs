#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use hyper; // For hyper::body::to_bytes
    use tower::ServiceExt; // for oneshot
    use pangolin_api::app;
    use pangolin_store::{memory::MemoryStore, CatalogStore};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_tenant_creation_blocked_in_no_auth_mode() {
        std::env::set_var("PANGOLIN_NO_AUTH", "1");

        let store = Arc::new(MemoryStore::new());
        let app = crate::app(store);

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
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "Cannot create additional tenants in NO_AUTH mode");
        assert!(json["message"].as_str().unwrap().contains("evaluation and testing"));
        assert!(json["hint"].as_str().unwrap().contains("PANGOLIN_NO_AUTH"));

        std::env::remove_var("PANGOLIN_NO_AUTH");
    }

    #[tokio::test]
    async fn test_tenant_creation_allowed_with_auth() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        // Ensure NO_AUTH is not set
        std::env::remove_var("PANGOLIN_NO_AUTH");
        std::env::set_var("PANGOLIN_ROOT_USER", "admin");
        std::env::set_var("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        let app = crate::app(store);

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

        std::env::remove_var("PANGOLIN_ROOT_USER");
        std::env::remove_var("PANGOLIN_ROOT_PASSWORD");
    }

    #[tokio::test]
    async fn test_list_tenants_works_in_no_auth_mode() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        std::env::set_var("PANGOLIN_NO_AUTH", "1");
        std::env::set_var("PANGOLIN_ROOT_USER", "admin");
        std::env::set_var("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        
        // Create default tenant
        let default_tenant = pangolin_core::model::Tenant {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(default_tenant).await.unwrap();

        let app = crate::app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        std::env::remove_var("PANGOLIN_NO_AUTH");
        std::env::remove_var("PANGOLIN_ROOT_USER");
        std::env::remove_var("PANGOLIN_ROOT_PASSWORD");
    }
}
