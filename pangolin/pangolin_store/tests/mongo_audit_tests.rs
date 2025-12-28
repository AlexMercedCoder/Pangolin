use pangolin_store::{CatalogStore, MongoStore};
use pangolin_core::audit::{AuditAction, AuditLogEntry, AuditLogFilter, AuditResult, ResourceType};
use uuid::Uuid;
use chrono::Utc;
use std::env;

#[tokio::test]
async fn test_mongo_audit_log_filtering() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) if url.starts_with("mongodb://") => url,
        _ => {
            println!("Skipping test_mongo_audit_log_filtering: DATABASE_URL not set to mongodb://");
            return;
        }
    };

    let store: MongoStore = MongoStore::new(&connection_string, "pangolin_test").await.expect("Failed to create MongoStore");
    let tenant_id = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let catalog_id = Uuid::new_v4();
    let table_id = Uuid::new_v4();

    // Create test audit logs
    let logs = vec![
        AuditLogEntry::success(
            tenant_id,
            Some(user1_id),
            "user1".to_string(),
            AuditAction::CreateCatalog,
            ResourceType::Catalog,
            Some(catalog_id),
            "test_catalog".to_string(),
        ),
        AuditLogEntry::success(
            tenant_id,
            Some(user1_id),
            "user1".to_string(),
            AuditAction::CreateTable,
            ResourceType::Table,
            Some(table_id),
            "test_table".to_string(),
        ),
        AuditLogEntry::success(
            tenant_id,
            Some(user2_id),
            "user2".to_string(),
            AuditAction::CreateNamespace,
            ResourceType::Namespace,
            None,
            "test_namespace".to_string(),
        ),
        AuditLogEntry::failure(
            tenant_id,
            Some(user2_id),
            "user2".to_string(),
            AuditAction::DropTable,
            ResourceType::Table,
            "missing_table".to_string(),
            "Table not found".to_string(),
        ),
    ];

    // Log all events
    for log in &logs {
        store.log_audit_event(log.clone()).await.unwrap();
    }

    // Test 1: Get all logs (no filter)
    let all_logs = store.list_audit_events(tenant_id, None).await.unwrap();
    assert!(all_logs.len() >= 4, "Should have at least 4 logs");
    println!("✓ Test 1 passed: Retrieved all logs");

    // Test 2: Filter by user_id
    let filter = AuditLogFilter {
        user_id: Some(user1_id),
        ..Default::default()
    };
    let user1_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(user1_logs.len(), 2, "User1 should have 2 logs");
    assert!(user1_logs.iter().all(|l| l.user_id == Some(user1_id)));
    println!("✓ Test 2 passed: Filter by user_id");

    // Test 3: Filter by action
    let filter = AuditLogFilter {
        action: Some(AuditAction::CreateTable),
        ..Default::default()
    };
    let create_table_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(create_table_logs.len(), 1, "Should have 1 CreateTable log");
    assert_eq!(create_table_logs[0].action, AuditAction::CreateTable);
    println!("✓ Test 3 passed: Filter by action");

    // Test 4: Filter by resource_type
    let filter = AuditLogFilter {
        resource_type: Some(ResourceType::Table),
        ..Default::default()
    };
    let table_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(table_logs.len(), 2, "Should have 2 Table logs");
    println!("✓ Test 4 passed: Filter by resource_type");

    // Test 5: Filter by result (success)
    let filter = AuditLogFilter {
        result: Some(AuditResult::Success),
        ..Default::default()
    };
    let success_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(success_logs.len(), 3, "Should have 3 success logs");
    println!("✓ Test 5 passed: Filter by result (success)");

    // Test 6: Filter by result (failure)
    let filter = AuditLogFilter {
        result: Some(AuditResult::Failure),
        ..Default::default()
    };
    let failure_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(failure_logs.len(), 1, "Should have 1 failure log");
    assert_eq!(failure_logs[0].result, AuditResult::Failure);
    println!("✓ Test 6 passed: Filter by result (failure)");

    // Test 7: Combined filters
    let filter = AuditLogFilter {
        user_id: Some(user1_id),
        result: Some(AuditResult::Success),
        ..Default::default()
    };
    let combined_logs = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(combined_logs.len(), 2, "Should have 2 logs matching combined filter");
    println!("✓ Test 7 passed: Combined filters");

    // Test 8: Pagination
    let filter = AuditLogFilter {
        limit: Some(2),
        offset: Some(0),
        ..Default::default()
    };
    let page1 = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(page1.len(), 2, "First page should have 2 logs");

    let filter = AuditLogFilter {
        limit: Some(2),
        offset: Some(2),
        ..Default::default()
    };
    let page2 = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert!(page2.len() >= 2, "Second page should have at least 2 logs");
    println!("✓ Test 8 passed: Pagination");

    // Test 9: Count events
    let count = store.count_audit_events(tenant_id, None).await.unwrap();
    assert!(count >= 4, "Should have at least 4 events");

    let filter = AuditLogFilter {
        user_id: Some(user1_id),
        ..Default::default()
    };
    let user1_count = store.count_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(user1_count, 2, "User1 should have 2 events");
    println!("✓ Test 9 passed: Count events");

    // Test 10: Get individual event
    let event_id = logs[0].id;
    let event = store.get_audit_event(event_id).await.unwrap();
    assert!(event.is_some(), "Should find the event");
    assert_eq!(event.unwrap().id, event_id);
    println!("✓ Test 10 passed: Get individual event");

    println!("\n✅ All MongoDB audit logging tests passed!");
}

#[tokio::test]
async fn test_mongo_audit_log_time_filtering() {
    let connection_string = match env::var("DATABASE_URL") {
        Ok(url) if url.starts_with("mongodb://") => url,
        _ => {
            println!("Skipping test_mongo_audit_log_time_filtering: DATABASE_URL not set to mongodb://");
            return;
        }
    };

    let store: MongoStore = MongoStore::new(&connection_string, "pangolin_test").await.expect("Failed to create MongoStore");
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Create logs at different times
    let now = Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);
    let two_hours_ago = now - chrono::Duration::hours(2);

    let mut log1 = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user".to_string(),
        AuditAction::CreateCatalog,
        ResourceType::Catalog,
        None,
        "catalog1".to_string(),
    );
    log1.timestamp = two_hours_ago;

    let mut log2 = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user".to_string(),
        AuditAction::CreateTable,
        ResourceType::Table,
        None,
        "table1".to_string(),
    );
    log2.timestamp = one_hour_ago;

    let log3 = AuditLogEntry::success(
        tenant_id,
        Some(user_id),
        "user".to_string(),
        AuditAction::CreateNamespace,
        ResourceType::Namespace,
        None,
        "namespace1".to_string(),
    );

    store.log_audit_event(log1).await.unwrap();
    store.log_audit_event(log2).await.unwrap();
    store.log_audit_event(log3).await.unwrap();

    // Test: Filter by start_time (last hour)
    let filter = AuditLogFilter {
        start_time: Some(one_hour_ago - chrono::Duration::minutes(1)),
        ..Default::default()
    };
    let recent_logs: Vec<AuditLogEntry> = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(recent_logs.len(), 2, "Should have 2 logs from last hour");

    // Test: Filter by end_time
    let filter = AuditLogFilter {
        end_time: Some(one_hour_ago + chrono::Duration::minutes(1)),
        ..Default::default()
    };
    let old_logs: Vec<AuditLogEntry> = store.list_audit_events(tenant_id, Some(filter)).await.unwrap();
    assert_eq!(old_logs.len(), 2, "Should have 2 logs before one hour ago");

    println!("✓ MongoDB time filtering test passed");
}
