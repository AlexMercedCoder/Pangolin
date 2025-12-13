#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Request, StatusCode};
    use tower::ServiceExt;
    use pangolin_api::app;
    use pangolin_store::CatalogStore;
    use pangolin_store::memory::MemoryStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_no_auth_mode_bypasses_authentication() {
        // Set NO_AUTH environment variable
        std::env::set_var("PANGOLIN_NO_AUTH", "1");

        let store = Arc::new(MemoryStore::new());
        let app = crate::app(store);

        // Request without any authentication headers
        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should succeed with default tenant
        assert_eq!(response.status(), StatusCode::OK);

        // Cleanup
        std::env::remove_var("PANGOLIN_NO_AUTH");
    }

    #[tokio::test]
    async fn test_config_endpoint_no_auth_required() {
        let store = Arc::new(MemoryStore::new());
        let app = crate::app(store);

        let request = Request::builder()
            .method("GET")
            .uri("/v1/config")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_bearer_token_authentication() {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use crate::auth::Claims;
        use chrono::Utc;

        let store = Arc::new(MemoryStore::new());
        let app = app(store);

        // Generate valid token
        let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let claims = Claims {
            sub: "test-user".to_string(),
            tenant_id: Some("00000000-0000-0000-0000-000000000001".to_string()),
            roles: vec!["User".to_string()],
            exp: (Utc::now().timestamp() + 3600) as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_bearer_token_rejected() {
        let store = Arc::new(MemoryStore::new());
        let app = app(store);

        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .header("Authorization", "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_token_rejected() {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use crate::auth::Claims;
        use chrono::Utc;

        let store = Arc::new(MemoryStore::new());
        let app = app(store);

        // Generate expired token
        let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let claims = Claims {
            sub: "test-user".to_string(),
            tenant_id: Some("00000000-0000-0000-0000-000000000001".to_string()),
            roles: vec!["User".to_string()],
            exp: (Utc::now().timestamp() - 3600) as usize, // Expired 1 hour ago
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_no_auth_uses_default_tenant() {
        std::env::set_var("PANGOLIN_NO_AUTH", "1");

        let store = Arc::new(MemoryStore::new());
        
        // Create default tenant
        let default_tenant = pangolin_core::model::Tenant {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(default_tenant).await.unwrap();

        let app = app(store);

        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        std::env::remove_var("PANGOLIN_NO_AUTH");
    }

    #[tokio::test]
    async fn test_basic_auth_for_root_user() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        std::env::set_var("PANGOLIN_ROOT_USER", "admin");
        std::env::set_var("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        let app = app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        std::env::remove_var("PANGOLIN_ROOT_USER");
        std::env::remove_var("PANGOLIN_ROOT_PASSWORD");
    }

    #[tokio::test]
    async fn test_wrong_basic_auth_rejected() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        std::env::set_var("PANGOLIN_ROOT_USER", "admin");
        std::env::set_var("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        let app = app(store);

        let credentials = STANDARD.encode("admin:wrongpassword");

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        std::env::remove_var("PANGOLIN_ROOT_USER");
        std::env::remove_var("PANGOLIN_ROOT_PASSWORD");
    }
}
