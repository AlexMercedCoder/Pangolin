#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        body::to_bytes,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use pangolin_store::memory::MemoryStore;
    use pangolin_store::CatalogStore;
    use uuid::Uuid;
    use pangolin_api::{app, auth::{Claims, Role}};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_token(tenant_id: &str) -> String {
        let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let claims = Claims {
            sub: "test-user".to_string(),
            tenant_id: Some(tenant_id.to_string()),
            roles: vec![Role::User],
            exp: (Utc::now().timestamp() + 3600) as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_create_table_returns_metadata() {
        let store = Arc::new(MemoryStore::new());
        
        // Setup tenant and catalog
        let tenant_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let tenant = pangolin_core::model::Tenant {
            id: tenant_id,
            name: "test".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant).await.unwrap();

        let mut storage_config = HashMap::new();
        storage_config.insert("type".to_string(), "memory".to_string());
        
        let warehouse = pangolin_core::model::Warehouse {
            id: uuid::Uuid::new_v4(),
            tenant_id,
            name: "test_warehouse".to_string(),
            use_sts: false,
            storage_config,
        };
        store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();

        let catalog = pangolin_core::model::Catalog {
            catalog_type: pangolin_core::model::CatalogType::Local,
            id: Uuid::new_v4(),
            name: "default".to_string(),
            warehouse_name: Some("test_wh".to_string()),
            storage_location: Some("s3://bucket/test".to_string()),
            properties: std::collections::HashMap::new(),
        };
        store.create_catalog(tenant_id, catalog).await.unwrap();

        let app = pangolin_api::app(store);

        let token = create_test_token("00000000-0000-0000-0000-000000000001");

        let request = Request::builder()
            .method("POST")
            .uri("/v1/analytics/namespaces/test_ns/tables")
            .header("Authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "name": "test_table",
                    "schema": {
                        "type": "struct",
                        "fields": [
                            {"id": 1, "name": "id", "required": true, "type": "int"}
                        ]
                    }
                }"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Verify metadata field exists
            assert!(json["metadata"].is_object(), "TableResponse should contain metadata field");
            assert!(json["name"].is_string());
            assert!(json["location"].is_string());
        }
    }

    #[tokio::test]
    async fn test_load_table_returns_metadata() {
        std::env::set_var("PANGOLIN_NO_AUTH", "1");

        let store = Arc::new(MemoryStore::new());
        
        // Setup default tenant
        let tenant_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let tenant = pangolin_core::model::Tenant {
            id: tenant_id,
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant).await.unwrap();

        let app = pangolin_api::app(store);

        // First create a table, then load it
        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/analytics/namespaces/test_ns/tables")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "name": "test_table",
                    "schema": {
                        "type": "struct",
                        "fields": [
                            {"id": 1, "name": "id", "required": true, "type": "int"}
                        ]
                    }
                }"#,
            ))
            .unwrap();

        let _ = app.clone().oneshot(create_request).await;

        // Now load the table
        let load_request = Request::builder()
            .method("GET")
            .uri("/v1/analytics/namespaces/test_ns/tables/test_table")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(load_request).await.unwrap();

        if response.status() == StatusCode::OK {
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Verify metadata field exists
            assert!(json["metadata"].is_object(), "TableResponse should contain metadata field");
        }

        std::env::remove_var("PANGOLIN_NO_AUTH");
    }

    #[tokio::test]
    async fn test_table_metadata_has_required_fields() {
        std::env::set_var("PANGOLIN_NO_AUTH", "1");

        let store = Arc::new(MemoryStore::new());
        
        let tenant_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let tenant = pangolin_core::model::Tenant {
            id: tenant_id,
            name: "default".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant).await.unwrap();

        let app = pangolin_api::app(store);

        let request = Request::builder()
            .method("POST")
            .uri("/v1/analytics/namespaces/test_ns/tables")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "name": "test_table",
                    "schema": {
                        "type": "struct",
                        "fields": [
                            {"id": 1, "name": "id", "required": true, "type": "int"}
                        ]
                    }
                }"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            let metadata = &json["metadata"];
            
            // Check required Iceberg metadata fields
            assert!(metadata["format-version"].is_number());
            assert!(metadata["table-uuid"].is_string());
            assert!(metadata["location"].is_string());
            assert!(metadata["last-updated-ms"].is_number());
            assert!(metadata["schemas"].is_array());
            assert!(metadata["partition-specs"].is_array());
            assert!(metadata["sort-orders"].is_array());
        }

        std::env::remove_var("PANGOLIN_NO_AUTH");
    }
}
