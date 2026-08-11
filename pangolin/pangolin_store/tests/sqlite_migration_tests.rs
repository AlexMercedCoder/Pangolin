//! Upgrade path for an existing SQLite database.
//!
//! The `audit_logs` fix was initially applied by editing `sqlite_schema.sql` and
//! bumping `SQLITE_SCHEMA_VERSION`. That is not a migration: the schema file is
//! written entirely with `CREATE TABLE IF NOT EXISTS`, which does nothing at all
//! when the table already exists. Fresh installs got the new columns and every
//! upgraded database kept the broken ones - with a version number now claiming
//! otherwise, which is worse than no version at all.
//!
//! These tests build a database with the genuine pre-v2 shape and assert the
//! upgrade actually works, which is the only way to tell a real migration from a
//! version bump.

use pangolin_core::audit::{AuditAction, AuditLogEntry, ResourceType};
use pangolin_store::{sqlite::SQLITE_SCHEMA_VERSION, SqliteStore};
use std::collections::HashMap;
use uuid::Uuid;

/// The `audit_logs` table exactly as it was declared before v2.
const V1_AUDIT_LOGS: &str = "CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    details TEXT
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_ts ON audit_logs(tenant_id, timestamp DESC);";

/// A unique on-disk database per test; SQLite needs a real file for the
/// rename-and-recreate the migration performs.
fn temp_db_url() -> (std::path::PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("pangolin_migration_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("pangolin.db");
    let url = format!("sqlite://{}?mode=rwc", path.to_string_lossy());
    (dir, url)
}

/// Stand up a store whose `audit_logs` has the pre-v2 shape, as an upgrading
/// deployment's database does.
async fn legacy_store(url: &str) -> SqliteStore {
    let store = SqliteStore::new(url).await.expect("open sqlite");

    // The tenants table has to exist first: audit_logs carries a foreign key to
    // it in the real schema, and the tests below insert a tenant.
    store
        .apply_schema(
            "CREATE TABLE IF NOT EXISTS tenants (
                 id TEXT PRIMARY KEY,
                 name TEXT NOT NULL,
                 properties TEXT
             );",
        )
        .await
        .expect("seed tenants");

    store.apply_schema(V1_AUDIT_LOGS).await.expect("seed v1");
    store
}

async fn audit_columns(store: &SqliteStore, table: &str) -> Vec<String> {
    store.table_columns(table).await.expect("table_info")
}

/// The core regression: after upgrading, an audit write must succeed.
///
/// Before the migration existed this failed with "table audit_logs has no
/// column named user_id" - on a database that `run_migrations` had just
/// reported as being at the current version.
#[tokio::test]
async fn upgrading_a_v1_database_makes_audit_logging_work() {
    let (_dir, url) = temp_db_url();
    let store = legacy_store(&url).await;

    // Sanity: the fixture really is the old shape.
    let before = audit_columns(&store, "audit_logs").await;
    assert!(
        before.iter().any(|c| c == "actor"),
        "fixture should start with the pre-v2 shape, got {before:?}"
    );
    assert!(
        !before.iter().any(|c| c == "user_id"),
        "fixture should not already have the v2 columns"
    );

    store.run_migrations().await.expect("migrate");

    let after = audit_columns(&store, "audit_logs").await;
    for expected in [
        "user_id",
        "username",
        "resource_type",
        "resource_id",
        "resource_name",
        "ip_address",
        "user_agent",
        "result",
        "error_message",
        "metadata",
    ] {
        assert!(
            after.iter().any(|c| c == expected),
            "column {expected} missing after migration; got {after:?}"
        );
    }

    // The behaviour the columns exist for.
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: "upgraded".to_string(),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    let entry = AuditLogEntry::success(
        tenant_id,
        Some(Uuid::new_v4()),
        "upgrader".to_string(),
        AuditAction::CreateBranch,
        ResourceType::Branch,
        Some(Uuid::new_v4()),
        "cat/feature".to_string(),
    );

    store
        .log_audit_event(tenant_id, entry)
        .await
        .expect("an audit write must succeed after upgrading");

    let events = store
        .list_audit_events(tenant_id, None)
        .await
        .expect("list audit events");
    assert_eq!(events.len(), 1, "the audit event should be readable back");
    assert_eq!(
        events[0].action,
        AuditAction::CreateBranch,
        "the action must round-trip, not collapse to a default (B22)"
    );
}

/// The old table is preserved rather than dropped.
#[tokio::test]
async fn upgrading_keeps_the_old_table_as_a_backup() {
    let (_dir, url) = temp_db_url();
    let store = legacy_store(&url).await;

    store.run_migrations().await.expect("migrate");

    let backup = audit_columns(&store, "audit_logs_pre_v2").await;
    assert!(
        backup.iter().any(|c| c == "actor"),
        "the pre-v2 table should be kept as audit_logs_pre_v2, got {backup:?}"
    );
}

/// Running migrations twice must be a no-op, not a second rename that clobbers
/// the real table.
#[tokio::test]
async fn migrating_twice_is_idempotent() {
    let (_dir, url) = temp_db_url();
    let store = legacy_store(&url).await;

    store.run_migrations().await.expect("first migrate");
    store
        .run_migrations()
        .await
        .expect("second migrate must be a no-op");

    let after = audit_columns(&store, "audit_logs").await;
    assert!(
        after.iter().any(|c| c == "user_id"),
        "audit_logs should still be the v2 shape after a second run, got {after:?}"
    );
    assert_eq!(
        store.schema_version().await.expect("version"),
        Some(SQLITE_SCHEMA_VERSION)
    );
}

/// A fresh database needs no migration and lands on the current shape.
#[tokio::test]
async fn a_fresh_database_is_created_at_the_current_version() {
    let (_dir, url) = temp_db_url();
    let store = SqliteStore::new(&url).await.expect("open sqlite");

    store.run_migrations().await.expect("migrate");

    let columns = audit_columns(&store, "audit_logs").await;
    assert!(columns.iter().any(|c| c == "user_id"));
    assert_eq!(
        store.schema_version().await.expect("version"),
        Some(SQLITE_SCHEMA_VERSION)
    );

    // Nothing to back up, so no backup table should have been created.
    assert!(
        audit_columns(&store, "audit_logs_pre_v2").await.is_empty(),
        "a fresh database should not produce a backup table"
    );
}
