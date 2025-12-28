use super::bulk_ops_tests::test_bulk_ops_and_ancestry;
use crate::mongo::MongoStore;
use std::env;

#[tokio::test]
async fn test_mongo_store_bulk_ops() {
    // dotenv is not needed if env var passed in CLI
    
    // Only run if TEST_DATABASE_URL is set or use default
    // We assume the docker container is running as per instructions
    let connection_string = env::var("MONGO_TEST_URL").unwrap_or_else(|_| "mongodb://testuser:testpass@localhost:27017".to_string());
    let db_name = "testdb_bulk_ops";
    
    let store = MongoStore::new(&connection_string, db_name).await.expect("Failed to connect to Mongo");
    
    // Cleanup before test (drop db)
    let _ = store.db.drop().await;
    
    test_bulk_ops_and_ancestry(&store).await;
}
