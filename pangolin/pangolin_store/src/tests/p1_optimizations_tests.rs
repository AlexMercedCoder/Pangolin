#[tokio::test]
async fn test_sqlite_p1_migration_validity() {
    // This test instantiates a SqliteStore, which triggers the migration runner.
    // If the new migration file (006_add_perf_indexes.sql) has syntax errors, this will panic/fail.
    use crate::sqlite::SqliteStore;
    
    // In-memory sqlite
    let store = SqliteStore::new("sqlite::memory:").await;
    assert!(store.is_ok(), "Failed to initialize SqliteStore, likely migration error: {:?}", store.err());
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn test_postgres_p1_migration_validity() {
    // This test instantiates a PostgresStore, which triggers the migration runner.
    // If the new migration file (20251228_add_perf_indexes.sql) has syntax errors, this will panic/fail.
    use crate::postgres::PostgresStore;
    use std::env;
    
    // Use the test database if available, or skip if not mocked/configured
    if let Ok(db_url) = env::var("TEST_DATABASE_URL") {
        let store = PostgresStore::new(&db_url).await;
        assert!(store.is_ok(), "Failed to initialize PostgresStore, likely migration error: {:?}", store.err());
    } else {
        println!("Skipping Postgres migration test: TEST_DATABASE_URL not set");
    }
}
