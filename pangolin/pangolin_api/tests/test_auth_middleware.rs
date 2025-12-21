#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{Request, StatusCode},
    };
    use axum::body::Body;
    use tower::ServiceExt;
    use pangolin_api::app;
    use pangolin_store::CatalogStore;
    use pangolin_api::auth::Claims;
    use pangolin_core::user::UserRole; // Use Core UserRole
    use pangolin_store::memory::MemoryStore;
    use std::sync::Arc;
    use serial_test::serial;
    use pangolin_api::tests_common::EnvGuard;

    #[tokio::test]
    #[serial]
    async fn test_no_auth_mode_bypasses_authentication() {
        // Set NO_AUTH environment variable
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");

        let store = Arc::new(MemoryStore::new());
        // Create default tenant and analytics catalog
        let default_tenant_id = uuid::Uuid::nil();
        store.create_tenant(pangolin_core::model::Tenant {
            id: default_tenant_id,
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        }).await.unwrap();
        store.create_catalog(default_tenant_id, pangolin_core::model::Catalog {
            catalog_type: pangolin_core::model::CatalogType::Local,
            id: uuid::Uuid::new_v4(),
            name: "analytics".to_string(),
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }).await.unwrap();

        let app = pangolin_api::app(store);

        // Request without any authentication headers
        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should succeed with default tenant
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_config_endpoint_no_auth_required() {
        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let request = Request::builder()
            .method("GET")
            .uri("/v1/config")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_bearer_token_authentication() {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use pangolin_api::auth::Claims; // Use proper Claims
        use chrono::Utc;

        let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "secret");

        let store = Arc::new(MemoryStore::new());
        
        // Setup Tenant and Catalog
        let tenant_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        store.create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: "test_tenant".to_string(),
            properties: std::collections::HashMap::new(),
        }).await.unwrap();
        store.create_catalog(tenant_id, pangolin_core::model::Catalog {
            catalog_type: pangolin_core::model::CatalogType::Local,
            id: uuid::Uuid::new_v4(),
            name: "analytics".to_string(),
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }).await.unwrap();

        let app = pangolin_api::app(store);

        // Generate valid token
        let secret = "secret";
        let claims = Claims {
            sub: uuid::Uuid::new_v4().to_string(),
            jti: Some(uuid::Uuid::new_v4().to_string()),
            username: "test_user".to_string(),
            tenant_id: Some(tenant_id.to_string()),
            role: UserRole::TenantAdmin,
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
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
    #[serial]
    async fn test_invalid_bearer_token_rejected() {
        let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "secret");
        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

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
    #[serial]
    async fn test_expired_token_rejected() {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use pangolin_api::auth::Claims;
        use chrono::Utc;

        let _guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "secret");
        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        // Generate expired token
        let secret = "secret";
        let claims = Claims {
            sub: uuid::Uuid::new_v4().to_string(),
            jti: Some(uuid::Uuid::new_v4().to_string()),
            username: "test_user".to_string(),
            tenant_id: Some("00000000-0000-0000-0000-000000000001".to_string()),
            role: UserRole::TenantUser,
            exp: Utc::now().timestamp() - 3600,
            iat: Utc::now().timestamp() - 7200,
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
    #[serial]
    async fn test_no_auth_uses_default_tenant() {
        let _guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");

        let store = Arc::new(MemoryStore::new());
        
        // Create default tenant and analytics catalog
        let default_tenant_id = uuid::Uuid::nil();
        store.create_tenant(pangolin_core::model::Tenant {
            id: default_tenant_id,
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        }).await.unwrap();
        store.create_catalog(default_tenant_id, pangolin_core::model::Catalog {
            catalog_type: pangolin_core::model::CatalogType::Local,
            id: uuid::Uuid::new_v4(),
            name: "analytics".to_string(),
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }).await.unwrap();

        let app = pangolin_api::app(store);

        let request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_basic_auth_for_root_user() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
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

    #[tokio::test]
    #[serial]
    async fn test_wrong_basic_auth_rejected() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:wrongpassword");

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
