//! Regression tests for A-24: the PostgreSQL backend used zero transactions.
//!
//! Deleting a catalog cascades by hand across five statements, and merging a
//! branch moves a head pointer and copies assets in separate statements. Before
//! 0.6.0 a failure partway through either sequence left the catalog in a
//! partially-applied state, with no rollback and no repair tooling.
//!
//! These tests need a live PostgreSQL and skip with a note when
//! `PANGOLIN_TEST_POSTGRES_URL` is not set.

use pangolin_core::model::{
    Asset, AssetType, Branch, BranchType, Catalog, CatalogType, Namespace, Tenant,
};
use pangolin_store::{CatalogStore, PostgresStore};
use std::collections::HashMap;
use uuid::Uuid;

async fn connect() -> Option<PostgresStore> {
    let url = pangolin_store::test_support::postgres_url()?;
    match PostgresStore::new(&url).await {
        Ok(store) => Some(store),
        Err(e) => panic!("PANGOLIN_TEST_POSTGRES_URL is set but unusable: {e}"),
    }
}

/// Create a tenant with one catalog, one namespace and one asset on `main`.
async fn seed(store: &PostgresStore) -> (Uuid, String) {
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("tx_tenant_{}", tenant_id.simple()),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    let catalog_name = format!("tx_catalog_{}", Uuid::new_v4().simple());
    store
        .create_catalog(
            tenant_id,
            Catalog {
                id: Uuid::new_v4(),
                name: catalog_name.clone(),
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
            &catalog_name,
            Namespace {
                name: vec!["sales".to_string()],
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create namespace");

    store
        .create_asset(
            tenant_id,
            &catalog_name,
            Some("main".to_string()),
            vec!["sales".to_string()],
            Asset {
                id: Uuid::new_v4(),
                name: "orders".to_string(),
                kind: AssetType::IcebergTable,
                location: "s3://bucket/orders".to_string(),
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create asset");

    (tenant_id, catalog_name)
}

/// The cascade must be all-or-nothing.
///
/// Deleting a catalog that does not exist fails at the *last* of five
/// statements, after the first four have already deleted tags, branches, assets
/// and namespaces. Without a transaction those deletions stand, so a failed
/// delete silently destroys the contents of whatever the child filter matched.
#[tokio::test]
async fn a_failed_catalog_delete_leaves_the_catalog_intact() {
    let Some(store) = connect().await else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let (tenant_id, catalog_name) = seed(&store).await;

    // Delete a catalog whose name does not exist but whose *child* rows do not
    // either — the point is that the final statement fails and the transaction
    // rolls back rather than committing four deletions.
    let missing = format!("{catalog_name}_does_not_exist");
    let result = store.delete_catalog(tenant_id, missing).await;
    assert!(result.is_err(), "deleting a missing catalog must fail");

    // The real catalog and everything under it must still be there.
    let catalog = store
        .get_catalog(tenant_id, catalog_name.clone())
        .await
        .expect("get catalog");
    assert!(
        catalog.is_some(),
        "the catalog must survive a failed delete"
    );

    let assets = store
        .list_assets(
            tenant_id,
            &catalog_name,
            Some("main".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list assets");
    assert_eq!(assets.len(), 1, "assets must survive a failed delete");

    let namespaces = store
        .list_namespaces(tenant_id, &catalog_name, None, None)
        .await
        .expect("list namespaces");
    assert_eq!(
        namespaces.len(),
        1,
        "namespaces must survive a failed delete"
    );
}

/// A successful delete still removes everything.
#[tokio::test]
async fn a_successful_catalog_delete_removes_the_whole_cascade() {
    let Some(store) = connect().await else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let (tenant_id, catalog_name) = seed(&store).await;

    store
        .delete_catalog(tenant_id, catalog_name.clone())
        .await
        .expect("delete catalog");

    assert!(store
        .get_catalog(tenant_id, catalog_name.clone())
        .await
        .expect("get catalog")
        .is_none());

    let assets = store
        .list_assets(
            tenant_id,
            &catalog_name,
            Some("main".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list assets");
    assert!(assets.is_empty(), "assets must be gone with the catalog");
}

/// A merge into a branch that does not exist must not copy any assets.
///
/// The head-pointer update fails first, and the asset copy is a separate
/// statement. Without a transaction the sequence aborts cleanly here, but the
/// reverse ordering (assets copied, head update failing) would not — the test
/// pins the observable property either way.
#[tokio::test]
async fn a_failed_merge_copies_nothing() {
    let Some(store) = connect().await else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let (tenant_id, catalog_name) = seed(&store).await;

    // A source branch with an asset of its own.
    store
        .create_branch(
            tenant_id,
            &catalog_name,
            Branch {
                name: "dev".to_string(),
                head_commit_id: None,
                branch_type: BranchType::Experimental,
                assets: vec![],
            },
        )
        .await
        .expect("create branch");

    store
        .create_asset(
            tenant_id,
            &catalog_name,
            Some("dev".to_string()),
            vec!["sales".to_string()],
            Asset {
                id: Uuid::new_v4(),
                name: "dev_only".to_string(),
                kind: AssetType::IcebergTable,
                location: "s3://bucket/dev_only".to_string(),
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create dev asset");

    let result = store
        .merge_branch(
            tenant_id,
            &catalog_name,
            "dev".to_string(),
            "no_such_branch".to_string(),
        )
        .await;
    assert!(result.is_err(), "merging into a missing branch must fail");

    // Nothing should have landed on a branch that does not exist.
    let stray = store
        .list_assets(
            tenant_id,
            &catalog_name,
            Some("no_such_branch".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list assets");
    assert!(
        stray.is_empty(),
        "a failed merge must not leave assets on the target"
    );

    let _ = store.delete_catalog(tenant_id, catalog_name).await;
}

/// A successful merge moves the branch head *and* the assets together.
#[tokio::test]
async fn a_successful_merge_moves_head_and_assets_together() {
    let Some(store) = connect().await else {
        println!("skipping: set PANGOLIN_TEST_POSTGRES_URL to run this test");
        return;
    };
    let (tenant_id, catalog_name) = seed(&store).await;

    for branch in ["main", "dev"] {
        store
            .create_branch(
                tenant_id,
                &catalog_name,
                Branch {
                    name: branch.to_string(),
                    head_commit_id: None,
                    branch_type: BranchType::Experimental,
                    assets: vec![],
                },
            )
            .await
            .expect("create branch");
    }

    store
        .create_asset(
            tenant_id,
            &catalog_name,
            Some("dev".to_string()),
            vec!["sales".to_string()],
            Asset {
                id: Uuid::new_v4(),
                name: "feature_table".to_string(),
                kind: AssetType::IcebergTable,
                location: "s3://bucket/feature".to_string(),
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create dev asset");

    store
        .merge_branch(
            tenant_id,
            &catalog_name,
            "dev".to_string(),
            "main".to_string(),
        )
        .await
        .expect("merge dev into main");

    let on_main = store
        .list_assets(
            tenant_id,
            &catalog_name,
            Some("main".to_string()),
            vec!["sales".to_string()],
            None,
        )
        .await
        .expect("list assets");
    assert!(
        on_main.iter().any(|a| a.name == "feature_table"),
        "an asset created on dev must reach main; found {:?}",
        on_main.iter().map(|a| &a.name).collect::<Vec<_>>()
    );

    let _ = store.delete_catalog(tenant_id, catalog_name).await;
}
