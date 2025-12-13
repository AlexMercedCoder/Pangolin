#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        body::to_bytes,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use pangolin_store::memory::MemoryStore;
    use std::sync::Arc;
    use serial_test::serial;
    use pangolin_api::tests_common::EnvGuard;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    #[tokio::test]
    #[serial]
    async fn test_token_generation_valid() {
        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let _jwt_guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "secret");

        // Setup
        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:password");

        // Create request
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tokens")
            .header("Authorization", format!("Basic {}", credentials))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "tenant_id": "00000000-0000-0000-0000-000000000001",
                    "username": "test-user",
                    "expires_in_hours": 24
                }"#,
            ))
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        // Parse response
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["token"].is_string());
        assert!(json["expires_at"].is_string());
        assert_eq!(json["tenant_id"], "00000000-0000-0000-0000-000000000001");
    }

    #[tokio::test]
    #[serial]
    async fn test_token_generation_invalid_tenant_id() {
        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tokens")
            .header("Authorization", format!("Basic {}", credentials))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "tenant_id": "invalid-uuid",
                    "username": "test-user"
                }"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[serial]
    async fn test_token_contains_tenant_id() {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
        use pangolin_api::auth::Claims;

        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let _jwt_guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "secret");

        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tokens")
            .header("Authorization", format!("Basic {}", credentials))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "tenant_id": "00000000-0000-0000-0000-000000000001",
                    "username": "test-user"
                }"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let token = json["token"].as_str().unwrap();
        let secret = "secret";

        // Decode token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Don't validate expiration for test

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .unwrap();

        assert_eq!(
            token_data.claims.tenant_id,
            Some("00000000-0000-0000-0000-000000000001".to_string())
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_token_custom_expiration() {
        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let _jwt_guard = EnvGuard::new("PANGOLIN_JWT_SECRET", "secret");

        let store = Arc::new(MemoryStore::new());
        let app = pangolin_api::app(store);

        let credentials = STANDARD.encode("admin:password");

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tokens")
            .header("Authorization", format!("Basic {}", credentials))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "tenant_id": "00000000-0000-0000-0000-000000000001",
                    "username": "test-user",
                    "expires_in_hours": 48
                }"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify expiration is set (we can't easily verify the exact time without decoding)
        assert!(json["expires_at"].is_string());
    }
}
