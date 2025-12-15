#[cfg(test)]
mod iceberg_endpoint_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;
    use pangolin_store::memory::MemoryStore;
    use pangolin_store::CatalogStore;
    use pangolin_api::app;
    use std::sync::Arc;
    use pangolin_core::model::{Tenant, Warehouse, Catalog};
    use uuid::Uuid;
    use serial_test::serial;
    use pangolin_api::tests_common::EnvGuard;

    async fn setup_test_env() -> (Arc<dyn pangolin_store::CatalogStore + Send + Sync>, Uuid, EnvGuard) {
        let guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");
        let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
        let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        
        // Create tenant
        store.create_tenant(Tenant {
            id: tenant_id,
            name: "test_tenant".to_string(),
            properties: std::collections::HashMap::new(),
        }).await.unwrap();

        // Create warehouse
        let mut storage_config = std::collections::HashMap::new();
        storage_config.insert("type".to_string(), "memory".to_string());
        
        store.create_warehouse(tenant_id, Warehouse {
            id: Uuid::new_v4(),
            name: "test_warehouse".to_string(),
            tenant_id,
            storage_config,
            use_sts: false,
        }).await.unwrap();

        // Create catalog
        store.create_catalog(tenant_id, Catalog {
            catalog_type: pangolin_core::model::CatalogType::Local,
            id: Uuid::new_v4(),
            name: "test_warehouse".to_string(), // Name matches the prefix used in tests
            warehouse_name: Some("test_warehouse".to_string()),
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        }).await.unwrap();

        (store, tenant_id, guard)
    }

    // ===== Config Endpoint Tests =====
    
    #[tokio::test]
    #[serial]
    async fn test_get_config_without_warehouse_param() {
        let (store, _, _guard) = setup_test_env().await;
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
        assert!(json.get("defaults").is_some());
        assert!(json.get("overrides").is_some());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_config_with_warehouse_param() {
        let (store, _, _guard) = setup_test_env().await;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/config?warehouse=test_warehouse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ===== Namespace Endpoint Tests =====

    #[tokio::test]
    #[serial]
    async fn test_list_namespaces_empty() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("namespaces").is_some());
    }

    #[tokio::test]
    #[serial]
    async fn test_create_namespace() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let create_payload = serde_json::json!({
            "namespace": ["test_namespace"],
            "properties": {}
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces")
                    .method("POST")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&create_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_namespace() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        
        // Create namespace first
        store.create_namespace(
            tenant_id,
            "test_warehouse",
            pangolin_core::model::Namespace {
                name: vec!["test_ns".to_string()],
                properties: std::collections::HashMap::new(),
            },
        ).await.unwrap();

        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns")
                    .method("DELETE")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_namespace_properties() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        
        // Create namespace first
        store.create_namespace(
            tenant_id,
            "test_warehouse",
            pangolin_core::model::Namespace {
                name: vec!["test_ns".to_string()],
                properties: std::collections::HashMap::new(),
            },
        ).await.unwrap();

        let app = app(store);

        let update_payload = serde_json::json!({
            "updates": {
                "key1": "value1"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns/properties")
                    .method("POST")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ===== Table Endpoint Tests =====

    #[tokio::test]
    #[serial]
    async fn test_list_tables_empty() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        
        // Create namespace first
        store.create_namespace(
            tenant_id,
            "test_warehouse",
            pangolin_core::model::Namespace {
                name: vec!["test_ns".to_string()],
                properties: std::collections::HashMap::new(),
            },
        ).await.unwrap();

        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns/tables")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_table() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        
        // Create namespace first
        store.create_namespace(
            tenant_id,
            "test_warehouse",
            pangolin_core::model::Namespace {
                name: vec!["test_ns".to_string()],
                properties: std::collections::HashMap::new(),
            },
        ).await.unwrap();

        let app = app(store);

        let create_payload = serde_json::json!({
            "name": "test_table",
            "location": "s3://bucket/test_ns/test_table",
            "schema": {
                "type": "struct",
                "fields": [
                    {"id": 1, "name": "id", "type": "int", "required": true}
                ]
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns/tables")
                    .method("POST")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&create_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 200 or 201
        assert!(response.status().is_success());
    }

    #[tokio::test]
    #[serial]
    async fn test_table_exists_head_request() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns/tables/test_table")
                    .method("HEAD")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 404 for non-existent table
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_rename_table() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let rename_payload = serde_json::json!({
            "source": {
                "namespace": ["test_ns"],
                "name": "old_table"
            },
            "destination": {
                "namespace": ["test_ns"],
                "name": "new_table"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/tables/rename")
                    .method("POST")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&rename_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 404 for non-existent table
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_report_metrics() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let metrics_payload = serde_json::json!({
            "report": {
                "type": "scan-report",
                "table-name": "test_table",
                "snapshot-id": 123456,
                "metrics": {}
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns/tables/test_table/metrics")
                    .method("POST")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&metrics_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Metrics endpoint should accept the request
        assert!(response.status().is_success() || response.status() == StatusCode::NO_CONTENT);
    }

    // ===== Error Handling Tests =====

    #[tokio::test]
    #[serial]
    async fn test_namespace_not_found() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/nonexistent")
                    .method("DELETE")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_table_not_found() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces/test_ns/tables/nonexistent")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_invalid_json_payload() {
        let (store, tenant_id, _guard) = setup_test_env().await;
        let app = app(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/test_warehouse/namespaces")
                    .method("POST")
                    .header("X-Pangolin-Tenant", tenant_id.to_string())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
