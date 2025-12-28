/// Regression tests for PostgreSQL Merge Operations
/// Ensures all 11 trait methods remain fully implemented
use crate::postgres::PostgresStore;
use crate::CatalogStore;
use pangolin_core::model::{
    ConflictResolution, ConflictType, MergeConflict, MergeOperation, MergeStatus,
    ResolutionStrategy, Tenant,
};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod postgres_merge_tests {
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
    async fn test_merge_operation_create() {
        let store = setup_postgres_store().await;
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
        let store = setup_postgres_store().await;
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
        let store = setup_postgres_store().await;
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

        let list = store.list_merge_operations(tenant_id, &catalog_name, None).await.unwrap();
        assert!(list.len() >= 3, "Should have at least 3 merge operations");
    }

    #[tokio::test]
    async fn test_merge_operation_update_status() {
        let store = setup_postgres_store().await;
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

        // Verify
        let updated = store.get_merge_operation(operation_id).await.unwrap().unwrap();
        assert_eq!(updated.status, MergeStatus::Conflicted);
    }

    #[tokio::test]
    async fn test_merge_operation_complete() {
        let store = setup_postgres_store().await;
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

        // Verify
        let completed = store.get_merge_operation(operation_id).await.unwrap().unwrap();
        assert_eq!(completed.status, MergeStatus::Completed);
        assert_eq!(completed.result_commit_id, Some(commit_id));
        assert!(completed.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_merge_operation_abort() {
        let store = setup_postgres_store().await;
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

        // Verify
        let aborted = store.get_merge_operation(operation_id).await.unwrap().unwrap();
        assert_eq!(aborted.status, MergeStatus::Aborted);
        assert!(aborted.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_merge_conflict_create() {
        let store = setup_postgres_store().await;
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
        let store = setup_postgres_store().await;
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
        let store = setup_postgres_store().await;
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
                ConflictType::DataOverlap {
                    asset_name: format!("table_{}", i),
                    overlapping_partitions: vec![format!("partition_{}", i)],
                },
                Some(Uuid::new_v4()),
                format!("Conflict {}", i),
            );
            store.create_merge_conflict(conflict).await.unwrap();
        }

        let list = store.list_merge_conflicts(operation_id, None).await.unwrap();
        assert!(list.len() >= 3, "Should have at least 3 merge conflicts");
    }

    #[tokio::test]
    async fn test_merge_conflict_resolve() {
        let store = setup_postgres_store().await;
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
            ConflictType::MetadataConflict {
                asset_name: "test_table".to_string(),
                conflicting_properties: vec!["property1".to_string()],
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

        // Verify
        let resolved = store.get_merge_conflict(conflict_id).await.unwrap().unwrap();
        assert!(resolved.is_resolved(), "Conflict should be resolved");
    }

    #[tokio::test]
    async fn test_add_conflict_to_operation() {
        let store = setup_postgres_store().await;
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

        // Add conflict to operation
        let result = store.add_conflict_to_operation(operation_id, conflict_id).await;
        assert!(result.is_ok(), "Failed to add conflict to operation");

        // Verify
        let updated_op = store.get_merge_operation(operation_id).await.unwrap().unwrap();
        assert!(updated_op.conflicts.contains(&conflict_id), "Operation should contain conflict");
    }
}
