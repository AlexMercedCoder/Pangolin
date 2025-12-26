/// Regression tests for PostgreSQL backend parity
/// Ensures Service Users, System Settings, and Audit Logs remain fully implemented
use crate::postgres::PostgresStore;
use crate::CatalogStore;
use pangolin_core::audit::{AuditAction, AuditLogEntry, AuditLogFilter, AuditResult, ResourceType};
use pangolin_core::model::{SystemSettings, Tenant};
use pangolin_core::user::{ServiceUser, UserRole};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod postgres_parity_tests {
    use super::*;

    async fn setup_postgres_store() -> PostgresStore {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:password@localhost:5432/pangolin_test".to_string());
        
        PostgresStore::new(&database_url)
            .await
            .expect("Failed to create PostgresStore")
    }

    async fn setup_tenant(store: &PostgresStore) -> Uuid {
        let tenant_id = Uuid::new_v4();
        let tenant = Tenant {
            id: tenant_id,
            name: format!("test_tenant_{}", tenant_id),
            properties: HashMap::new(),
        };
        store.create_tenant(tenant).await.expect("Failed to create tenant");
        tenant_id
    }

    #[tokio::test]
    async fn test_service_user_create() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test-service-user".to_string(),
            description: Some("Test service user".to_string()),
            tenant_id,
            api_key_hash: "$2b$12$test_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        let result = store.create_service_user(service_user.clone()).await;
        assert!(result.is_ok(), "Failed to create service user: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_service_user_get() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user_id = Uuid::new_v4();
        let service_user = ServiceUser {
            id: service_user_id,
            name: "test-get-service-user".to_string(),
            description: Some("Test get".to_string()),
            tenant_id,
            api_key_hash: "$2b$12$test_hash_get".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        store.create_service_user(service_user.clone()).await.unwrap();

        let retrieved = store.get_service_user(service_user_id).await.unwrap();
        assert!(retrieved.is_some(), "Service user should exist");
        assert_eq!(retrieved.unwrap().id, service_user_id);
    }

    #[tokio::test]
    async fn test_service_user_get_by_api_key_hash() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let api_key_hash = format!("$2b$12$unique_hash_{}", Uuid::new_v4());
        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: format!("test-hash-lookup-{}", Uuid::new_v4()),
            description: Some("Test hash lookup".to_string()),
            tenant_id,
            api_key_hash: api_key_hash.clone(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        store.create_service_user(service_user.clone()).await.unwrap();

        let retrieved = store.get_service_user_by_api_key_hash(&api_key_hash).await.unwrap();
        assert!(retrieved.is_some(), "Service user should be found by API key hash");
        assert_eq!(retrieved.unwrap().api_key_hash, api_key_hash);
    }

    #[tokio::test]
    async fn test_service_user_list() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        // Create multiple service users
        for i in 0..3 {
            let service_user = ServiceUser {
                id: Uuid::new_v4(),
                name: format!("test-list-user-{}-{}", tenant_id, i),
                description: Some(format!("User {}", i)),
                tenant_id,
                api_key_hash: format!("$2b$12$hash_{}", i),
                role: UserRole::TenantUser,
                created_at: Utc::now(),
                created_by: Uuid::new_v4(),
                last_used: None,
                expires_at: None,
                active: true,
            };
            store.create_service_user(service_user).await.unwrap();
        }

        let list = store.list_service_users(tenant_id).await.unwrap();
        assert!(list.len() >= 3, "Should have at least 3 service users");
    }

    #[tokio::test]
    async fn test_service_user_update() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user_id = Uuid::new_v4();
        let service_user = ServiceUser {
            id: service_user_id,
            name: "test-update-user".to_string(),
            description: Some("Original description".to_string()),
            tenant_id,
            api_key_hash: "$2b$12$update_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        store.create_service_user(service_user).await.unwrap();

        // Update
        let result = store.update_service_user(
            service_user_id,
            Some("updated-name".to_string()),
            Some("Updated description".to_string()),
            Some(false),
        ).await;
        assert!(result.is_ok(), "Failed to update service user");

        // Verify update
        let updated = store.get_service_user(service_user_id).await.unwrap().unwrap();
        assert_eq!(updated.name, "updated-name");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.active, false);
    }

    #[tokio::test]
    async fn test_service_user_delete() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user_id = Uuid::new_v4();
        let service_user = ServiceUser {
            id: service_user_id,
            name: "test-delete-user".to_string(),
            description: Some("To be deleted".to_string()),
            tenant_id,
            api_key_hash: "$2b$12$delete_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        store.create_service_user(service_user).await.unwrap();

        // Delete
        let result = store.delete_service_user(service_user_id).await;
        assert!(result.is_ok(), "Failed to delete service user");

        // Verify deletion
        let retrieved = store.get_service_user(service_user_id).await.unwrap();
        assert!(retrieved.is_none(), "Service user should be deleted");
    }

    #[tokio::test]
    async fn test_service_user_update_last_used() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user_id = Uuid::new_v4();
        let service_user = ServiceUser {
            id: service_user_id,
            name: "test-last-used-user".to_string(),
            description: Some("Test last used".to_string()),
            tenant_id,
            api_key_hash: "$2b$12$last_used_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };

        store.create_service_user(service_user).await.unwrap();

        // Update last_used
        let timestamp = Utc::now();
        let result = store.update_service_user_last_used(service_user_id, timestamp).await;
        assert!(result.is_ok(), "Failed to update last_used");

        // Verify
        let updated = store.get_service_user(service_user_id).await.unwrap().unwrap();
        assert!(updated.last_used.is_some(), "last_used should be set");
    }

    #[tokio::test]
    async fn test_system_settings_get_default() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let settings = store.get_system_settings(tenant_id).await.unwrap();
        
        // Default settings should have all None values
        assert!(settings.allow_public_signup.is_none());
        assert!(settings.default_warehouse_bucket.is_none());
        assert!(settings.default_retention_days.is_none());
    }

    #[tokio::test]
    async fn test_system_settings_update() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let new_settings = SystemSettings {
            allow_public_signup: Some(false),
            default_warehouse_bucket: Some("test-bucket".to_string()),
            default_retention_days: Some(90),
            smtp_host: Some("smtp.example.com".to_string()),
            smtp_port: Some(587),
            smtp_user: Some("user@example.com".to_string()),
            smtp_password: Some("password".to_string()),
        };

        let result = store.update_system_settings(tenant_id, new_settings.clone()).await;
        assert!(result.is_ok(), "Failed to update system settings");

        // Verify
        let retrieved = store.get_system_settings(tenant_id).await.unwrap();
        assert_eq!(retrieved.allow_public_signup, Some(false));
        assert_eq!(retrieved.default_warehouse_bucket, Some("test-bucket".to_string()));
        assert_eq!(retrieved.default_retention_days, Some(90));
    }

    #[tokio::test]
    async fn test_system_settings_upsert() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        // First update
        let settings1 = SystemSettings {
            allow_public_signup: Some(true),
            default_warehouse_bucket: Some("bucket1".to_string()),
            default_retention_days: Some(30),
            smtp_host: None,
            smtp_port: None,
            smtp_user: None,
            smtp_password: None,
        };
        store.update_system_settings(tenant_id, settings1).await.unwrap();

        // Second update (upsert)
        let settings2 = SystemSettings {
            allow_public_signup: Some(false),
            default_warehouse_bucket: Some("bucket2".to_string()),
            default_retention_days: Some(60),
            smtp_host: None,
            smtp_port: None,
            smtp_user: None,
            smtp_password: None,
        };
        store.update_system_settings(tenant_id, settings2).await.unwrap();

        // Verify latest values
        let retrieved = store.get_system_settings(tenant_id).await.unwrap();
        assert_eq!(retrieved.allow_public_signup, Some(false));
        assert_eq!(retrieved.default_warehouse_bucket, Some("bucket2".to_string()));
        assert_eq!(retrieved.default_retention_days, Some(60));
    }

    #[tokio::test]
    async fn test_audit_log_enhanced_schema() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;

        let audit_entry = AuditLogEntry {
            id: Uuid::new_v4(),
            tenant_id,
            timestamp: Utc::now(),
            user_id: Some(Uuid::new_v4()),
            username: "test_user".to_string(),
            action: AuditAction::CreateTable,
            resource_type: ResourceType::ServiceUser,
            resource_name: "test-service-user".to_string(),
            resource_id: Some(Uuid::new_v4()),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            result: AuditResult::Success,
            error_message: None,
            metadata: Some(serde_json::json!({"test": "data"})),
        };

        let result = store.log_audit_event(tenant_id, audit_entry.clone()).await;
        assert!(result.is_ok(), "Failed to log audit event with enhanced schema");

        // Verify retrieval
        let logs = store.list_audit_events(tenant_id, None).await.unwrap();
        assert!(!logs.is_empty(), "Should have at least one audit log");
    }

    #[tokio::test]
    async fn test_audit_log_filtering() {
        let store = setup_postgres_store().await;
        let tenant_id = setup_tenant(&store).await;
        let user_id = Uuid::new_v4();

        // Create multiple audit entries
        for i in 0..5 {
            let audit_entry = AuditLogEntry {
                id: Uuid::new_v4(),
                tenant_id,
                timestamp: Utc::now(),
                user_id: Some(user_id),
                username: format!("user_{}", i),
                action: AuditAction::CreateWarehouse,
                resource_type: ResourceType::Warehouse,
                resource_name: format!("resource_{}", i),
                resource_id: Some(Uuid::new_v4()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
                result: AuditResult::Success,
                error_message: None,
                metadata: None,
            };
            store.log_audit_event(tenant_id, audit_entry).await.unwrap();
        }

        // Test filtering by user_id
        let filter = AuditLogFilter {
            user_id: Some(user_id),
            action: None,
            resource_type: None,
            resource_id: None,
            start_time: None,
            end_time: None,
            result: None,
            limit: Some(10),
            offset: Some(0),
        };

        let filtered_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
        assert!(filtered_logs.len() >= 5, "Should have at least 5 filtered logs");
    }
}
