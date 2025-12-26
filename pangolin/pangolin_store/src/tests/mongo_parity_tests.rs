/// Regression tests for MongoStore Service Users and Merge Operations
/// Ensures all trait methods remain fully implemented
use crate::mongo::MongoStore;
use crate::CatalogStore;
use pangolin_core::model::{
    ConflictResolution, ConflictType, MergeConflict, MergeOperation, MergeStatus,
    ResolutionStrategy, Tenant,
};
use pangolin_core::user::{ServiceUser, UserRole};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod mongo_parity_tests {
    use super::*;

    async fn setup_mongo_store() -> MongoStore {
        let mongodb_url = std::env::var("TEST_MONGODB_URL")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let db_name = format!("pangolin_test_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        MongoStore::new(&mongodb_url, &db_name)
            .await
            .expect("Failed to create MongoStore")
    }

    async fn setup_tenant(store: &MongoStore) -> Uuid {
        let tenant_id = Uuid::new_v4();
        let tenant = Tenant {
            id: tenant_id,
            name: format!("test_tenant_{}", tenant_id),
            properties: HashMap::new(),
        };
        store.create_tenant(tenant).await.expect("Failed to create tenant");
        tenant_id
    }

    // ==================== Service User Tests ====================

    #[tokio::test]
    async fn test_service_user_create() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_user".to_string(),
            description: Some("Test service user".to_string()),
            tenant_id,
            api_key_hash: "test_hash".to_string(),
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
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_user".to_string(),
            description: Some("Test service user".to_string()),
            tenant_id,
            api_key_hash: "test_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };
        let user_id = service_user.id;

        store.create_service_user(service_user).await.unwrap();

        let retrieved = store.get_service_user(user_id).await.unwrap();
        assert!(retrieved.is_some(), "Service user should exist");
        assert_eq!(retrieved.unwrap().id, user_id);
    }

    #[tokio::test]
    async fn test_service_user_get_by_api_key_hash() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let api_key_hash = format!("test_hash_{}", Uuid::new_v4());
        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_user".to_string(),
            description: Some("Test service user".to_string()),
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
        assert!(retrieved.is_some(), "Service user should exist");
        assert_eq!(retrieved.unwrap().api_key_hash, api_key_hash);
    }

    #[tokio::test]
    async fn test_service_user_list() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        // Create multiple service users
        for i in 0..3 {
            let service_user = ServiceUser {
                id: Uuid::new_v4(),
                name: format!("test_user_{}", i),
                description: Some(format!("Test user {}", i)),
                tenant_id,
                api_key_hash: format!("hash_{}", i),
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
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_user".to_string(),
            description: Some("Original description".to_string()),
            tenant_id,
            api_key_hash: "test_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };
        let user_id = service_user.id;

        store.create_service_user(service_user).await.unwrap();

        // Update
        let result = store.update_service_user(
            user_id,
            Some("updated_name".to_string()),
            Some("Updated description".to_string()),
            Some(false),
        ).await;
        assert!(result.is_ok(), "Failed to update service user");

        // Verify
        let updated = store.get_service_user(user_id).await.unwrap().unwrap();
        assert_eq!(updated.name, "updated_name");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.active, false);
    }

    #[tokio::test]
    async fn test_service_user_delete() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_user".to_string(),
            description: Some("Test service user".to_string()),
            tenant_id,
            api_key_hash: "test_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };
        let user_id = service_user.id;

        store.create_service_user(service_user).await.unwrap();

        // Delete
        let result = store.delete_service_user(user_id).await;
        assert!(result.is_ok(), "Failed to delete service user");

        // Verify
        let deleted = store.get_service_user(user_id).await.unwrap();
        assert!(deleted.is_none(), "Service user should be deleted");
    }

    #[tokio::test]
    async fn test_service_user_update_last_used() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_user".to_string(),
            description: Some("Test service user".to_string()),
            tenant_id,
            api_key_hash: "test_hash".to_string(),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
            last_used: None,
            expires_at: None,
            active: true,
        };
        let user_id = service_user.id;

        store.create_service_user(service_user).await.unwrap();

        // Update last used
        let timestamp = Utc::now();
        let result = store.update_service_user_last_used(user_id, timestamp).await;
        assert!(result.is_ok(), "Failed to update last used");
    }

    // ==================== Merge Operation Tests ====================

    #[tokio::test]
    async fn test_merge_operation_create() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );

        let result = store.create_merge_operation(operation.clone()).await;
        assert!(result.is_ok(), "Failed to create merge operation: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_merge_operation_get() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation_id = operation.id;

        store.create_merge_operation(operation).await.unwrap();

        let retrieved = store.get_merge_operation(operation_id).await.unwrap();
        assert!(retrieved.is_some(), "Merge operation should exist");
        assert_eq!(retrieved.unwrap().id, operation_id);
    }

    #[tokio::test]
    async fn test_merge_operation_list() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;
        let catalog_name = format!("test_catalog_{}", Uuid::new_v4());

        // Create multiple operations
        for i in 0..3 {
            let operation = MergeOperation::new(
                tenant_id,
                catalog_name.clone(),
                format!("feature-{}", i),
                "main".to_string(),
                None,
                Uuid::new_v4(),
            );
            store.create_merge_operation(operation).await.unwrap();
        }

        let list = store.list_merge_operations(tenant_id, &catalog_name).await.unwrap();
        assert!(list.len() >= 3, "Should have at least 3 merge operations");
    }

    #[tokio::test]
    async fn test_merge_operation_update_status() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation_id = operation.id;

        store.create_merge_operation(operation).await.unwrap();

        // Update status
        let result = store.update_merge_operation_status(operation_id, MergeStatus::Conflicted).await;
        assert!(result.is_ok(), "Failed to update status");
    }

    #[tokio::test]
    async fn test_merge_operation_complete() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation_id = operation.id;

        store.create_merge_operation(operation).await.unwrap();

        // Complete
        let commit_id = Uuid::new_v4();
        let result = store.complete_merge_operation(operation_id, commit_id).await;
        assert!(result.is_ok(), "Failed to complete merge operation");
    }

    #[tokio::test]
    async fn test_merge_operation_abort() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation_id = operation.id;

        store.create_merge_operation(operation).await.unwrap();

        // Abort
        let result = store.abort_merge_operation(operation_id).await;
        assert!(result.is_ok(), "Failed to abort merge operation");
    }

    #[tokio::test]
    async fn test_merge_conflict_create() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        store.create_merge_operation(operation.clone()).await.unwrap();

        let conflict = MergeConflict::new(
            operation.id,
            ConflictType::SchemaChange {
                asset_name: "test_table".to_string(),
                source_schema: serde_json::json!({"col1": "int"}),
                target_schema: serde_json::json!({"col1": "string"}),
            },
            Some(Uuid::new_v4()),
            "Schema conflict in test_table".to_string(),
        );

        let result = store.create_merge_conflict(conflict).await;
        assert!(result.is_ok(), "Failed to create merge conflict: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_merge_conflict_get() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        store.create_merge_operation(operation.clone()).await.unwrap();

        let conflict = MergeConflict::new(
            operation.id,
            ConflictType::DataOverlap {
                asset_name: "test_table".to_string(),
                overlapping_partitions: vec!["partition1".to_string()],
            },
            Some(Uuid::new_v4()),
            "Test conflict".to_string(),
        );
        let conflict_id = conflict.id;

        store.create_merge_conflict(conflict).await.unwrap();

        let retrieved = store.get_merge_conflict(conflict_id).await.unwrap();
        assert!(retrieved.is_some(), "Merge conflict should exist");
        assert_eq!(retrieved.unwrap().id, conflict_id);
    }

    #[tokio::test]
    async fn test_merge_conflict_list() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation_id = operation.id;
        store.create_merge_operation(operation).await.unwrap();

        // Create multiple conflicts
        for i in 0..3 {
            let conflict = MergeConflict::new(
                operation_id,
                ConflictType::MetadataConflict {
                    asset_name: format!("table_{}", i),
                    conflicting_properties: vec![format!("prop_{}", i)],
                },
                Some(Uuid::new_v4()),
                format!("Conflict {}", i),
            );
            store.create_merge_conflict(conflict).await.unwrap();
        }

        let list = store.list_merge_conflicts(operation_id).await.unwrap();
        assert!(list.len() >= 3, "Should have at least 3 merge conflicts");
    }

    #[tokio::test]
    async fn test_merge_conflict_resolve() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        store.create_merge_operation(operation.clone()).await.unwrap();

        let conflict = MergeConflict::new(
            operation.id,
            ConflictType::DeletionConflict {
                asset_name: "test_table".to_string(),
                deleted_in: "source".to_string(),
                modified_in: "target".to_string(),
            },
            Some(Uuid::new_v4()),
            "Test conflict".to_string(),
        );
        let conflict_id = conflict.id;

        store.create_merge_conflict(conflict).await.unwrap();

        // Resolve
        let resolution = ConflictResolution {
            conflict_id,
            strategy: ResolutionStrategy::TakeSource,
            resolved_value: None,
            resolved_by: Uuid::new_v4(),
            resolved_at: Utc::now(),
        };

        let result = store.resolve_merge_conflict(conflict_id, resolution).await;
        assert!(result.is_ok(), "Failed to resolve conflict");
    }

    #[tokio::test]
    async fn test_add_conflict_to_operation() {
        let store = setup_mongo_store().await;
        let tenant_id = setup_tenant(&store).await;

        let operation = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation_id = operation.id;
        store.create_merge_operation(operation).await.unwrap();

        let conflict = MergeConflict::new(
            operation_id,
            ConflictType::SchemaChange {
                asset_name: "test_table".to_string(),
                source_schema: serde_json::json!({"col1": "int"}),
                target_schema: serde_json::json!({"col1": "string"}),
            },
            Some(Uuid::new_v4()),
            "Test conflict".to_string(),
        );
        let conflict_id = conflict.id;
        store.create_merge_conflict(conflict).await.unwrap();

        // Add conflict to operation
        let result = store.add_conflict_to_operation(operation_id, conflict_id).await;
        assert!(result.is_ok(), "Failed to add conflict to operation");
    }
}
