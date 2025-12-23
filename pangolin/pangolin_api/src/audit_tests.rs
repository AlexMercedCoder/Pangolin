#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;
    use pangolin_store::memory::MemoryStore;
    use pangolin_store::CatalogStore;
    use std::sync::Arc;
    use serde_json::json;
    use crate::tests_common::EnvGuard;
    use uuid::Uuid;
    use serial_test::serial;
    use crate::auth::TenantId;
    use pangolin_core::audit::AuditAction;

    fn app_with_store() -> (Router, Arc<MemoryStore>) {
        let store = Arc::new(MemoryStore::new());
        (crate::app(store.clone()), store)
    }

    async fn get_auth_token(app: &Router, username: &str, password: &str) -> String {
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/users/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({
                        "username": username,
                        "password": password
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        json["token"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    #[serial]
    async fn test_iceberg_audit_logs() {
        let _guard_user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
        let _guard_pass = EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password");
        let (app, store) = app_with_store();
        
        let root_token = get_auth_token(&app, "admin", "password").await;
        
        // 1. Setup Tenant and User
        let t_res = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tenants").header("Authorization", format!("Bearer {}", root_token)).header("Content-Type", "application/json").body(Body::from(json!({"name":"audit-tenant", "organization":"org"}).to_string())).unwrap()).await.unwrap();
        let t_id_str = serde_json::from_slice::<serde_json::Value>(&to_bytes(t_res.into_body(), usize::MAX).await.unwrap()).unwrap()["id"].as_str().unwrap().to_string();
        let tenant_id = Uuid::parse_str(&t_id_str).unwrap();

         let u_res = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/users").header("Authorization", format!("Bearer {}", root_token)).header("Content-Type", "application/json").body(Body::from(json!({"username":"audit_user","email":"a@t.com","password":"pw","role":"tenant-admin","tenant_id":t_id_str}).to_string())).unwrap()).await.unwrap();
         assert_eq!(u_res.status(), StatusCode::CREATED);
        let user_token = get_auth_token(&app, "audit_user", "pw").await;

        // 2. Setup Warehouse and Catalog
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/warehouses").header("Authorization", format!("Bearer {}", user_token)).header("Content-Type", "application/json").body(Body::from(json!({"name":"wh","use_sts":false,"storage_config":{"type":"s3","bucket":"bh"}}).to_string())).unwrap()).await.unwrap();
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/catalogs").header("Authorization", format!("Bearer {}", user_token)).header("Content-Type", "application/json").body(Body::from(json!({"name":"cat","warehouse_name":"wh","type":"pangolin"}).to_string())).unwrap()).await.unwrap();

        // 3. Perform Operations and Verify Audit Logs
        
        // A. Create Namespace
        app.clone().oneshot(Request::builder().method("POST").uri("/v1/cat/v1/namespaces").header("Authorization", format!("Bearer {}", user_token)).header("Content-Type", "application/json").body(Body::from(json!({"namespace": ["ns1"]}).to_string())).unwrap()).await.unwrap();
        
        // B. Create Table
        app.clone().oneshot(Request::builder().method("POST").uri("/v1/cat/v1/namespaces/ns1/tables").header("Authorization", format!("Bearer {}", user_token)).header("Content-Type", "application/json").body(Body::from(json!({
            "name": "tbl1",
            "schema": { "type": "struct", "fields": [] },
            "location": "s3://bh/ns1/tbl1"
        }).to_string())).unwrap()).await.unwrap();

        // C. Delete Table
        app.clone().oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/cat/v1/namespaces/ns1/tables/tbl1")
                .header("Authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        // 4. Verify Store Logs
        let logs = store.list_audit_events(tenant_id, None).await.unwrap();
        
        // Check for "audit_user" instead of "system"
        let create_table_log = logs.iter().find(|l| l.action == AuditAction::CreateTable).expect("create_table log missing");
        assert_eq!(create_table_log.username, "audit_user");
        
        // Iceberg "delete_table" logs as "DropTable" in legacy_new
        let delete_table_log = logs.iter().find(|l| l.action == AuditAction::DropTable).expect("drop_table log missing");
        assert_eq!(delete_table_log.username, "audit_user");

        let create_ns_log = logs.iter().find(|l| l.action == AuditAction::CreateNamespace).expect("create_namespace log missing");
        assert_eq!(create_ns_log.username, "audit_user");
    }
}
