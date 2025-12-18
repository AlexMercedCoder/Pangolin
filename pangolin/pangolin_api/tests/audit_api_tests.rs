use pangolin_api::audit_handlers::{AuditListQuery, AuditCountResponse};
use pangolin_core::audit::{AuditAction, AuditLogEntry, AuditResult, ResourceType};
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

#[tokio::test]
async fn test_audit_list_query_serialization() {
    // Test that AuditListQuery can be properly constructed
    let query = AuditListQuery {
        user_id: Some(Uuid::new_v4()),
        action: Some("create_table".to_string()),
        resource_type: Some("table".to_string()),
        resource_id: Some(Uuid::new_v4()),
        start_time: Some("2025-01-01T00:00:00Z".to_string()),
        end_time: Some("2025-12-31T23:59:59Z".to_string()),
        result: Some("success".to_string()),
        limit: Some(100),
        offset: Some(0),
    };
    
    assert!(query.user_id.is_some());
    assert_eq!(query.action.as_ref().unwrap(), "create_table");
    assert_eq!(query.limit.unwrap(), 100);
}

#[tokio::test]
async fn test_audit_count_response_serialization() {
    // Test that AuditCountResponse can be serialized
    let response = AuditCountResponse { count: 42 };
    
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"count\":42"));
    
    let deserialized: AuditCountResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.count, 42);
}

#[tokio::test]
async fn test_audit_logging_with_memory_store() {
    let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Create test audit log entries
    let entry1 = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "test_user".to_string(),
        AuditAction::CreateTable,
        ResourceType::Table,
        Some(Uuid::new_v4()),
        "test_table".to_string(),
    );
    
    let entry2 = AuditLogEntry::failure(
        tenant_id,
        Some(user_id),
        "test_user".to_string(),
        AuditAction::DropTable,
        ResourceType::Table,
        "missing_table".to_string(),
        "Table not found".to_string(),
    );
    
    // Log the entries
    store.log_audit_event(tenant_id, entry1.clone()).await.unwrap();
    store.log_audit_event(tenant_id, entry2.clone()).await.unwrap();
    
    // Retrieve all logs
    let logs = store.list_audit_events(tenant_id, None).await.unwrap();
    assert!(logs.len() >= 2, "Should have at least 2 audit logs");
    
    // Count logs
    let count = store.count_audit_events(tenant_id, None).await.unwrap();
    assert!(count >= 2, "Should count at least 2 audit logs");
}

#[tokio::test]
async fn test_audit_filtering_by_action() {
    let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Create different types of audit entries
    let create_entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user1".to_string(),
        AuditAction::CreateTable,
        ResourceType::Table,
        None,
        "table1".to_string(),
    );
    
    let drop_entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user1".to_string(),
        AuditAction::DropTable,
        ResourceType::Table,
        None,
        "table2".to_string(),
    );
    
    store.log_audit_event(tenant_id, create_entry).await.unwrap();
    store.log_audit_event(tenant_id, drop_entry).await.unwrap();
    
    // Filter by CreateTable action
    let filter = pangolin_core::audit::AuditLogFilter {
        action: Some(AuditAction::CreateTable),
        ..Default::default()
    };
    
    let filtered_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(filtered_logs.len(), 1, "Should only have 1 CreateTable log");
    assert_eq!(filtered_logs[0].action, AuditAction::CreateTable);
}

#[tokio::test]
async fn test_audit_filtering_by_result() {
    let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Create success and failure entries
    let success_entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user1".to_string(),
        AuditAction::CreateTable,
        ResourceType::Table,
        None,
        "table1".to_string(),
    );
    
    let failure_entry = AuditLogEntry::failure(
        tenant_id,
        Some(user_id),
        "user1".to_string(),
        AuditAction::DropTable,
        ResourceType::Table,
        "table2".to_string(),
        "Not found".to_string(),
    );
    
    store.log_audit_event(tenant_id, success_entry).await.unwrap();
    store.log_audit_event(tenant_id, failure_entry).await.unwrap();
    
    // Filter by failure result
    let filter = pangolin_core::audit::AuditLogFilter {
        result: Some(AuditResult::Failure),
        ..Default::default()
    };
    
    let filtered_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(filtered_logs.len(), 1, "Should only have 1 failure log");
    assert_eq!(filtered_logs[0].result, AuditResult::Failure);
    assert!(filtered_logs[0].error_message.is_some());
}

#[tokio::test]
async fn test_audit_pagination() {
    let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Create 10 audit entries
    for i in 0..10 {
        let entry = AuditLogEntry::success(
            tenant_id,
            Some(user_id),
            "user1".to_string(),
            AuditAction::CreateTable,
            ResourceType::Table,
            None,
            format!("table_{}", i),
        );
        store.log_audit_event(tenant_id, entry).await.unwrap();
    }
    
    // Get first page (5 items)
    let filter = pangolin_core::audit::AuditLogFilter {
        limit: Some(5),
        offset: Some(0),
        ..Default::default()
    };
    let page1 = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(page1.len(), 5, "First page should have 5 items");
    
    // Get second page (5 items)
    let filter = pangolin_core::audit::AuditLogFilter {
        limit: Some(5),
        offset: Some(5),
        ..Default::default()
    };
    let page2 = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(page2.len(), 5, "Second page should have 5 items");
    
    // Verify different items
    assert_ne!(page1[0].id, page2[0].id, "Pages should have different items");
}

#[tokio::test]
async fn test_get_specific_audit_event() {
    let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    let entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user1".to_string(),
        AuditAction::CreateCatalog,
        ResourceType::Catalog,
        None,
        "test_catalog".to_string(),
    );
    
    let event_id = entry.id;
    store.log_audit_event(tenant_id, entry).await.unwrap();
    
    // Retrieve the specific event
    let retrieved = store.get_audit_event(tenant_id, event_id).await.unwrap();
    assert!(retrieved.is_some(), "Should find the event");
    
    let event = retrieved.unwrap();
    assert_eq!(event.id, event_id);
    assert_eq!(event.action, AuditAction::CreateCatalog);
    assert_eq!(event.resource_name, "test_catalog");
}

#[tokio::test]
async fn test_tenant_isolation() {
    let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
    let tenant1_id = Uuid::new_v4();
    let tenant2_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Create entries for tenant 1
    for i in 0..3 {
        let entry = AuditLogEntry::success(
            tenant1_id,
            Some(user_id),
            "user1".to_string(),
            AuditAction::CreateTable,
            ResourceType::Table,
            None,
            format!("tenant1_table_{}", i),
        );
        store.log_audit_event(tenant1_id, entry).await.unwrap();
    }
    
    // Create entries for tenant 2
    for i in 0..2 {
        let entry = AuditLogEntry::success(
            tenant2_id,
            Some(user_id),
            "user2".to_string(),
            AuditAction::CreateTable,
            ResourceType::Table,
            None,
            format!("tenant2_table_{}", i),
        );
        store.log_audit_event(tenant2_id, entry).await.unwrap();
    }
    
    // Verify tenant 1 only sees their logs
    let tenant1_logs = store.list_audit_events(tenant1_id, None).await.unwrap();
    assert_eq!(tenant1_logs.len(), 3, "Tenant 1 should have 3 logs");
    assert!(tenant1_logs.iter().all(|log| log.tenant_id == tenant1_id));
    
    // Verify tenant 2 only sees their logs
    let tenant2_logs = store.list_audit_events(tenant2_id, None).await.unwrap();
    assert_eq!(tenant2_logs.len(), 2, "Tenant 2 should have 2 logs");
    assert!(tenant2_logs.iter().all(|log| log.tenant_id == tenant2_id));
}

#[tokio::test]
async fn test_audit_entry_builder_pattern() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Test success builder
    let success_entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "test_user".to_string(),
        AuditAction::CreateTable,
        ResourceType::Table,
        Some(Uuid::new_v4()),
        "test_table".to_string(),
    );
    
    assert_eq!(success_entry.result, AuditResult::Success);
    assert!(success_entry.error_message.is_none());
    assert_eq!(success_entry.username, "test_user");
    
    // Test failure builder
    let failure_entry = AuditLogEntry::failure(
        tenant_id,
        Some(user_id),
        "test_user".to_string(),
        AuditAction::DropTable,
        ResourceType::Table,
        "missing_table".to_string(),
        "Table not found".to_string(),
    );
    
    assert_eq!(failure_entry.result, AuditResult::Failure);
    assert_eq!(failure_entry.error_message.as_ref().unwrap(), "Table not found");
    assert!(failure_entry.resource_id.is_none());
}

#[tokio::test]
async fn test_audit_metadata() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    let mut entry = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "test_user".to_string(),
        AuditAction::CreateTable,
        ResourceType::Table,
        None,
        "test_table".to_string(),
    );
    
    // Add metadata
    let metadata = serde_json::json!({
        "schema": "public",
        "columns": 5
    });
    entry = entry.with_metadata(metadata);
    
    let meta = entry.metadata.as_ref().unwrap();
    assert_eq!(meta.get("schema").unwrap(), "public");
    assert_eq!(meta.get("columns").unwrap(), 5);
}
