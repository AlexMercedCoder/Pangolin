//! Locating the databases that backend integration tests need.
//!
//! The backend test targets used to hardcode connection strings and then
//! `.expect()` on the connection, so `cargo test` on a clean checkout produced
//! dozens of hard failures that looked like defects but were only missing
//! infrastructure (B-15). Worse, the Postgres *and* Mongo suites both read
//! `DATABASE_URL`, so the two could never be satisfied at the same time.
//!
//! Each backend now has its own variable, with `DATABASE_URL` accepted as a
//! fallback when its scheme matches. When a variable is absent the test skips
//! with a printed note instead of failing.
//!
//! ```text
//! docker compose -f docker-compose.db-test.yml up -d postgres mongo
//! export PANGOLIN_TEST_POSTGRES_URL=postgresql://testuser:testpass@localhost:5432/testdb
//! export PANGOLIN_TEST_MONGO_URL=mongodb://testuser:testpass@localhost:27017
//! cargo test -p pangolin_store
//! ```

fn env_url(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn database_url_with_scheme(prefixes: &[&str]) -> Option<String> {
    let url = env_url("DATABASE_URL")?;
    prefixes.iter().any(|p| url.starts_with(p)).then_some(url)
}

/// Connection string for the Postgres test database, if one is configured.
pub fn postgres_url() -> Option<String> {
    env_url("PANGOLIN_TEST_POSTGRES_URL")
        .or_else(|| database_url_with_scheme(&["postgres://", "postgresql://"]))
}

/// Connection string for the MongoDB test deployment, if one is configured.
pub fn mongo_url() -> Option<String> {
    env_url("PANGOLIN_TEST_MONGO_URL")
        .or_else(|| database_url_with_scheme(&["mongodb://", "mongodb+srv://"]))
}

/// Database name to use for MongoDB tests.
pub fn mongo_db_name() -> String {
    env_url("PANGOLIN_TEST_MONGO_DB").unwrap_or_else(|| "pangolin_test".to_string())
}

/// Resolve a Postgres URL or return from the calling test with a note.
///
/// ```ignore
/// let url = pangolin_store::require_postgres!();
/// ```
#[macro_export]
macro_rules! require_postgres {
    () => {
        match $crate::test_support::postgres_url() {
            Some(url) => url,
            None => {
                println!(
                    "skipping: set PANGOLIN_TEST_POSTGRES_URL (or a postgres DATABASE_URL) to run \
                     this test"
                );
                return;
            }
        }
    };
}

/// Resolve a MongoDB URL or return from the calling test with a note.
#[macro_export]
macro_rules! require_mongo {
    () => {
        match $crate::test_support::mongo_url() {
            Some(url) => url,
            None => {
                println!(
                    "skipping: set PANGOLIN_TEST_MONGO_URL (or a mongodb DATABASE_URL) to run this \
                     test"
                );
                return;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two suites must not fight over one variable.
    #[test]
    fn database_url_is_routed_by_scheme() {
        assert_eq!(database_url_with_scheme(&["postgres://"]), None_if_unset());
        // Scheme matching is what keeps a Mongo DATABASE_URL from being handed
        // to the Postgres suite and vice versa.
        assert!(!["postgres://", "postgresql://"]
            .iter()
            .any(|p| "mongodb://host/db".starts_with(p)));
        assert!(["mongodb://", "mongodb+srv://"]
            .iter()
            .any(|p| "mongodb://host/db".starts_with(p)));
    }

    /// `DATABASE_URL` may legitimately be set in the ambient environment, so
    /// this test asserts on scheme routing rather than on absence.
    #[allow(non_snake_case)]
    fn None_if_unset() -> Option<String> {
        match std::env::var("DATABASE_URL") {
            Ok(url) if url.starts_with("postgres://") => Some(url),
            _ => None,
        }
    }

    #[test]
    fn mongo_db_name_has_a_default() {
        if std::env::var("PANGOLIN_TEST_MONGO_DB").is_err() {
            assert_eq!(mongo_db_name(), "pangolin_test");
        }
    }
}
