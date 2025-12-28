
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
    use pangolin_core::model::Tenant;
    use std::collections::HashMap;

    #[tokio::test]
    #[serial]
    async fn test_tenant_pagination() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        // Authenticate as root
        let _user_guard = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _pass_guard = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");

        let store = Arc::new(MemoryStore::new());
        
        // Seed 5 tenants
        for i in 0..5 {
            let tenant = Tenant {
                id: uuid::Uuid::new_v4(),
                name: format!("tenant-{}", i),
                properties: HashMap::new(),
            };
            store.create_tenant(tenant).await.unwrap();
        }

        let app = pangolin_api::app(store);
        let credentials = STANDARD.encode("admin:password");

        // Page 1: Limit 2, Offset 0
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants?limit=2&offset=0")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tenants: Vec<Tenant> = serde_json::from_slice(&body).unwrap();
        assert_eq!(tenants.len(), 2);

        // Page 2: Limit 2, Offset 2
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants?limit=2&offset=2")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tenants: Vec<Tenant> = serde_json::from_slice(&body).unwrap();
        assert_eq!(tenants.len(), 2);

        // Page 3: Limit 2, Offset 4 (Should return 1)
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/tenants?limit=2&offset=4")
            .header("Authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let tenants: Vec<Tenant> = serde_json::from_slice(&body).unwrap();
        assert_eq!(tenants.len(), 1);
    }
}
