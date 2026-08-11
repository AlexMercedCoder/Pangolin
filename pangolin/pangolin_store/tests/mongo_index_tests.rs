//! The indexes this backend needs must actually exist after startup.
//!
//! `indexes.rs` has unit tests over the *table* of indexes; they prove the list
//! is sensible and say nothing about whether MongoDB accepted any of it. The
//! previous code created two indexes and threw the result away with `.ok()`, so
//! "we create indexes" was true and "the indexes exist" was unverified.
//!
//! Requires a MongoDB; skips with a note when `PANGOLIN_TEST_MONGO_URL` is
//! unset.

use mongodb::bson::Document;
use pangolin_store::MongoStore;
use uuid::Uuid;

/// The index keys present on a collection, as `{"field": 1}` documents.
async fn index_keys(db: &mongodb::Database, collection: &str) -> Vec<Document> {
    let mut cursor = db
        .collection::<Document>(collection)
        .list_indexes()
        .await
        .expect("list indexes");

    let mut keys = Vec::new();
    while cursor.advance().await.expect("advance") {
        let model = cursor.deserialize_current().expect("deserialize index");
        keys.push(model.keys);
    }
    keys
}

fn has_index_on(indexes: &[Document], fields: &[&str]) -> bool {
    indexes
        .iter()
        .any(|keys| keys.len() == fields.len() && fields.iter().all(|f| keys.contains_key(*f)))
}

macro_rules! store_or_skip {
    ($db:ident) => {{
        let Some(url) = pangolin_store::test_support::mongo_url() else {
            println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
            return;
        };
        let $db = format!("pangolin_idx_{}", Uuid::new_v4().simple());
        let store = MongoStore::new(&url, &$db).await.expect("open mongo");
        let client = mongodb::Client::with_uri_str(&url)
            .await
            .expect("own client");
        (store, client.database(&$db))
    }};
}

#[tokio::test]
async fn the_authorization_hot_path_is_indexed_in_the_database() {
    let (_store, db) = store_or_skip!(name);

    // These are consulted on every authenticated request. Before this, each was
    // a collection scan.
    for (collection, fields) in [
        ("user_roles", vec!["user-id"]),
        ("permissions", vec!["user-id"]),
        ("revoked_tokens", vec!["token_id"]),
        ("service_users", vec!["api-key-hash"]),
    ] {
        let indexes = index_keys(&db, collection).await;
        assert!(
            has_index_on(&indexes, &fields),
            "{collection} has no index on {fields:?}; this is read on every \
             request. Present: {indexes:?}"
        );
    }

    let _ = db.drop().await;
}

#[tokio::test]
async fn catalog_lookups_are_indexed() {
    let (_store, db) = store_or_skip!(name);

    for collection in ["catalogs", "warehouses"] {
        let indexes = index_keys(&db, collection).await;
        assert!(
            has_index_on(&indexes, &["tenant_id", "name"]),
            "{collection} has no (tenant_id, name) index. Present: {indexes:?}"
        );
    }

    let indexes = index_keys(&db, "assets").await;
    assert!(
        has_index_on(&indexes, &["tenant_id", "catalog_name", "branch_name"]),
        "assets has no branch index; listing or copying a branch scans the \
         whole collection. Present: {indexes:?}"
    );

    let _ = db.drop().await;
}

/// The uniqueness the SQL backends get from primary keys.
///
/// Without this MongoDB holds two catalogs of the same name in one tenant and
/// returns an arbitrary one, which is a correctness difference from the other
/// three backends rather than a performance one.
#[tokio::test]
async fn duplicate_catalog_names_are_rejected() {
    let Some(url) = pangolin_store::test_support::mongo_url() else {
        println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
        return;
    };
    let db_name = format!("pangolin_idx_{}", Uuid::new_v4().simple());
    let store = MongoStore::new(&url, &db_name).await.expect("open mongo");

    use pangolin_core::model::{Catalog, CatalogType, Tenant};
    use pangolin_store::CatalogStore;
    use std::collections::HashMap;

    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("t-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    let catalog = |name: &str| Catalog {
        id: Uuid::new_v4(),
        name: name.to_string(),
        catalog_type: CatalogType::Local,
        warehouse_name: None,
        storage_location: None,
        federated_config: None,
        properties: HashMap::new(),
    };

    store
        .create_catalog(tenant_id, catalog("dupe"))
        .await
        .expect("the first create should succeed");

    let second = store.create_catalog(tenant_id, catalog("dupe")).await;
    assert!(
        second.is_err(),
        "MongoDB accepted a second catalog named 'dupe' in the same tenant. \
         PostgreSQL rejects this with a primary-key violation; without the \
         unique index the two backends disagree about what is valid."
    );

    let client = mongodb::Client::with_uri_str(&url).await.expect("client");
    let _ = client.database(&db_name).drop().await;
}

/// Running twice must be a no-op, because it runs on every startup.
#[tokio::test]
async fn creating_the_indexes_is_idempotent() {
    let Some(url) = pangolin_store::test_support::mongo_url() else {
        println!("skipping: set PANGOLIN_TEST_MONGO_URL to run this test");
        return;
    };
    let db_name = format!("pangolin_idx_{}", Uuid::new_v4().simple());

    let _first = MongoStore::new(&url, &db_name).await.expect("first open");
    let client = mongodb::Client::with_uri_str(&url).await.expect("client");
    let db = client.database(&db_name);
    let before = index_keys(&db, "catalogs").await.len();

    // A second server starting against the same database, which is what a
    // rolling restart does.
    let _second = MongoStore::new(&url, &db_name).await.expect("second open");
    let after = index_keys(&db, "catalogs").await.len();

    assert_eq!(
        before, after,
        "a second startup changed the index set; this runs on every boot and \
         must be a no-op"
    );

    let _ = db.drop().await;
}
