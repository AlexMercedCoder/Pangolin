//! Permission matrix: who is allowed to call what.
//!
//! Roadmap improvement #0, the highest-leverage addition in the August audit.
//!
//! Every finding in the B0a-B0m cluster was invisible to CI. The code compiled,
//! it was formatted, it was lint-clean, and the tests passed - because nothing
//! asserted *who is allowed to call what*. A handler that simply forgot to call
//! `check_permission` looks identical, to every existing check, to one that
//! calls it correctly. `POST /api/v1/tokens` minting a `Root` JWT for any
//! authenticated caller sat there through a full security release.
//!
//! This module drives each sensitive route as several principals and asserts
//! the outcome for each. A missing authorization check fails here as a `200`
//! where a `403` was expected, which is the only signal that distinguishes the
//! two shapes of handler from the outside.
//!
//! Deliberately end-to-end through the real router: a unit test of
//! `check_permission` proves the function works, not that the handler calls it,
//! and "the handler does not call it" is the entire bug class.

use crate::tests_common::EnvGuard;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use pangolin_store::memory::MemoryStore;
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn app() -> Router {
    let store = Arc::new(MemoryStore::new());
    crate::app(store)
}

/// The principals every route is driven as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Principal {
    /// Global superuser.
    Root,
    /// Administrator of the tenant that owns the resource.
    TenantAdmin,
    /// Ordinary member of the tenant with no explicit grants.
    TenantUser,
    /// Administrator of a *different* tenant.
    ForeignTenantAdmin,
}

struct Fixture {
    app: Router,
    root_token: String,
    admin_token: String,
    user_token: String,
    foreign_admin_token: String,
    tenant_id: Uuid,
    foreign_tenant_id: Uuid,
}

impl Fixture {
    fn token(&self, who: Principal) -> &str {
        match who {
            Principal::Root => &self.root_token,
            Principal::TenantAdmin => &self.admin_token,
            Principal::TenantUser => &self.user_token,
            Principal::ForeignTenantAdmin => &self.foreign_admin_token,
        }
    }

    fn tenant_header(&self, who: Principal) -> Uuid {
        match who {
            Principal::ForeignTenantAdmin => self.foreign_tenant_id,
            _ => self.tenant_id,
        }
    }

    async fn request(
        &self,
        who: Principal,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> StatusCode {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("Authorization", format!("Bearer {}", self.token(who)))
            .header("Content-Type", "application/json")
            .header("X-Pangolin-Tenant", self.tenant_header(who).to_string());

        // Root has no tenant of its own, so it addresses one explicitly.
        if who == Principal::Root {
            builder = builder.header("X-Pangolin-Tenant", self.tenant_id.to_string());
        }

        let body = match body {
            Some(v) => Body::from(v.to_string()),
            None => Body::empty(),
        };

        self.app
            .clone()
            .oneshot(builder.body(body).unwrap())
            .await
            .unwrap()
            .status()
    }
}

async fn login(app: &Router, username: &str, password: &str, tenant_id: Option<Uuid>) -> String {
    let mut body = json!({ "username": username, "password": password });
    if let Some(tid) = tenant_id {
        body.as_object_mut()
            .unwrap()
            .insert("tenant-id".to_string(), json!(tid));
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users/login")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "login for {username} should succeed"
    );
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["token"].as_str().unwrap().to_string()
}

async fn create_tenant(app: &Router, root_token: &str, name: &str) -> Uuid {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Authorization", format!("Bearer {root_token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "name": name }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    Uuid::parse_str(json["id"].as_str().unwrap()).unwrap()
}

async fn create_user(
    app: &Router,
    token: &str,
    tenant_id: Uuid,
    username: &str,
    password: &str,
    role: &str,
) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("Authorization", format!("Bearer {token}"))
                .header("X-Pangolin-Tenant", tenant_id.to_string())
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": password,
                        "email": format!("{username}@example.test"),
                        "role": role,
                        "tenant_id": tenant_id,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "creating {username} should succeed"
    );
}

/// Two tenants, each with an admin, plus one ordinary user with no grants.
async fn fixture() -> Fixture {
    let app = app();
    let root_token = login(&app, "admin", "password", None).await;

    let tenant_id = create_tenant(&app, &root_token, "matrix-tenant").await;
    let foreign_tenant_id = create_tenant(&app, &root_token, "matrix-other-tenant").await;

    // Root creates the tenant admins; root is deliberately *not* allowed to
    // create tenant users (it has no tenant context of its own), so the admin
    // creates those.
    create_user(
        &app,
        &root_token,
        tenant_id,
        "matrix_admin",
        "password",
        "tenant-admin",
    )
    .await;
    create_user(
        &app,
        &root_token,
        foreign_tenant_id,
        "matrix_foreign_admin",
        "password",
        "tenant-admin",
    )
    .await;

    let admin_token = login(&app, "matrix_admin", "password", Some(tenant_id)).await;
    create_user(
        &app,
        &admin_token,
        tenant_id,
        "matrix_user",
        "password",
        "tenant-user",
    )
    .await;

    Fixture {
        user_token: login(&app, "matrix_user", "password", Some(tenant_id)).await,
        admin_token,
        foreign_admin_token: login(
            &app,
            "matrix_foreign_admin",
            "password",
            Some(foreign_tenant_id),
        )
        .await,
        app,
        root_token,
        tenant_id,
        foreign_tenant_id,
    }
}

fn root_env() -> (EnvGuard, EnvGuard, EnvGuard) {
    (
        EnvGuard::new("PANGOLIN_ROOT_USER", "admin"),
        EnvGuard::new("PANGOLIN_ROOT_PASSWORD", "password"),
        EnvGuard::new(
            "PANGOLIN_JWT_SECRET",
            "authz-matrix-test-secret-of-adequate-length-000000",
        ),
    )
}

/// **B0a.** Token minting is the single most dangerous endpoint in the API: a
/// `Root` token is a global bypass, because `check_permission` short-circuits
/// for `Root`. This endpoint took no session at all, so any authenticated
/// principal - down to a `TenantUser` - could mint one for any tenant.
#[tokio::test]
#[serial]
async fn token_minting_is_not_open_to_everyone() {
    let _env = root_env();
    let f = fixture().await;

    // A TenantUser must not be able to mint anything at all.
    let status = f
        .request(
            Principal::TenantUser,
            "POST",
            "/api/v1/tokens",
            Some(json!({ "tenant_id": f.tenant_id, "roles": ["Root"] })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a TenantUser must not be able to mint a Root token (B0a)"
    );

    // Nor a token for itself.
    let status = f
        .request(
            Principal::TenantUser,
            "POST",
            "/api/v1/tokens",
            Some(json!({ "tenant_id": f.tenant_id })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a TenantUser must not be able to mint tokens (B0a)"
    );

    // A TenantAdmin may mint within its own tenant, but not above its rank.
    let status = f
        .request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/tokens",
            Some(json!({ "tenant_id": f.tenant_id, "roles": ["Root"] })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a TenantAdmin must not be able to mint a Root token (B0a)"
    );

    // ...and not for somebody else's tenant.
    let status = f
        .request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/tokens",
            Some(json!({ "tenant_id": f.foreign_tenant_id, "roles": ["tenant-admin"] })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a TenantAdmin must not be able to mint a token for another tenant (B0a)"
    );

    // The legitimate case still works.
    let status = f
        .request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/tokens",
            Some(json!({ "tenant_id": f.tenant_id, "roles": ["tenant-user"] })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a TenantAdmin must still be able to mint a lesser token for its own tenant"
    );
}

/// **B0m.** `expires_in_hours` is caller-controlled and used to reach
/// `chrono::Duration::hours` unclamped, which panics on a large enough value -
/// aborting the connection task, since nothing caught it.
#[tokio::test]
#[serial]
async fn an_absurd_token_lifetime_does_not_take_the_server_down() {
    let _env = root_env();
    let f = fixture().await;

    let status = f
        .request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/tokens",
            Some(json!({
                "tenant_id": f.tenant_id,
                "roles": ["tenant-user"],
                "expires_in_hours": u64::MAX,
            })),
        )
        .await;

    // Clamped rather than panicking; either a clamp (200) or a refusal (400) is
    // acceptable, a dropped connection is not.
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
        "an absurd expires_in_hours must be handled, not panic (B0m); got {status}"
    );
}

/// **Improvement #0.** Request structs reject fields they do not have.
///
/// This is what turns client drift from a silent no-op into an error. Before
/// it, a CLI sending `warehouse` instead of `warehouse_name` got a `201` for a
/// catalog created without the warehouse it named.
#[tokio::test]
#[serial]
async fn unknown_request_fields_are_rejected() {
    let _env = root_env();
    let f = fixture().await;

    let status = f
        .request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/catalogs",
            Some(json!({ "name": "drift", "warehouse": "wh", "type": "pangea" })),
        )
        .await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a request naming fields the server does not have must be refused, not \
         silently emptied (improvement #0)"
    );
}

/// **B0e.** View creation and reading had no authorization at all. A view's
/// `properties["sql"]` is its whole definition.
#[tokio::test]
#[serial]
async fn view_endpoints_require_permission() {
    let _env = root_env();
    let f = fixture().await;

    // Setup: an admin creates the catalog and namespace.
    assert_eq!(
        f.request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/warehouses",
            Some(json!({
                "name": "wh",
                "use_sts": false,
                "storage_config": { "type": "filesystem", "root": "/tmp/pangolin-authz" }
            })),
        )
        .await,
        StatusCode::CREATED
    );
    assert_eq!(
        f.request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/catalogs",
            Some(json!({ "name": "cat", "warehouse_name": "wh", "catalog_type": "Local" })),
        )
        .await,
        StatusCode::CREATED
    );
    assert_eq!(
        f.request(
            Principal::TenantAdmin,
            "POST",
            "/v1/cat/namespaces",
            Some(json!({ "namespace": ["ns"] })),
        )
        .await,
        StatusCode::OK
    );

    // A TenantUser with no grants must not be able to create a view.
    let status = f
        .request(
            Principal::TenantUser,
            "POST",
            "/v1/cat/namespaces/ns/views",
            Some(json!({ "name": "v", "sql": "SELECT 1" })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "creating a view must require Create on the namespace (B0e)"
    );
}

/// **B0f.** Maintenance expires snapshots and deletes orphan files. It had no
/// session and no check, and ran against a hardcoded `"default"` catalog.
#[tokio::test]
#[serial]
async fn maintenance_requires_permission() {
    let _env = root_env();
    let f = fixture().await;

    let status = f
        .request(
            Principal::TenantUser,
            "POST",
            "/v1/cat/namespaces/ns/tables/t/maintenance",
            Some(json!({ "job_type": "expire_snapshots" })),
        )
        .await;

    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "destructive maintenance must never be reachable by an ungranted user \
         (B0f); got {status}"
    );
    assert_ne!(
        status,
        StatusCode::OK,
        "maintenance ran for a user with no grants (B0f)"
    );
}

/// **B0b.** Credential vending hands out real cloud-storage credentials. It
/// performed no authorization, never looked the table up, and hardcoded
/// read+write.
#[tokio::test]
#[serial]
async fn credential_vending_requires_permission() {
    let _env = root_env();
    let f = fixture().await;

    let status = f
        .request(
            Principal::TenantUser,
            "GET",
            "/v1/cat/namespaces/ns/tables/does_not_exist/credentials",
            None,
        )
        .await;

    assert_ne!(
        status,
        StatusCode::OK,
        "credentials were vended for a table that does not exist, to a user \
         with no grants (B0b)"
    );
}

/// **B0i.** A tenant admin must not reach another tenant's resources, even
/// though `check_permission` short-circuits on the role.
#[tokio::test]
#[serial]
async fn a_tenant_admin_cannot_reach_another_tenants_catalogs() {
    let _env = root_env();
    let f = fixture().await;

    // Tenant A's admin creates a catalog.
    assert_eq!(
        f.request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/warehouses",
            Some(json!({
                "name": "wh",
                "use_sts": false,
                "storage_config": { "type": "filesystem", "root": "/tmp/pangolin-authz-2" }
            })),
        )
        .await,
        StatusCode::CREATED
    );
    assert_eq!(
        f.request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/catalogs",
            Some(json!({ "name": "private", "warehouse_name": "wh", "catalog_type": "Local" })),
        )
        .await,
        StatusCode::CREATED
    );

    // Tenant B's admin must not see it. Their requests are scoped to their own
    // tenant, so the catalog must simply not be there.
    let status = f
        .request(
            Principal::ForeignTenantAdmin,
            "GET",
            "/api/v1/catalogs/private",
            None,
        )
        .await;

    assert_ne!(
        status,
        StatusCode::OK,
        "a tenant admin reached another tenant's catalog (B0i)"
    );
}

/// **B0j.** Logout must actually revoke: the token has to stop working.
///
/// It used to revoke `session.user_id`, which no token carries as its `jti`, so
/// the revocation check could never match and logout was cosmetic.
#[tokio::test]
#[serial]
async fn logout_revokes_the_presented_token() {
    let _env = root_env();
    let f = fixture().await;

    // The token works before logout.
    assert_eq!(
        f.request(Principal::TenantAdmin, "GET", "/api/v1/catalogs", None)
            .await,
        StatusCode::OK
    );

    assert_eq!(
        f.request(
            Principal::TenantAdmin,
            "POST",
            "/api/v1/auth/revoke",
            Some(json!({ "reason": "logout" })),
        )
        .await,
        StatusCode::OK
    );

    // And must not afterwards.
    let status = f
        .request(Principal::TenantAdmin, "GET", "/api/v1/catalogs", None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "a revoked token kept working after logout (B0j)"
    );
}

/// **B0k.** The OAuth code-exchange endpoint must be reachable without a
/// bearer token: it is the endpoint whose job is to issue the first one.
#[tokio::test]
#[serial]
async fn the_oauth_exchange_endpoint_is_reachable_unauthenticated() {
    let _env = root_env();
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/oauth/exchange")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "code": "nonexistent" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "the OAuth code-exchange endpoint demanded the token it exists to \
         issue, making the login flow unreachable (B0k)"
    );
}

/// Unauthenticated requests are refused across the board.
///
/// The floor under everything above: if this regresses, the per-principal
/// assertions stop meaning anything.
#[tokio::test]
#[serial]
async fn protected_routes_reject_anonymous_callers() {
    let _env = root_env();
    let app = app();

    for (method, uri) in [
        ("GET", "/api/v1/catalogs"),
        ("POST", "/api/v1/catalogs"),
        ("GET", "/api/v1/warehouses"),
        ("POST", "/api/v1/tokens"),
        ("GET", "/api/v1/users"),
        ("GET", "/api/v1/tenants"),
        ("POST", "/api/v1/permissions"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} served an anonymous caller"
        );
    }
}
