use super::bulk_ops_tests::test_bulk_ops_and_ancestry;
use crate::mongo::MongoStore;
use std::env;

#[tokio::test]
async fn test_mongo_store_bulk_ops() {
    // dotenv is not needed if env var passed in CLI

    // Only run if TEST_DATABASE_URL is set or use default
    // We assume the docker container is running as per instructions
    let Some(connection_string) = crate::test_support::mongo_url() else {
        println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
        return;
    };
    let db_name = "testdb_bulk_ops";

    let store = MongoStore::new(&connection_string, db_name)
        .await
        .expect("PANGOLIN_TEST_MONGO_URL is set but unusable");

    // Cleanup before test (drop db)
    let _ = store.db.drop().await;

    test_bulk_ops_and_ancestry(&store).await;
}
