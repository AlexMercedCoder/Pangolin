use crate::postgres::PostgresStore;
// use crate::CatalogStore; // Not strictly needed if only using in setup, but good practice

#[cfg(test)]
mod postgres_bulk_ops_tests {
    use super::*;
    use crate::tests::bulk_ops_tests::test_bulk_ops_and_ancestry;

    async fn setup_postgres_store() -> PostgresStore {
        let database_url = std::env::var("TEST_DATABASE_URL")
            // Default to the docker container used in development if var not set
            // matches docker-compose.db-test.yml
            .unwrap_or_else(|_| "postgres://testuser:testpass@localhost:5432/testdb".to_string());
        
        PostgresStore::new(&database_url)
            .await
            .expect("Failed to create PostgresStore")
    }

    #[tokio::test]
    async fn test_postgres_store_bulk_ops() {
        let store = setup_postgres_store().await;
        test_bulk_ops_and_ancestry(&store).await;
    }
}
