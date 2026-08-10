//! Cross-backend parity suite.
//!
//! Roadmap improvement #1. Nearly half the storage-layer findings in the
//! August audit (B1-B7, B17-B30) were one backend silently diverging from the
//! other three: a tenant filter dropped on Mongo, a CAS skipped in memory, an
//! enum round-tripping to the wrong variant on all the SQL backends, pagination
//! that repeats or skips rows, four different answers to the same search.
//!
//! Each of those is invisible to a per-backend test, because per-backend tests
//! assert what that backend does. What catches them is asserting that all four
//! backends do the *same* thing. Every function here runs against whichever
//! `CatalogStore` it is handed, and `tests/store_integration.rs` runs the whole
//! set against memory, SQLite, Postgres and Mongo.
//!
//! Each assertion names the finding it locks down, so a regression points
//! straight at what it broke.

use crate::CatalogStore;
use pangolin_core::business_metadata::BusinessMetadata;
use pangolin_core::model::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Build the tenant -> warehouse -> catalog -> namespace chain the SQL backends'
/// foreign keys require, and return the tenant id.
async fn seed_hierarchy<S: CatalogStore>(store: &S, catalog: &str, namespace: &[String]) -> Uuid {
    let tenant_id = Uuid::new_v4();

    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("parity_{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    let _ = store
        .create_warehouse(
            tenant_id,
            Warehouse {
                id: Uuid::new_v4(),
                name: "wh".to_string(),
                tenant_id,
                storage_config: HashMap::new(),
                use_sts: false,
                vending_strategy: None,
            },
        )
        .await;

    store
        .create_catalog(
            tenant_id,
            Catalog {
                id: Uuid::new_v4(),
                name: catalog.to_string(),
                catalog_type: CatalogType::Local,
                warehouse_name: Some("wh".to_string()),
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
            catalog,
            Namespace {
                name: namespace.to_vec(),
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create namespace");

    tenant_id
}

fn asset(name: &str, kind: AssetType) -> Asset {
    Asset {
        id: Uuid::new_v4(),
        name: name.to_string(),
        kind,
        location: format!("s3://bucket/{name}"),
        properties: HashMap::new(),
    }
}

/// **B7.** Every `AssetType` variant must survive a write/read round trip.
///
/// All three persistent backends stored the Debug spelling and parsed only
/// `IcebergTable`/`View`, defaulting the other 15 variants to `IcebergTable` -
/// so a `DeltaTable` came back as an Iceberg table with no error anywhere.
pub async fn asset_types_round_trip<S: CatalogStore>(store: &S) {
    let catalog = "types";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    for kind in AssetType::all() {
        let name = format!("asset_{}", kind.as_stored_str().to_lowercase());
        let written = asset(&name, kind.clone());

        store
            .create_asset(tenant_id, catalog, None, namespace.clone(), written.clone())
            .await
            .expect("create asset");

        let read = store
            .get_asset(tenant_id, catalog, None, namespace.clone(), name.clone())
            .await
            .expect("get asset")
            .unwrap_or_else(|| panic!("asset {name} vanished"));

        assert_eq!(
            read.kind, kind,
            "asset type {kind:?} did not round-trip (B7); it came back as {:?}",
            read.kind
        );
    }
}

/// **B27.** Two pages must cover the set exactly once.
///
/// Every paginated query outside three Postgres call sites ran `LIMIT/OFFSET`
/// with no `ORDER BY`, and the memory backend paged over DashMap iteration
/// order. Both can repeat or skip a row between pages, which is invisible until
/// a client silently misses data.
pub async fn pagination_covers_the_set_exactly_once<S: CatalogStore>(store: &S) {
    let catalog = "paging";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    const TOTAL: usize = 7;
    const PAGE: usize = 3;

    for i in 0..TOTAL {
        store
            .create_asset(
                tenant_id,
                catalog,
                None,
                namespace.clone(),
                // Zero-padded so lexical and numeric order agree, and the
                // assertion is about paging rather than about collation.
                asset(&format!("t{i:03}"), AssetType::IcebergTable),
            )
            .await
            .expect("create asset");
    }

    let mut seen: Vec<String> = Vec::new();
    let mut offset = 0;
    loop {
        let page = store
            .list_assets(
                tenant_id,
                catalog,
                None,
                namespace.clone(),
                Some(crate::PaginationParams {
                    limit: Some(PAGE),
                    offset: Some(offset),
                }),
            )
            .await
            .expect("list assets");

        if page.is_empty() {
            break;
        }
        seen.extend(page.iter().map(|a| a.name.clone()));
        offset += PAGE;

        // Guard against a backend that ignores offset and loops forever.
        assert!(offset <= TOTAL * 4, "pagination did not terminate");
    }

    let mut unique = seen.clone();
    unique.sort();
    unique.dedup();

    assert_eq!(
        unique.len(),
        seen.len(),
        "paging returned a duplicate row (B27): {seen:?}"
    );
    assert_eq!(
        unique.len(),
        TOTAL,
        "paging skipped a row (B27): saw {} of {TOTAL}",
        unique.len()
    );
}

/// **B27.** Repeating the same listing must return the same order.
pub async fn listing_order_is_stable<S: CatalogStore>(store: &S) {
    let catalog = "stable";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    for name in ["delta", "alpha", "charlie", "bravo"] {
        store
            .create_asset(
                tenant_id,
                catalog,
                None,
                namespace.clone(),
                asset(name, AssetType::IcebergTable),
            )
            .await
            .expect("create asset");
    }

    let first: Vec<String> = store
        .list_assets(tenant_id, catalog, None, namespace.clone(), None)
        .await
        .expect("list assets")
        .into_iter()
        .map(|a| a.name)
        .collect();

    for _ in 0..3 {
        let again: Vec<String> = store
            .list_assets(tenant_id, catalog, None, namespace.clone(), None)
            .await
            .expect("list assets")
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(again, first, "listing order is not stable (B27)");
    }
}

/// **B2 / B0j.** A revoked token must read back as revoked.
///
/// On Mongo the revocation write and the revocation *check* used different
/// field names and different types, so the check could never match: revocation
/// - including logout - was a silent no-op.
pub async fn revocation_round_trips<S: CatalogStore>(store: &S) {
    let token_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    assert!(
        !store
            .is_token_revoked(token_id)
            .await
            .expect("check revocation"),
        "a fresh token must not be revoked"
    );

    store
        .revoke_token(token_id, expires_at, Some("parity test".to_string()))
        .await
        .expect("revoke token");

    assert!(
        store
            .is_token_revoked(token_id)
            .await
            .expect("check revocation"),
        "a revoked token must read back as revoked (B2)"
    );
}

/// **B1.** An audit event must not be readable from another tenant.
pub async fn audit_events_are_tenant_scoped<S: CatalogStore>(store: &S) {
    let owner = Uuid::new_v4();
    let stranger = Uuid::new_v4();

    for tenant in [owner, stranger] {
        store
            .create_tenant(Tenant {
                id: tenant,
                name: format!("audit_{tenant}"),
                properties: HashMap::new(),
            })
            .await
            .expect("create tenant");
    }

    let entry = pangolin_core::audit::AuditLogEntry::success(
        owner,
        Some(Uuid::new_v4()),
        "owner".to_string(),
        pangolin_core::audit::AuditAction::CreateCatalog,
        pangolin_core::audit::ResourceType::Catalog,
        Some(Uuid::new_v4()),
        "secret_catalog".to_string(),
    );
    let event_id = entry.id;

    store
        .log_audit_event(owner, entry)
        .await
        .expect("log audit event");

    assert!(
        store
            .get_audit_event(owner, event_id)
            .await
            .expect("get audit event")
            .is_some(),
        "the owning tenant must be able to read its own audit event"
    );

    assert!(
        store
            .get_audit_event(stranger, event_id)
            .await
            .expect("get audit event")
            .is_none(),
        "an audit event must not be readable across tenants (B1)"
    );
}

/// **B22 / B23.** Audit actions must round-trip, not collapse to a default.
///
/// SQLite persisted the Debug spelling and parsed snake_case, then swallowed the
/// mismatch with `unwrap_or(CreateCatalog)`, so nearly every multi-word action
/// was misattributed. Mongo's filters had the mirror-image problem and always
/// matched zero rows.
pub async fn audit_actions_round_trip<S: CatalogStore>(store: &S) {
    use pangolin_core::audit::{AuditAction, AuditLogEntry, ResourceType};

    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("audit_actions_{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    // Deliberately multi-word: the single-word ones happened to survive.
    let actions = [
        AuditAction::CreateBranch,
        AuditAction::DeleteNamespace,
        AuditAction::CommitTable,
    ];

    for action in &actions {
        store
            .log_audit_event(
                tenant_id,
                AuditLogEntry::success(
                    tenant_id,
                    Some(Uuid::new_v4()),
                    "auditor".to_string(),
                    action.clone(),
                    ResourceType::Table,
                    Some(Uuid::new_v4()),
                    format!("{action:?}_target"),
                ),
            )
            .await
            .expect("log audit event");
    }

    let events = store
        .list_audit_events(tenant_id, None)
        .await
        .expect("list audit events");

    for action in &actions {
        assert!(
            events.iter().any(|e| e.action == *action),
            "audit action {action:?} did not round-trip (B22); \
             the listing held {:?}",
            events.iter().map(|e| &e.action).collect::<Vec<_>>()
        );
    }
}

/// **B28.** A search term containing LIKE metacharacters must be literal.
pub async fn search_treats_wildcards_literally<S: CatalogStore>(store: &S) {
    let catalog = "search";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    for name in ["margin_100pct", "unrelated_table"] {
        store
            .create_asset(
                tenant_id,
                catalog,
                None,
                namespace.clone(),
                asset(name, AssetType::IcebergTable),
            )
            .await
            .expect("create asset");
    }

    // `%` is a LIKE wildcard. Unescaped, this matched everything.
    let hits = store
        .search_assets(tenant_id, "%", None)
        .await
        .expect("search assets");

    assert!(
        hits.is_empty(),
        "a literal '%' matched {} assets (B28); the term was treated as a wildcard",
        hits.len()
    );

    // A genuine substring still matches.
    let hits = store
        .search_assets(tenant_id, "margin", None)
        .await
        .expect("search assets");
    assert_eq!(
        hits.len(),
        1,
        "an ordinary substring search should still find its asset"
    );
}

/// **B28.** Tag filtering is ALL-match, and an empty filter is no filter.
pub async fn tag_filter_semantics_agree<S: CatalogStore>(store: &S) {
    let catalog = "tags";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    let both = asset("tagged_both", AssetType::IcebergTable);
    let one = asset("tagged_one", AssetType::IcebergTable);

    for a in [&both, &one] {
        store
            .create_asset(tenant_id, catalog, None, namespace.clone(), a.clone())
            .await
            .expect("create asset");
    }

    let mut meta_both = BusinessMetadata::new(both.id, Uuid::new_v4());
    meta_both.tags = vec!["pii".to_string(), "finance".to_string()];
    store
        .upsert_business_metadata(meta_both)
        .await
        .expect("upsert metadata");

    let mut meta_one = BusinessMetadata::new(one.id, Uuid::new_v4());
    meta_one.tags = vec!["pii".to_string()];
    store
        .upsert_business_metadata(meta_one)
        .await
        .expect("upsert metadata");

    // One tag: both assets carry it.
    let hits = store
        .search_assets(tenant_id, "tagged", Some(vec!["pii".to_string()]))
        .await
        .expect("search assets");
    assert_eq!(hits.len(), 2, "single-tag filter should match both assets");

    // Two tags: ALL-match, so only the asset carrying both.
    let hits = store
        .search_assets(
            tenant_id,
            "tagged",
            Some(vec!["pii".to_string(), "finance".to_string()]),
        )
        .await
        .expect("search assets");
    assert_eq!(
        hits.len(),
        1,
        "the tag filter must be ALL-match (B28): every requested tag has to be present"
    );

    // An empty filter is not a filter.
    let hits = store
        .search_assets(tenant_id, "tagged", Some(vec![]))
        .await
        .expect("search assets");
    assert_eq!(
        hits.len(),
        2,
        "an empty tag list must mean 'no tag filter' (B28)"
    );
}

/// **B26 / B5.** The compare-and-swap must actually compare.
///
/// Mongo ignored `expected_location` entirely, so two concurrent commits both
/// "succeeded" and one snapshot was lost; memory skipped the check whenever the
/// expectation was `None`.
pub async fn metadata_cas_rejects_a_stale_writer<S: CatalogStore>(store: &S) {
    let catalog = "cas";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    let table = "committed".to_string();
    let v1 = "s3://bucket/cas/v1.json".to_string();
    let mut a = asset(&table, AssetType::IcebergTable);
    a.location = v1.clone();
    a.properties
        .insert("metadata_location".to_string(), v1.clone());

    store
        .create_asset(tenant_id, catalog, None, namespace.clone(), a)
        .await
        .expect("create asset");

    // Writer A wins.
    let v2 = "s3://bucket/cas/v2.json".to_string();
    store
        .update_metadata_location(
            tenant_id,
            catalog,
            None,
            namespace.clone(),
            table.clone(),
            Some(v1.clone()),
            v2.clone(),
        )
        .await
        .expect("the first writer's CAS should succeed");

    // Writer B still believes the table is at v1: it must be refused.
    let v3 = "s3://bucket/cas/v3.json".to_string();
    let stale = store
        .update_metadata_location(
            tenant_id,
            catalog,
            None,
            namespace.clone(),
            table.clone(),
            Some(v1.clone()),
            v3,
        )
        .await;

    assert!(
        stale.is_err(),
        "a stale writer's CAS must fail (B5/B26); it silently overwrote the winner"
    );

    let current = store
        .get_metadata_location(tenant_id, catalog, None, namespace, table)
        .await
        .expect("get metadata location");
    assert_eq!(
        current,
        Some(v2),
        "the losing writer must not have changed the published metadata"
    );
}

/// **B6 / B30.** Deleting one tenant's data must not disturb another's.
///
/// Catalog names are per-tenant, but the memory backend's by-id asset index was
/// keyed on catalog name alone, so deleting tenant A's `sales` broke
/// `get_asset_by_id` for tenant B's unrelated `sales`.
pub async fn deleting_a_catalog_does_not_touch_another_tenant<S: CatalogStore>(store: &S) {
    let catalog = "sales";
    let namespace = vec!["ns".to_string()];

    let tenant_a = seed_hierarchy(store, catalog, &namespace).await;
    let tenant_b = seed_hierarchy(store, catalog, &namespace).await;

    let a_asset = asset("orders", AssetType::IcebergTable);
    let b_asset = asset("orders", AssetType::IcebergTable);

    store
        .create_asset(tenant_a, catalog, None, namespace.clone(), a_asset.clone())
        .await
        .expect("create asset");
    store
        .create_asset(tenant_b, catalog, None, namespace.clone(), b_asset.clone())
        .await
        .expect("create asset");

    store
        .delete_catalog(tenant_a, catalog.to_string())
        .await
        .expect("delete catalog");

    let survivor = store
        .get_asset_by_id(tenant_b, b_asset.id)
        .await
        .expect("get asset by id");

    assert!(
        survivor.is_some(),
        "deleting tenant A's catalog must not evict tenant B's identically-named \
         catalog from the by-id index (B6)"
    );
}

/// **B21.** Deleting a catalog that does not exist must destroy nothing.
pub async fn deleting_a_missing_catalog_destroys_nothing<S: CatalogStore>(store: &S) {
    let catalog = "present";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    let kept = asset("keep_me", AssetType::IcebergTable);
    store
        .create_asset(tenant_id, catalog, None, namespace.clone(), kept.clone())
        .await
        .expect("create asset");

    // The SQLite cascade used to run *before* the existence check, so this call
    // deleted every child row whose catalog_name matched and only then errored.
    let result = store.delete_catalog(tenant_id, "absent".to_string()).await;
    assert!(
        result.is_err(),
        "deleting a nonexistent catalog should be an error"
    );

    let still_there = store
        .get_asset(tenant_id, catalog, None, namespace, kept.name.clone())
        .await
        .expect("get asset");
    assert!(
        still_there.is_some(),
        "a failed delete_catalog must not have destroyed another catalog's assets (B21)"
    );
}

/// **B16h.** Namespace property removals must actually remove.
pub async fn namespace_property_replacement_removes_keys<S: CatalogStore>(store: &S) {
    let catalog = "props";
    let namespace = vec!["ns".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    let mut properties = HashMap::new();
    properties.insert("keep".to_string(), "yes".to_string());
    properties.insert("drop".to_string(), "please".to_string());

    store
        .update_namespace_properties(tenant_id, catalog, namespace.clone(), properties)
        .await
        .expect("seed properties");

    let mut remaining = HashMap::new();
    remaining.insert("keep".to_string(), "yes".to_string());

    store
        .replace_namespace_properties(tenant_id, catalog, namespace.clone(), remaining)
        .await
        .expect("replace properties");

    let ns = store
        .get_namespace(tenant_id, catalog, namespace)
        .await
        .expect("get namespace")
        .expect("namespace should exist");

    assert_eq!(ns.properties.get("keep").map(String::as_str), Some("yes"));
    assert!(
        !ns.properties.contains_key("drop"),
        "replace_namespace_properties must drop keys the caller left out (B16h)"
    );
}

/// **B17 / B19.** A multi-level namespace must be usable, not just creatable.
pub async fn nested_namespaces_are_addressable<S: CatalogStore>(store: &S) {
    let catalog = "nested";
    let namespace = vec!["outer".to_string(), "inner".to_string()];
    let tenant_id = seed_hierarchy(store, catalog, &namespace).await;

    let found = store
        .get_namespace(tenant_id, catalog, namespace.clone())
        .await
        .expect("get namespace");
    assert!(
        found.is_some(),
        "a nested namespace must be retrievable by its full path (B17)"
    );

    let mut properties = HashMap::new();
    properties.insert("level".to_string(), "two".to_string());
    store
        .update_namespace_properties(tenant_id, catalog, namespace.clone(), properties)
        .await
        .expect("a nested namespace must be updatable (B17)");

    store
        .delete_namespace(tenant_id, catalog, namespace.clone())
        .await
        .expect("a nested namespace must be deletable (B17)");

    assert!(
        store
            .get_namespace(tenant_id, catalog, namespace)
            .await
            .expect("get namespace")
            .is_none(),
        "the namespace should be gone after delete"
    );
}

/// **B24.** A federated catalog must still look federated in a *listing*.
pub async fn catalog_type_survives_a_listing<S: CatalogStore>(store: &S) {
    let tenant_id = Uuid::new_v4();
    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: format!("fed_{tenant_id}"),
            properties: HashMap::new(),
        })
        .await
        .expect("create tenant");

    store
        .create_catalog(
            tenant_id,
            Catalog {
                id: Uuid::new_v4(),
                name: "remote".to_string(),
                catalog_type: CatalogType::Federated,
                warehouse_name: None,
                storage_location: None,
                federated_config: Some(FederatedCatalogConfig {
                    properties: HashMap::new(),
                }),
                properties: HashMap::new(),
            },
        )
        .await
        .expect("create catalog");

    let listed = store
        .list_catalogs(tenant_id, None)
        .await
        .expect("list catalogs");

    let remote = listed
        .iter()
        .find(|c| c.name == "remote")
        .expect("the federated catalog should be listed");

    assert_eq!(
        remote.catalog_type,
        CatalogType::Federated,
        "list_catalogs must report the real catalog type (B24); \
         Postgres used to hardcode Local"
    );
}

/// Run the whole parity suite against one backend.
///
/// `tests/store_integration.rs` calls this for each of the four, which is what
/// turns "this backend behaves like this" into "all four behave alike".
pub async fn run_all<S: CatalogStore>(store: &S) {
    asset_types_round_trip(store).await;
    pagination_covers_the_set_exactly_once(store).await;
    listing_order_is_stable(store).await;
    revocation_round_trips(store).await;
    audit_events_are_tenant_scoped(store).await;
    audit_actions_round_trip(store).await;
    search_treats_wildcards_literally(store).await;
    tag_filter_semantics_agree(store).await;
    metadata_cas_rejects_a_stale_writer(store).await;
    deleting_a_catalog_does_not_touch_another_tenant(store).await;
    deleting_a_missing_catalog_destroys_nothing(store).await;
    namespace_property_replacement_removes_keys(store).await;
    nested_namespaces_are_addressable(store).await;
    catalog_type_survives_a_listing(store).await;
}
