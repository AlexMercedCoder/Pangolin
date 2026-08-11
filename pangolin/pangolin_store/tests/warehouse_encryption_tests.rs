//! No backend may write a cloud credential to storage in the clear.
//!
//! C-11. `secrets`' unit tests prove the crypto; they say nothing about whether
//! a given backend calls it. This reads what actually landed in the database,
//! through a connection of its own rather than through the store, because the
//! store is the thing under test - asking it to read back its own writes would
//! pass just as happily if `seal` were never called and `open` were a no-op.
//!
//! Each backend gets the same two assertions:
//!
//! 1. the persisted bytes do not contain the plaintext secret;
//! 2. reading through the store still yields the plaintext, so the round trip
//!    is usable.
//!
//! A backend added later without wiring `secrets` fails (1) here rather than
//! quietly storing credentials in plaintext, which is exactly how the MongoDB
//! UUID encoding defects survived for so long: no test compared what was
//! written against what was expected.
//!
//! The memory backend is deliberately absent. It keeps warehouses in a
//! `DashMap` and loses everything on restart - there is no "at rest" for it to
//! encrypt, and sealing there would cost work to protect a secret that is in
//! the same process's heap either way.

use pangolin_core::model::Warehouse;
use pangolin_store::{secrets, CatalogStore, MongoStore, PostgresStore, SqliteStore};
use serial_test::serial;
use std::collections::HashMap;
use uuid::Uuid;

const SECRET: &str = "AWS-SECRET-THAT-MUST-NOT-BE-STORED-IN-CLEAR";

/// Sets the encryption key for the duration of a test.
struct KeyGuard(Option<String>);

impl KeyGuard {
    fn set() -> Self {
        let previous = std::env::var("PANGOLIN_ENCRYPTION_KEY").ok();
        // A fixed key: the test asserts on ciphertext presence, not its value.
        // 32 bytes exactly; a shorter one is rejected by the key validator.
        std::env::set_var(
            "PANGOLIN_ENCRYPTION_KEY",
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=",
        );
        Self(previous)
    }
}

impl Drop for KeyGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => std::env::set_var("PANGOLIN_ENCRYPTION_KEY", v),
            None => std::env::remove_var("PANGOLIN_ENCRYPTION_KEY"),
        }
    }
}

fn warehouse_with_secret(tenant_id: Uuid, name: &str) -> Warehouse {
    Warehouse {
        id: Uuid::new_v4(),
        name: name.to_string(),
        tenant_id,
        storage_config: HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "customer-data".to_string()),
            ("secret_access_key".to_string(), SECRET.to_string()),
        ]),
        use_sts: false,
        vending_strategy: None,
    }
}

/// The shared assertions, given what the database actually holds.
fn assert_sealed(backend: &str, persisted: &str, read_back: &Warehouse) {
    assert!(
        !persisted.contains(SECRET),
        "{backend} wrote the credential in plaintext. What is stored: {persisted}"
    );
    assert!(
        persisted.contains("enc:v1:"),
        "{backend} stored something, but not in the sealed format: {persisted}"
    );
    assert!(
        persisted.contains("customer-data"),
        "{backend} encrypted the bucket name as well; only credentials should be \
         sealed, or the object-store factory cannot build a client"
    );
    assert_eq!(
        read_back.storage_config.get("secret_access_key").unwrap(),
        SECRET,
        "{backend} could not read its own credential back"
    );
    assert!(
        !secrets::has_plaintext_secret(&{
            let mut c = read_back.storage_config.clone();
            let _ = secrets::seal(&mut c);
            c
        }),
        "{backend}: re-sealing the round-tripped config left a plaintext secret"
    );
}

#[tokio::test]
#[serial(encryption_key_env)]
async fn sqlite_does_not_store_credentials_in_the_clear() {
    let _key = KeyGuard::set();

    let dir = std::env::temp_dir().join(format!("pangolin-enc-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("catalog.db");
    let url = format!("sqlite://{}?mode=rwc", path.display());

    let store = SqliteStore::new(&url).await.expect("open sqlite");
    store.run_migrations().await.expect("apply the schema");
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: format!("t-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");
    store
        .create_warehouse(tenant_id, warehouse_with_secret(tenant_id, "wh"))
        .await
        .expect("create warehouse");

    // Read the file through a connection of our own.
    let pool = sqlx::SqlitePool::connect(&url).await.expect("own pool");
    let persisted: String =
        sqlx::query_scalar("SELECT storage_config FROM warehouses WHERE name = ?")
            .bind("wh")
            .fetch_one(&pool)
            .await
            .expect("read the raw row");

    let read_back = store
        .get_warehouse(tenant_id, "wh".to_string())
        .await
        .expect("get warehouse")
        .expect("warehouse should exist");

    assert_sealed("sqlite", &persisted, &read_back);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
#[serial(encryption_key_env)]
async fn postgres_does_not_store_credentials_in_the_clear() {
    let Some(url) = pangolin_store::test_support::postgres_url() else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let _key = KeyGuard::set();

    let store = PostgresStore::new(&url).await.expect("open postgres");
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: format!("t-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");
    store
        .create_warehouse(tenant_id, warehouse_with_secret(tenant_id, "wh"))
        .await
        .expect("create warehouse");

    let pool = sqlx::PgPool::connect(&url).await.expect("own pool");
    let persisted: serde_json::Value = sqlx::query_scalar(
        "SELECT storage_config FROM warehouses WHERE tenant_id = $1 AND name = $2",
    )
    .bind(tenant_id)
    .bind("wh")
    .fetch_one(&pool)
    .await
    .expect("read the raw row");

    let read_back = store
        .get_warehouse(tenant_id, "wh".to_string())
        .await
        .expect("get warehouse")
        .expect("warehouse should exist");

    assert_sealed("postgres", &persisted.to_string(), &read_back);
}

#[tokio::test]
#[serial(encryption_key_env)]
async fn mongodb_does_not_store_credentials_in_the_clear() {
    let Some(url) = pangolin_store::test_support::mongo_url() else {
        println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
        return;
    };
    let _key = KeyGuard::set();

    let db_name = format!("pangolin_enc_{}", Uuid::new_v4().simple());
    let store = MongoStore::new(&url, &db_name).await.expect("open mongo");
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: format!("t-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");
    store
        .create_warehouse(tenant_id, warehouse_with_secret(tenant_id, "wh"))
        .await
        .expect("create warehouse");

    let client = mongodb::Client::with_uri_str(&url)
        .await
        .expect("own client");
    let raw: mongodb::bson::Document = client
        .database(&db_name)
        .collection("warehouses")
        .find_one(mongodb::bson::doc! { "name": "wh" })
        .await
        .expect("read the raw document")
        .expect("the document should exist");

    let read_back = store
        .get_warehouse(tenant_id, "wh".to_string())
        .await
        .expect("get warehouse")
        .expect("warehouse should exist");

    assert_sealed("mongodb", &raw.to_string(), &read_back);
    let _ = client.database(&db_name).drop().await;
}

/// Rotating a credential must not write the new one in the clear.
///
/// `create` and `update` are separate code paths in every backend, and sealing
/// only on create would mean the first credential is protected and every
/// rotation after it is not - the worst of both, because the table looks
/// encrypted.
#[tokio::test]
#[serial(encryption_key_env)]
async fn updating_a_credential_seals_the_new_value() {
    let _key = KeyGuard::set();

    let dir = std::env::temp_dir().join(format!("pangolin-enc-upd-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let url = format!("sqlite://{}?mode=rwc", dir.join("catalog.db").display());

    let store = SqliteStore::new(&url).await.expect("open sqlite");
    store.run_migrations().await.expect("apply the schema");
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(pangolin_core::model::Tenant {
            id: tenant_id,
            name: format!("t-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");
    store
        .create_warehouse(tenant_id, warehouse_with_secret(tenant_id, "wh"))
        .await
        .expect("create warehouse");

    const ROTATED: &str = "ROTATED-SECRET-ALSO-MUST-NOT-BE-CLEAR";
    store
        .update_warehouse(
            tenant_id,
            "wh".to_string(),
            pangolin_core::model::WarehouseUpdate {
                name: None,
                storage_config: Some(HashMap::from([
                    ("type".to_string(), "s3".to_string()),
                    ("bucket".to_string(), "customer-data".to_string()),
                    ("secret_access_key".to_string(), ROTATED.to_string()),
                ])),
                use_sts: None,
                vending_strategy: None,
            },
        )
        .await
        .expect("rotate the credential");

    let pool = sqlx::SqlitePool::connect(&url).await.expect("own pool");
    let persisted: String =
        sqlx::query_scalar("SELECT storage_config FROM warehouses WHERE name = ?")
            .bind("wh")
            .fetch_one(&pool)
            .await
            .expect("read the raw row");

    assert!(
        !persisted.contains(ROTATED),
        "the rotated credential was written in plaintext: {persisted}"
    );

    let read_back = store
        .get_warehouse(tenant_id, "wh".to_string())
        .await
        .expect("get warehouse")
        .expect("warehouse should exist");
    assert_eq!(
        read_back.storage_config.get("secret_access_key").unwrap(),
        ROTATED
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// A database written before encryption existed must still be readable.
///
/// Deploying this must not turn every existing warehouse into an error. The
/// read path tolerates plaintext precisely so that an upgrade is not an outage.
#[tokio::test]
#[serial(encryption_key_env)]
async fn warehouses_written_before_encryption_still_load() {
    let dir = std::env::temp_dir().join(format!("pangolin-enc-legacy-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let url = format!("sqlite://{}?mode=rwc", dir.join("catalog.db").display());

    let tenant_id = Uuid::new_v4();
    // Written with no key configured, i.e. exactly as an older release would.
    {
        std::env::remove_var("PANGOLIN_ENCRYPTION_KEY");
        let store = SqliteStore::new(&url).await.expect("open sqlite");
        store.run_migrations().await.expect("apply the schema");
        store
            .create_tenant(pangolin_core::model::Tenant {
                id: tenant_id,
                name: format!("t-{tenant_id}"),
                properties: HashMap::new(),
            })
            .await
            .expect("create tenant");
        store
            .create_warehouse(tenant_id, warehouse_with_secret(tenant_id, "legacy"))
            .await
            .expect("create warehouse");
    }

    // Now the operator turns encryption on and restarts.
    let _key = KeyGuard::set();
    let store = SqliteStore::new(&url).await.expect("reopen sqlite");
    store.run_migrations().await.expect("apply the schema");
    let read_back = store
        .get_warehouse(tenant_id, "legacy".to_string())
        .await
        .expect("get warehouse")
        .expect("warehouse should exist");

    assert_eq!(
        read_back.storage_config.get("secret_access_key").unwrap(),
        SECRET,
        "turning encryption on must not make existing warehouses unreadable"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
