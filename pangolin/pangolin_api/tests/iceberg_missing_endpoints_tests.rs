//! The Iceberg REST operations that had no route at all (A-5).
//!
//! `listViews`, `viewExists`, `dropView` and `registerTable` returned `404` for
//! the *route*, not for the resource — indistinguishable, from a client's point
//! of view, from a catalog that simply had no such view. Spark's `SHOW VIEWS`,
//! `DROP VIEW` and any migration from another catalog all need these.
//!
//! `commitTransaction` is deliberately still absent; see the note at the bottom
//! of this file.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use pangolin_api::app;
use pangolin_api::tests_common::EnvGuard;
use pangolin_core::model::{Catalog, CatalogType, Namespace, Tenant};
use pangolin_store::memory::MemoryStore;
use serial_test::serial;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup() -> (
    Arc<dyn pangolin_store::CatalogStore + Send + Sync>,
    EnvGuard,
) {
    let guard = EnvGuard::new("PANGOLIN_NO_AUTH", "true");
    let store = Arc::new(MemoryStore::new()) as Arc<dyn pangolin_store::CatalogStore + Send + Sync>;
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    store
        .create_tenant(Tenant {
            id: tenant_id,
            name: "t".to_string(),
            properties: HashMap::new(),
        })
        .await
        .unwrap();

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
        .unwrap();

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
        .unwrap();

    (store, guard)
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn create_view(app: &axum::Router, name: &str) -> StatusCode {
    app.clone()
        .oneshot(json_request(
            "POST",
            "/v1/cat/namespaces/sales/views",
            // `CreateViewRequest` is {name, sql, dialect?, properties?} and is
            // `deny_unknown_fields`, so a spec-shaped `schema` is rejected.
            serde_json::json!({
                "name": name,
                "sql": "SELECT 1"
            }),
        ))
        .await
        .unwrap()
        .status()
}

#[tokio::test]
#[serial]
async fn list_views_returns_what_was_created() {
    let (store, _guard) = setup().await;
    let app = app(store);

    assert!(
        create_view(&app, "daily_totals").await.is_success(),
        "the view should be creatable"
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/cat/namespaces/sales/views")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "listViews had no route at all; a client could not discover any view"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let identifiers = body
        .get("identifiers")
        .and_then(|v| v.as_array())
        .expect("the spec's response is {\"identifiers\": [...]}");
    assert!(
        identifiers
            .iter()
            .any(|i| i.get("name").and_then(|n| n.as_str()) == Some("daily_totals")),
        "the created view is missing from the listing: {body}"
    );
}

#[tokio::test]
#[serial]
async fn view_exists_answers_without_a_body() {
    let (store, _guard) = setup().await;
    let app = app(store);
    assert!(create_view(&app, "v1").await.is_success());

    let present = app
        .clone()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/v1/cat/namespaces/sales/views/v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(present.status(), StatusCode::NO_CONTENT);

    let absent = app
        .clone()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/v1/cat/namespaces/sales/views/nope")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(absent.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn drop_view_removes_it() {
    let (store, _guard) = setup().await;
    let app = app(store);
    assert!(create_view(&app, "doomed").await.is_success());

    let dropped = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/cat/namespaces/sales/views/doomed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        dropped.status(),
        StatusCode::NO_CONTENT,
        "dropView had no route; a view created through the Iceberg API could \
         never be removed through it"
    );

    let after = app
        .clone()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/v1/cat/namespaces/sales/views/doomed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        after.status(),
        StatusCode::NOT_FOUND,
        "the view survived its drop"
    );
}

/// Dropping a *table* through the view endpoint must not work.
///
/// Views and tables are both assets; without an explicit kind check, a caller
/// permitted to drop views could remove a table by addressing it as one.
#[tokio::test]
#[serial]
async fn drop_view_refuses_a_table() {
    let (store, _guard) = setup().await;

    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
    store
        .create_asset(
            tenant_id,
            "cat",
            Some("main".to_string()),
            vec!["sales".to_string()],
            pangolin_core::model::Asset {
                id: Uuid::new_v4(),
                name: "real_table".to_string(),
                kind: pangolin_core::model::AssetType::IcebergTable,
                location: "s3://bucket/real_table".to_string(),
                properties: HashMap::new(),
            },
        )
        .await
        .unwrap();

    let app = app(store.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/cat/namespaces/sales/views/real_table")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "a table must not be droppable through the view endpoint"
    );

    assert!(
        store
            .get_asset(
                tenant_id,
                "cat",
                Some("main".to_string()),
                vec!["sales".to_string()],
                "real_table".to_string()
            )
            .await
            .unwrap()
            .is_some(),
        "the table was deleted through the view endpoint"
    );
}

/// `registerTable` must reject a metadata location it cannot read.
///
/// Registering a location that is not there would leave a table whose every
/// subsequent `loadTable` fails, with the failure surfacing far from the
/// request that caused it.
#[tokio::test]
#[serial]
async fn register_table_rejects_an_unreadable_location() {
    let (store, _guard) = setup().await;
    let app = app(store);

    let response = app
        .oneshot(json_request(
            "POST",
            "/v1/cat/namespaces/sales/register",
            serde_json::json!({
                "name": "adopted",
                "metadata-location": "s3://nowhere/does-not-exist.metadata.json"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "an unreadable metadata location must be refused at registration time"
    );
}

/// The route exists at all — which is the thing A-5 was about.
///
/// Before this, `POST .../register` was a routing 404, indistinguishable to a
/// client from a catalog that had rejected the request.
#[tokio::test]
#[serial]
async fn register_table_is_routed() {
    let (store, _guard) = setup().await;
    let app = app(store);

    let response = app
        .oneshot(json_request(
            "POST",
            "/v1/cat/namespaces/sales/register",
            serde_json::json!({
                "name": "adopted",
                "metadata-location": "s3://nowhere/x.json"
            }),
        ))
        .await
        .unwrap();

    assert_ne!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "the register route is not wired"
    );
    // A 400 from the handler is a real answer; a 404 here would mean no route.
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// `commitTransaction` is still absent, on purpose.
///
/// The spec's `POST /v1/{prefix}/transactions/commit` is a *multi-table atomic*
/// commit: either every table's metadata pointer moves or none does. Pangolin's
/// commit path does compare-and-swap per table through the store trait, and
/// there is no cross-table transaction behind it.
///
/// Routing it and committing the tables one at a time would be worse than
/// leaving it unrouted. An engine that sees the endpoint will rely on the
/// atomicity the spec promises, and a partial failure would leave half a
/// multi-table change applied with no way to tell. A 404 makes the client fall
/// back to per-table commits, which is what actually happens today and is
/// honest about it.
///
/// This test pins that decision so it is a choice rather than an oversight.
#[tokio::test]
#[serial]
async fn commit_transaction_is_deliberately_absent() {
    let (store, _guard) = setup().await;
    let app = app(store);

    let response = app
        .oneshot(json_request(
            "POST",
            "/v1/cat/transactions/commit",
            serde_json::json!({ "table-changes": [] }),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "commitTransaction must stay unrouted until the store can commit \
         several tables atomically. If this test fails because someone added \
         the route, check that it is genuinely atomic across tables."
    );
}
