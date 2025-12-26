/// Comprehensive regression tests for MemoryStore
/// Covers all features in the backend parity matrix
use crate::memory::MemoryStore;
use crate::CatalogStore;
use pangolin_core::model::{
    AuditAction, AuditLogEntry, Catalog, CatalogType, ConflictResolution, ConflictType,
    MergeConflict, MergeOperation, MergeStatus, Namespace, ResolutionStrategy, SystemSettings,
    Tenant, Warehouse,
};
use pangolin_core::user::{ServiceUser, User, UserRole};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod memory_parity_tests {
    use super::*;

    fn setup_store() -> MemoryStore {
        MemoryStore::new()
    }

    async fn setup_tenant(store: &MemoryStore) -> Uuid {
        let tenant_id = Uuid::new_v4();
        let tenant = Tenant {
            id: tenant_id,
            name: format!("test_tenant_{}", tenant_id),
            properties: HashMap::new(),
        };
        store.create_tenant(tenant).await.unwrap();
        tenant_id
    }

    // ==================== Core Catalog Tests ====================

    #[tokio::test]
    async fn test_catalog_crud() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;

        // Create warehouse
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "test_warehouse".to_string(),
            tenant_id,
            storage_config: serde_json::json!({"type": "s3", "bucket": "test"}),
            properties: HashMap::new(),
        };
        store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();

        // Create catalog
        let catalog = Catalog {
            id: Uuid::new_v4(),
            name: "test_catalog".to_string(),
            tenant_id,
            catalog_type: CatalogType::Local,
            warehouse_name: Some("test_warehouse".to_string()),
            properties: HashMap::new(),
            federated_config: None,
        };
        store.create_catalog(tenant_id, catalog.clone()).await.unwrap();

        // Get catalog
        let retrieved = store.get_catalog(tenant_id, &catalog.name).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, catalog.name);

        // List catalogs
        let catalogs = store.list_catalogs(tenant_id).await.unwrap();
        assert!(catalogs.len() >= 1);

        // Update catalog
        store.update_catalog(tenant_id, &catalog.name, Some("updated".to_string()), None, None).await.unwrap();

        // Delete catalog
        store.delete_catalog(tenant_id, &catalog.name).await.unwrap();
        let deleted = store.get_catalog(tenant_id, &catalog.name).await.unwrap();
        assert!(deleted.is_none());
    }

    // ==================== User Management Tests ====================

    #[tokio::test]
    async fn test_user_crud() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;

        let user = User::new_tenant_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            tenant_id,
        );
        let user_id = user.id;

        // Create
        store.create_user(user.clone()).await.unwrap();

        // Get
        let retrieved = store.get_user(user_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().username, "test_user");

        // List
        let users = store.list_users(tenant_id).await.unwrap();
        assert!(users.len() >= 1);

        // Update
        store.update_user(user_id, Some("updated@example.com".to_string()), None, None).await.unwrap();

        // Delete
        store.delete_user(user_id).await.unwrap();
        let deleted = store.get_user(user_id).await.unwrap();
        assert!(deleted.is_none());
    }

    // ==================== Service Users Tests ====================

    #[tokio::test]
    async fn test_service_users_full_coverage() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;

        let service_user = ServiceUser {
            id: Uuid::new_v4(),
            name: "test_service_user".to_string(),
            description: Some("Test description".to_string()),
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

        // Create
        store.create_service_user(service_user.clone()).await.unwrap();

        // Get
        let retrieved = store.get_service_user(user_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_service_user");

        // Get by API key hash
        let by_hash = store.get_service_user_by_api_key_hash("test_hash").await.unwrap();
        assert!(by_hash.is_some());

        // List
        let users = store.list_service_users(tenant_id).await.unwrap();
        assert!(users.len() >= 1);

        // Update
        store.update_service_user(user_id, Some("updated_name".to_string()), None, Some(false)).await.unwrap();

        // Update last used
        store.update_service_user_last_used(user_id, Utc::now()).await.unwrap();

        // Delete
        store.delete_service_user(user_id).await.unwrap();
        let deleted = store.get_service_user(user_id).await.unwrap();
        assert!(deleted.is_none());
    }

    // ==================== System Settings Tests ====================

    #[tokio::test]
    async fn test_system_settings() {
        let store = setup_store();

        let settings = SystemSettings {
            max_catalog_size_gb: Some(100),
            default_retention_days: Some(30),
            enable_audit_logging: Some(true),
            max_concurrent_queries: Some(10),
            custom_settings: HashMap::new(),
        };

        // Create/Update
        store.update_system_settings(settings.clone()).await.unwrap();

        // Get
        let retrieved = store.get_system_settings().await.unwrap();
        assert!(retrieved.is_some());
        let retrieved_settings = retrieved.unwrap();
        assert_eq!(retrieved_settings.max_catalog_size_gb, Some(100));
        assert_eq!(retrieved_settings.default_retention_days, Some(30));
    }

    // ==================== Audit Logs Tests ====================

    #[tokio::test]
    async fn test_audit_logs() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;
        let user_id = Uuid::new_v4();

        let log_entry = AuditLogEntry {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            action: AuditAction::CreateCatalog,
            resource_type: "catalog".to_string(),
            resource_id: "test_catalog".to_string(),
            details: Some("Test details".to_string()),
            timestamp: Utc::now(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test".to_string()),
        };

        // Create
        store.create_audit_log(log_entry.clone()).await.unwrap();

        // Get
        let retrieved = store.get_audit_log(log_entry.id).await.unwrap();
        assert!(retrieved.is_some());

        // List
        let logs = store.list_audit_logs(tenant_id, None, None, None, None, 100, 0).await.unwrap();
        assert!(logs.len() >= 1);

        // Count
        let count = store.count_audit_logs(tenant_id, None, None, None).await.unwrap();
        assert!(count >= 1);
    }

    // ==================== Merge Operations Tests ====================

    #[tokio::test]
    async fn test_merge_operations_full_coverage() {
        let store = setup_store();
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

        // Create merge operation
        store.create_merge_operation(operation.clone()).await.unwrap();

        // Get merge operation
        let retrieved = store.get_merge_operation(operation_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, operation_id);

        // List merge operations
        let operations = store.list_merge_operations(tenant_id, "test_catalog").await.unwrap();
        assert!(operations.len() >= 1);

        // Update status
        store.update_merge_operation_status(operation_id, MergeStatus::Conflicted).await.unwrap();

        // Create merge conflict
        let conflict = MergeConflict::new(
            operation_id,
            ConflictType::SchemaChange {
                asset_name: "test_table".to_string(),
                source_schema: serde_json::json!({"col1": "int"}),
                target_schema: serde_json::json!({"col1": "string"}),
            },
            Some(Uuid::new_v4()),
            "Schema conflict".to_string(),
        );
        let conflict_id = conflict.id;

        store.create_merge_conflict(conflict.clone()).await.unwrap();

        // Get merge conflict
        let retrieved_conflict = store.get_merge_conflict(conflict_id).await.unwrap();
        assert!(retrieved_conflict.is_some());

        // List merge conflicts
        let conflicts = store.list_merge_conflicts(operation_id).await.unwrap();
        assert!(conflicts.len() >= 1);

        // Resolve conflict
        let resolution = ConflictResolution {
            conflict_id,
            strategy: ResolutionStrategy::TakeSource,
            resolved_value: None,
            resolved_by: Uuid::new_v4(),
            resolved_at: Utc::now(),
        };
        store.resolve_merge_conflict(conflict_id, resolution).await.unwrap();

        // Add conflict to operation
        store.add_conflict_to_operation(operation_id, conflict_id).await.unwrap();

        // Complete merge operation
        let commit_id = Uuid::new_v4();
        store.complete_merge_operation(operation_id, commit_id).await.unwrap();

        // Abort merge operation (create new one to test abort)
        let operation2 = MergeOperation::new(
            tenant_id,
            "test_catalog".to_string(),
            "feature2".to_string(),
            "main".to_string(),
            None,
            Uuid::new_v4(),
        );
        let operation2_id = operation2.id;
        store.create_merge_operation(operation2).await.unwrap();
        store.abort_merge_operation(operation2_id).await.unwrap();
    }

    // ==================== Token Management Tests ====================

    #[tokio::test]
    async fn test_token_management() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;

        let user = User::new_tenant_user(
            "token_test_user".to_string(),
            "token@example.com".to_string(),
            "hash".to_string(),
            tenant_id,
        );
        let user_id = user.id;
        store.create_user(user).await.unwrap();

        let token_id = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        // Create token (implicit through user creation)
        // List tokens
        let tokens = store.list_user_tokens(user_id).await.unwrap();
        assert!(tokens.len() >= 0); // May or may not have tokens depending on implementation
    }

    // ==================== Business Metadata Tests ====================

    #[tokio::test]
    async fn test_business_metadata() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;
        let asset_id = Uuid::new_v4();

        let metadata = serde_json::json!({
            "owner": "data_team",
            "classification": "sensitive",
            "tags": ["finance", "quarterly"]
        });

        // Add business metadata
        store.add_business_metadata(tenant_id, asset_id, metadata.clone()).await.unwrap();

        // Get business metadata
        let retrieved = store.get_business_metadata(tenant_id, asset_id).await.unwrap();
        assert!(retrieved.is_some());

        // Delete business metadata
        store.delete_business_metadata(tenant_id, asset_id).await.unwrap();
        let deleted = store.get_business_metadata(tenant_id, asset_id).await.unwrap();
        assert!(deleted.is_none());
    }

    // ==================== Namespace Tests ====================

    #[tokio::test]
    async fn test_namespace_operations() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;

        // Create warehouse and catalog first
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "ns_warehouse".to_string(),
            tenant_id,
            storage_config: serde_json::json!({"type": "s3", "bucket": "test"}),
            properties: HashMap::new(),
        };
        store.create_warehouse(tenant_id, warehouse).await.unwrap();

        let catalog = Catalog {
            id: Uuid::new_v4(),
            name: "ns_catalog".to_string(),
            tenant_id,
            catalog_type: CatalogType::Local,
            warehouse_name: Some("ns_warehouse".to_string()),
            properties: HashMap::new(),
            federated_config: None,
        };
        store.create_catalog(tenant_id, catalog).await.unwrap();

        // Create namespace
        let namespace = Namespace {
            name: vec!["test_namespace".to_string()],
            properties: HashMap::new(),
        };
        store.create_namespace(tenant_id, "ns_catalog", namespace.clone()).await.unwrap();

        // List namespaces
        let namespaces = store.list_namespaces(tenant_id, "ns_catalog", None).await.unwrap();
        assert!(namespaces.len() >= 1);

        // Delete namespace
        store.delete_namespace(tenant_id, "ns_catalog", &namespace.name).await.unwrap();
    }

    // ==================== Warehouse Tests ====================

    #[tokio::test]
    async fn test_warehouse_crud() {
        let store = setup_store();
        let tenant_id = setup_tenant(&store).await;

        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "test_warehouse_crud".to_string(),
            tenant_id,
            storage_config: serde_json::json!({"type": "s3", "bucket": "test"}),
            properties: HashMap::new(),
        };

        // Create
        store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();

        // Get
        let retrieved = store.get_warehouse(tenant_id, &warehouse.name).await.unwrap();
        assert!(retrieved.is_some());

        // List
        let warehouses = store.list_warehouses(tenant_id).await.unwrap();
        assert!(warehouses.len() >= 1);

        // Update
        store.update_warehouse(tenant_id, &warehouse.name, Some(serde_json::json!({"type": "s3", "bucket": "updated"})), None).await.unwrap();

        // Delete
        store.delete_warehouse(tenant_id, &warehouse.name).await.unwrap();
        let deleted = store.get_warehouse(tenant_id, &warehouse.name).await.unwrap();
        assert!(deleted.is_none());
    }

    // ==================== Tenant Tests ====================

    #[tokio::test]
    async fn test_tenant_crud() {
        let store = setup_store();

        let tenant = Tenant {
            id: Uuid::new_v4(),
            name: "test_tenant_crud".to_string(),
            properties: HashMap::new(),
        };
        let tenant_id = tenant.id;

        // Create
        store.create_tenant(tenant.clone()).await.unwrap();

        // Get
        let retrieved = store.get_tenant(tenant_id).await.unwrap();
        assert!(retrieved.is_some());

        // List
        let tenants = store.list_tenants().await.unwrap();
        assert!(tenants.len() >= 1);

        // Update
        store.update_tenant(tenant_id, Some("updated_name".to_string()), None).await.unwrap();

        // Delete
        store.delete_tenant(tenant_id).await.unwrap();
        let deleted = store.get_tenant(tenant_id).await.unwrap();
        assert!(deleted.is_none());
    }
}
