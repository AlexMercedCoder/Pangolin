//! Creating a branch by copy must be all-or-nothing.
//!
//! A-24's remainder. The branch row and its copied assets used to be written by
//! independent statements, so a failure between them left a branch that existed
//! holding an arbitrary subset of its assets — with no rollback and no repair
//! tool. The API then returned `200`, because the handler logged the copy error
//! and carried on.
//!
//! The interesting assertion is the negative one: after a failed create, the
//! branch must not exist *at all*. A test that only checks the happy path would
//! pass against the old non-transactional code.

use pangolin_core::model::{
    Asset, AssetType, Branch, BranchType, Catalog, CatalogType, Namespace, Tenant,
};
use pangolin_store::{CatalogStore, PostgresStore, SqliteStore};
use std::collections::HashMap;
use uuid::Uuid;

/// A catalog with two assets on `main`, ready to branch from.
async fn seed(store: &dyn CatalogStore) -> (Uuid, String) {
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("t-{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    store
        .create_catalog(
            tenant_id,
            Catalog {
                id: Uuid::new_v4(),
                name: "cat".to_string(),
                catalog_type: CatalogType::Local,
                warehouse_name: None,
                storage_location: None,
                federated_config: None,
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create catalog");

    store
        .create_namespace(
            tenant_id,
            "cat",
            Namespace {
                name: vec!["sales".to_string()],
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create namespace");

    store
        .create_branch(
            tenant_id,
            "cat",
            Branch {
                name: "main".to_string(),
                head_commit_id: None,
                branch_type: BranchType::Experimental,
                assets: vec![],
            },
        )
        .await
        .expect("create main");

    for name in ["orders", "customers"] {
        store
            .create_asset(
                tenant_id,
                "cat",
                Some("main".to_string()),
                vec!["sales".to_string()],
                Asset {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    kind: AssetType::IcebergTable,
                    location: format!("s3://bucket/{name}"),
                    properties: HashMap::new(),
                },
            )
            .await
            .expect("create asset");
    }

    (tenant_id, "cat".to_string())
}

async fn happy_path(store: &dyn CatalogStore, backend: &str) {
    let (tenant_id, catalog) = seed(store).await;

    let copied = store
        .create_branch_with_assets(
            tenant_id,
            &catalog,
            Branch {
                name: "feature".to_string(),
                head_commit_id: None,
                branch_type: BranchType::Experimental,
                assets: vec![],
            },
            "main",
            None,
        )
        .await
        .unwrap_or_else(|e| panic!("{backend}: create_branch_with_assets failed: {e}"));

    assert_eq!(copied, 2, "{backend}: both assets should have been copied");

    let branch = store
        .get_branch(tenant_id, &catalog, "feature".to_string())
        .await
        .expect("get branch")
        .unwrap_or_else(|| panic!("{backend}: the branch should exist"));
    assert_eq!(branch.name, "feature");

    let assets = store
        .list_assets(
            tenant_id,
            &catalog,
            Some("feature".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list assets");
    assert_eq!(
        assets.len(),
        2,
        "{backend}: the new branch should carry both assets, saw {assets:?}"
    );

    // The source must be untouched - a copy, not a move.
    let source = store
        .list_assets(
            tenant_id,
            &catalog,
            Some("main".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list source assets");
    assert_eq!(
        source.len(),
        2,
        "{backend}: copying must not empty the source"
    );
}

/// The assertion that distinguishes this from the old code.
async fn rolls_back(store: &dyn CatalogStore, backend: &str) {
    let (tenant_id, catalog) = seed(store).await;

    // A malformed asset name fails after the branch row has been inserted,
    // which is exactly the window that used to leave a half-made branch.
    let result = store
        .create_branch_with_assets(
            tenant_id,
            &catalog,
            Branch {
                name: "doomed".to_string(),
                head_commit_id: None,
                branch_type: BranchType::Experimental,
                assets: vec![],
            },
            "main",
            Some(vec!["this-name-has-no-namespace".to_string()]),
        )
        .await;

    assert!(
        result.is_err(),
        "{backend}: a malformed asset name must fail rather than silently copying nothing"
    );

    let branch = store
        .get_branch(tenant_id, &catalog, "doomed".to_string())
        .await
        .expect("get branch");
    assert!(
        branch.is_none(),
        "{backend}: the branch row survived a failed create. The transaction did \
         not roll back, which is the defect this test exists for."
    );
}

/// Copying a named subset must take only those assets.
async fn selective_copy(store: &dyn CatalogStore, backend: &str) {
    let (tenant_id, catalog) = seed(store).await;

    let copied = store
        .create_branch_with_assets(
            tenant_id,
            &catalog,
            Branch {
                name: "partial".to_string(),
                head_commit_id: None,
                branch_type: BranchType::Experimental,
                assets: vec![],
            },
            "main",
            Some(vec!["sales.orders".to_string()]),
        )
        .await
        .unwrap_or_else(|e| panic!("{backend}: selective copy failed: {e}"));

    assert_eq!(copied, 1, "{backend}: only one asset was named");

    let assets = store
        .list_assets(
            tenant_id,
            &catalog,
            Some("partial".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list assets");
    assert_eq!(assets.len(), 1, "{backend}: saw {assets:?}");
    assert_eq!(assets[0].name, "orders");
}

async fn sqlite_store() -> (SqliteStore, tempdir::Guard) {
    let guard = tempdir::Guard::new();
    let url = format!(
        "sqlite://{}?mode=rwc",
        guard.path().join("catalog.db").display()
    );
    let store = SqliteStore::new(&url).await.expect("open sqlite");
    store.run_migrations().await.expect("apply the schema");
    (store, guard)
}

/// A temporary directory that removes itself, so a failing test does not leave
/// databases behind in /tmp.
mod tempdir {
    pub struct Guard(std::path::PathBuf);

    impl Guard {
        pub fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("pangolin-branch-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).expect("temp dir");
            Self(path)
        }
        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[tokio::test]
async fn sqlite_creates_a_branch_with_its_assets() {
    let (store, _guard) = sqlite_store().await;
    happy_path(&store, "sqlite").await;
}

#[tokio::test]
async fn sqlite_rolls_back_a_failed_create() {
    let (store, _guard) = sqlite_store().await;
    rolls_back(&store, "sqlite").await;
}

#[tokio::test]
async fn sqlite_copies_only_the_named_assets() {
    let (store, _guard) = sqlite_store().await;
    selective_copy(&store, "sqlite").await;
}

#[tokio::test]
async fn postgres_creates_a_branch_with_its_assets() {
    let Some(url) = pangolin_store::test_support::postgres_url() else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let store = PostgresStore::new(&url).await.expect("open postgres");
    happy_path(&store, "postgres").await;
}

#[tokio::test]
async fn postgres_rolls_back_a_failed_create() {
    let Some(url) = pangolin_store::test_support::postgres_url() else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let store = PostgresStore::new(&url).await.expect("open postgres");
    rolls_back(&store, "postgres").await;
}

#[tokio::test]
async fn postgres_copies_only_the_named_assets() {
    let Some(url) = pangolin_store::test_support::postgres_url() else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let store = PostgresStore::new(&url).await.expect("open postgres");
    selective_copy(&store, "postgres").await;
}
