use crate::postgres::PostgresStore;
// use crate::CatalogStore; // Not strictly needed if only using in setup, but good practice

#[cfg(test)]
mod postgres_bulk_ops_tests {
    use super::*;
    use crate::tests::bulk_ops_tests::test_bulk_ops_and_ancestry;

    /// Connect to the Postgres test database, or `None` when none is
    /// configured so the caller can skip rather than fail. There used to be a
    /// hardcoded default URL and an `.expect()`, which turned "no database
    /// running" into a wall of failures indistinguishable from real defects.
    async fn setup_postgres_store() -> Option<PostgresStore> {
        let database_url = crate::test_support::postgres_url()?;
        match PostgresStore::new(&database_url).await {
            Ok(store) => Some(store),
            Err(e) => panic!("PANGOLIN_TEST_POSTGRES_URL is set but unusable: {e}"),
        }
    }

    #[tokio::test]
    async fn test_postgres_store_bulk_ops() {
        let Some(store) = setup_postgres_store().await else {
            println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
            return;
        };
        test_bulk_ops_and_ancestry(&store).await;
    }
}
