use pangolin_store::{
    tests::{test_asset_update_consistency, test_dashboard_stats_consistency},
    MemoryStore, MongoStore, PostgresStore, SqliteStore,
};
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn test_memory_store_regression() {
    let store = MemoryStore::new();
    test_asset_update_consistency(&store).await;
    test_dashboard_stats_consistency(&store).await;
}

#[tokio::test]
async fn test_sqlite_store_regression() {
    let temp_dir = std::env::temp_dir().join(format!("pangolin_test_sqlite_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("pangolin.db");
    let conn_str = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let store = SqliteStore::new(&conn_str)
        .await
        .expect("Failed to create sqlite store");

    // Apply Schema (Required for SQLite as it doesn't auto-migrate in new())
    let schema = include_str!("../sql/sqlite_schema.sql");
    store
        .apply_schema(schema)
        .await
        .expect("Failed to apply schema");

    test_asset_update_consistency(&store).await;
    test_dashboard_stats_consistency(&store).await;
}

#[tokio::test]
async fn test_postgres_store_regression() {
    let conn_str = env::var("POSTGRES_TEST_URL")
        .unwrap_or_else(|_| "postgres://pangolin:password@localhost:5433/pangolin".to_string());

    // Check connection capability (e.g. skip if docker not running, but user asked for regression so we assume running)
    let store = match PostgresStore::new(&conn_str).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping Postgres test: {}", e);
            return;
        }
    };

    test_asset_update_consistency(&store).await;
    test_dashboard_stats_consistency(&store).await;
}

#[tokio::test]
async fn test_mongo_store_regression() {
    // `MongoStore::new` succeeds without contacting the server, so an
    // unreachable or unauthenticated deployment used to surface as a failure
    // deep inside the assertions rather than as a skip.
    let Some(conn_str) = pangolin_store::test_support::mongo_url() else {
        println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
        return;
    };
    let db_name = format!("pangolin_test_{}", Uuid::new_v4()); // Unique DB per test run

    let store = match MongoStore::new(&conn_str, &db_name).await {
        Ok(s) => s,
        Err(e) => panic!("PANGOLIN_TEST_MONGO_URL is set but unusable: {e}"),
    };

    test_asset_update_consistency(&store).await;
    test_dashboard_stats_consistency(&store).await;
}
