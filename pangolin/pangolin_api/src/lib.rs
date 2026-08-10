use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Method};
use axum::routing::{delete, get, post, put};
use axum::Router;
use pangolin_store::CatalogStore;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;

pub mod asset_handlers;
pub mod auth;
pub mod cached_store;
pub mod credential_signers;
pub mod credential_vending;
pub mod iceberg;
pub mod pangolin_handlers;
pub mod signing_handlers;
pub mod tenant_handlers;
pub mod warehouse_handlers;

pub mod api_key;
pub mod audit_handlers;
#[cfg(test)]
pub mod audit_tests;
pub mod auth_middleware;
pub mod authz;
pub mod authz_utils; // Permission filtering utilities
pub mod business_metadata_handlers;
pub mod config;
pub mod conflict_detector;
pub mod federated_catalog_handlers;
pub mod federated_proxy;
pub mod health;
pub mod merge_handlers;
pub mod metrics;
pub mod oauth_handlers;
pub mod oauth_state;
pub mod observability;
pub mod public_paths;
/// Shared test fixtures.
///
/// Compiled only for tests: these helpers used to ship inside the release
/// binary because the module was declared unconditionally (B-7). Integration
/// tests under `tests/` cannot see a `#[cfg(test)]` module of the library, so
/// the gate is `test` *or* the `test-fixtures` feature, which the dev-dependency
/// on this crate enables.
#[cfg(any(test, feature = "test-fixtures"))]
pub mod tests_common;
pub mod token_handlers;
pub mod user_handlers;
#[cfg(test)]
pub mod verification_tests;

pub mod cleanup_job; // Token cleanup background job
pub mod dashboard_handlers; // Dashboard statistics
pub mod error; // Custom error types
pub mod openapi; // OpenAPI documentation
pub mod optimization_handlers;
pub mod permission_handlers; // Registered new module
pub mod service_user_handlers; // Service user management
pub mod system_config_handlers; // System configuration // Search, bulk ops, validation

/// Build the router with default (development-friendly) settings.
pub fn app(store: Arc<dyn CatalogStore + Send + Sync>) -> Router {
    app_with_options(store, RouterOptions::default())
}

/// Build the router from a validated [`config::AppConfig`].
pub fn app_with_config(
    store: Arc<dyn CatalogStore + Send + Sync>,
    config: config::AppConfig,
) -> Router {
    app_with_options(
        store,
        RouterOptions {
            body_limit_bytes: config.body_limit_bytes,
            request_timeout: config.request_timeout,
            concurrency_limit: config.concurrency_limit,
            metrics_enabled: config.metrics_enabled,
            cors_allowed_origins: config.cors_allowed_origins.clone(),
        },
    )
}

/// Router-level resource and CORS settings.
#[derive(Debug, Clone)]
pub struct RouterOptions {
    /// Maximum accepted request body size, in bytes.
    pub body_limit_bytes: usize,
    /// Deadline applied to every request.
    pub request_timeout: std::time::Duration,
    /// Maximum number of requests in flight at once.
    pub concurrency_limit: usize,
    /// Whether `/metrics` is routed.
    pub metrics_enabled: bool,
    /// Explicit CORS origins. `None` allows any origin.
    pub cors_allowed_origins: Option<Vec<String>>,
}

impl Default for RouterOptions {
    fn default() -> Self {
        Self {
            body_limit_bytes: 16 * 1024 * 1024,
            request_timeout: std::time::Duration::from_secs(30),
            concurrency_limit: 512,
            metrics_enabled: true,
            cors_allowed_origins: None,
        }
    }
}

pub fn app_with_options(
    store: Arc<dyn CatalogStore + Send + Sync>,
    options: RouterOptions,
) -> Router {
    let cors = build_cors(options.cors_allowed_origins.as_deref());

    let metrics_route = if options.metrics_enabled {
        Router::new().route("/metrics", get(metrics::metrics_handler))
    } else {
        Router::new()
    };

    Router::new()
        // Health. `/health` is kept as an alias of readiness for compatibility.
        .route("/health", get(health::health))
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .merge(metrics_route)
        // Swagger UI for API documentation
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
        // System Config
        .route(
            "/api/v1/config/settings",
            get(system_config_handlers::get_system_settings)
                .put(system_config_handlers::update_system_settings),
        )
        .route(
            "/v1/config",
            get(iceberg::config::get_iceberg_catalog_config_handler),
        )
        .route(
            "/v1/:prefix/config",
            get(iceberg::config::get_prefixed_catalog_config_handler),
        )
        .route(
            "/v1/:prefix/namespaces",
            get(iceberg::namespaces::list_namespaces).post(iceberg::namespaces::create_namespace),
        )
        .route(
            // `loadNamespaceMetadata` and `namespaceExists` were on the README's
            // "not implemented" list: a client could create a namespace and set
            // properties but never read them back, and had no cheap existence
            // probe.
            "/v1/:prefix/namespaces/:namespace",
            get(iceberg::namespaces::load_namespace_metadata)
                .head(iceberg::namespaces::namespace_exists)
                .delete(iceberg::namespaces::delete_namespace),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/properties",
            post(iceberg::namespaces::update_namespace_properties),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/tables",
            get(iceberg::tables::list_tables).post(iceberg::tables::create_table),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/tables/:table",
            get(iceberg::tables::load_table)
                .post(iceberg::tables::update_table)
                .delete(iceberg::tables::delete_table)
                .head(iceberg::tables::table_exists),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/tables/:table/maintenance",
            post(iceberg::tables::perform_maintenance),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/tables/:table/metrics",
            post(iceberg::tables::report_metrics),
        )
        .route(
            "/v1/:prefix/tables/rename",
            post(iceberg::tables::rename_table),
        )
        // PyIceberg compatibility: it might append v1/config to a path that already includes v1/prefix
        .route(
            "/v1/:prefix/v1/config",
            get(iceberg::config::get_prefixed_catalog_config_handler),
        )
        .route(
            "/v1/:prefix/v1/namespaces",
            get(iceberg::namespaces::list_namespaces).post(iceberg::namespaces::create_namespace),
        )
        .route(
            "/v1/:prefix/v1/namespaces/:namespace",
            get(iceberg::namespaces::load_namespace_metadata)
                .head(iceberg::namespaces::namespace_exists)
                .delete(iceberg::namespaces::delete_namespace),
        )
        .route(
            "/v1/:prefix/v1/namespaces/:namespace/properties",
            post(iceberg::namespaces::update_namespace_properties),
        )
        .route(
            "/v1/:prefix/v1/namespaces/:namespace/tables",
            get(iceberg::tables::list_tables).post(iceberg::tables::create_table),
        )
        .route(
            "/v1/:prefix/v1/namespaces/:namespace/tables/:table",
            get(iceberg::tables::load_table)
                .post(iceberg::tables::update_table)
                .delete(iceberg::tables::delete_table)
                .head(iceberg::tables::table_exists),
        )
        .route(
            "/v1/:prefix/v1/namespaces/:namespace/tables/:table/maintenance",
            post(iceberg::tables::perform_maintenance),
        )
        .route(
            "/v1/:prefix/v1/namespaces/:namespace/tables/:table/metrics",
            post(iceberg::tables::report_metrics),
        )
        .route(
            "/v1/:prefix/v1/tables/rename",
            post(iceberg::tables::rename_table),
        )
        .route(
            "/v1/:prefix/v1/oauth/tokens",
            post(iceberg::oauth::handle_oauth_token),
        )
        .route(
            "/v1/:prefix/oauth/tokens",
            post(iceberg::oauth::handle_oauth_token),
        ) // Support non-nested v1 too just in case
        // Support /api/v1/iceberg prefix style
        .route(
            "/api/v1/iceberg/:prefix/v1/oauth/tokens",
            post(iceberg::oauth::handle_oauth_token),
        )
        .route(
            "/api/v1/iceberg/:prefix/oauth/tokens",
            post(iceberg::oauth::handle_oauth_token),
        )
        // Pangolin Extended APIs
        // Branch Operations
        .route(
            "/api/v1/branches",
            post(pangolin_handlers::create_branch).get(pangolin_handlers::list_branches),
        )
        .route(
            "/api/v1/branches/merge",
            post(pangolin_handlers::merge_branch),
        )
        .route(
            "/api/v1/branches/:name/rebase",
            post(pangolin_handlers::rebase_branch),
        ) // Rebase endpoint
        .route(
            // B33: the UI's branch-delete control has been 404ing because only
            // GET was registered here.
            "/api/v1/branches/:name",
            get(pangolin_handlers::get_branch).delete(pangolin_handlers::delete_branch),
        )
        .route(
            "/api/v1/branches/:name/commits",
            get(pangolin_handlers::list_commits),
        )
        // Merge Operations
        .route(
            "/api/v1/catalogs/:catalog_name/merge-operations",
            get(merge_handlers::list_merge_operations),
        )
        .route(
            "/api/v1/merge-operations/:operation_id",
            get(merge_handlers::get_merge_operation),
        )
        .route(
            "/api/v1/merge-operations/:operation_id/conflicts",
            get(merge_handlers::list_merge_conflicts),
        )
        .route(
            "/api/v1/merge-operations/:operation_id/complete",
            post(merge_handlers::complete_merge),
        )
        .route(
            "/api/v1/merge-operations/:operation_id/abort",
            post(merge_handlers::abort_merge),
        )
        .route(
            "/api/v1/conflicts/:conflict_id/resolve",
            post(merge_handlers::resolve_conflict),
        )
        // Tag Operations
        .route(
            "/api/v1/tags",
            post(pangolin_handlers::create_tag).get(pangolin_handlers::list_tags),
        )
        .route("/api/v1/tags/:name", delete(pangolin_handlers::delete_tag))
        // Audit Logs (Enhanced with filtering)
        .route("/api/v1/audit", get(audit_handlers::list_audit_events))
        .route(
            "/api/v1/audit/count",
            get(audit_handlers::count_audit_events),
        )
        .route(
            "/api/v1/audit/:event_id",
            get(audit_handlers::get_audit_event),
        )
        // Tenant Management
        .route(
            "/api/v1/tenants",
            get(tenant_handlers::list_tenants).post(tenant_handlers::create_tenant),
        )
        .route(
            "/api/v1/tenants/:id",
            get(tenant_handlers::get_tenant)
                .put(tenant_handlers::update_tenant)
                .delete(tenant_handlers::delete_tenant),
        )
        // Warehouse Management
        .route(
            "/api/v1/warehouses",
            get(warehouse_handlers::list_warehouses).post(warehouse_handlers::create_warehouse),
        )
        .route(
            "/api/v1/warehouses/:name",
            get(warehouse_handlers::get_warehouse)
                .put(warehouse_handlers::update_warehouse)
                .delete(warehouse_handlers::delete_warehouse),
        )
        .route(
            "/api/v1/warehouses/:name/credentials",
            get(warehouse_handlers::get_warehouse_credentials),
        )
        // Catalog Management
        .route(
            "/api/v1/catalogs",
            get(pangolin_handlers::list_catalogs).post(pangolin_handlers::create_catalog),
        )
        .route(
            "/api/v1/catalogs/:name",
            get(pangolin_handlers::get_catalog)
                .put(pangolin_handlers::update_catalog)
                .delete(pangolin_handlers::delete_catalog),
        )
        .route(
            "/api/v1/catalogs/:prefix/namespaces/tree",
            get(iceberg::namespaces::list_namespaces_tree),
        )
        .route(
            "/api/v1/catalogs/:catalog_name/namespaces/:namespace/assets",
            get(asset_handlers::list_assets).post(asset_handlers::register_asset),
        )
        .route(
            "/api/v1/catalogs/:catalog_name/namespaces/:namespace/assets/:asset",
            get(asset_handlers::get_asset),
        )
        // Federated Catalog Management
        .route(
            "/api/v1/federated-catalogs",
            post(federated_catalog_handlers::create_federated_catalog)
                .get(federated_catalog_handlers::list_federated_catalogs),
        )
        .route(
            "/api/v1/federated-catalogs/:name",
            get(federated_catalog_handlers::get_federated_catalog)
                .delete(federated_catalog_handlers::delete_federated_catalog),
        )
        .route(
            "/api/v1/federated-catalogs/:name/test",
            post(federated_catalog_handlers::test_federated_connection),
        )
        .route(
            "/api/v1/federated-catalogs/:name/sync",
            post(federated_catalog_handlers::sync_federated_catalog),
        )
        .route(
            "/api/v1/federated-catalogs/:name/stats",
            get(federated_catalog_handlers::get_federated_catalog_stats),
        )
        // Asset Management (Views)
        .route(
            "/v1/:prefix/namespaces/:namespace/views",
            post(asset_handlers::create_view),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/views/:view",
            get(asset_handlers::get_view),
        )
        // Signing APIs
        .route(
            "/v1/:prefix/namespaces/:namespace/tables/:table/credentials",
            get(signing_handlers::get_table_credentials),
        )
        .route(
            "/v1/:prefix/namespaces/:namespace/tables/:table/presign",
            get(signing_handlers::get_presigned_url),
        )
        // Business Metadata
        .route(
            "/api/v1/assets/:id/metadata",
            post(business_metadata_handlers::add_business_metadata)
                .get(business_metadata_handlers::get_business_metadata)
                .delete(business_metadata_handlers::delete_business_metadata),
        )
        .route(
            "/api/v1/assets/search",
            get(business_metadata_handlers::search_assets),
        )
        .route(
            "/api/v1/assets/:id",
            get(business_metadata_handlers::get_asset_details),
        )
        .route(
            "/api/v1/assets/:id/access-requests",
            post(business_metadata_handlers::request_access),
        )
        .route(
            "/api/v1/access-requests",
            get(business_metadata_handlers::list_access_requests),
        )
        .route(
            "/api/v1/access-requests/:id",
            put(business_metadata_handlers::update_access_request)
                .get(business_metadata_handlers::get_access_request),
        )
        // User Management
        .route(
            "/api/v1/users",
            post(user_handlers::create_user).get(user_handlers::list_users),
        )
        .route(
            "/api/v1/users/:id",
            get(user_handlers::get_user)
                .put(user_handlers::update_user)
                .delete(user_handlers::delete_user),
        )
        .route("/api/v1/users/login", post(user_handlers::login))
        .route("/api/v1/app-config", get(user_handlers::get_app_config))
        .route("/api/v1/users/me", get(user_handlers::get_current_user))
        .route("/api/v1/users/logout", post(user_handlers::logout))
        // Token Revocation & Management
        .route(
            "/api/v1/auth/revoke",
            post(token_handlers::revoke_current_token),
        )
        .route(
            "/api/v1/auth/revoke/:token_id",
            post(token_handlers::revoke_token_by_id),
        )
        .route(
            "/api/v1/auth/cleanup-tokens",
            post(token_handlers::cleanup_expired_tokens),
        )
        .route(
            "/api/v1/users/me/tokens",
            get(token_handlers::list_my_tokens),
        )
        .route(
            "/api/v1/users/:user_id/tokens",
            get(token_handlers::list_user_tokens),
        )
        .route("/api/v1/tokens/rotate", post(token_handlers::rotate_token))
        .route(
            "/api/v1/tokens/:token_id",
            delete(token_handlers::delete_token),
        )
        // Role Management
        .route(
            "/api/v1/roles",
            post(permission_handlers::create_role).get(permission_handlers::list_roles),
        )
        .route(
            "/api/v1/roles/:id",
            get(permission_handlers::get_role)
                .put(permission_handlers::update_role)
                .delete(permission_handlers::delete_role),
        )
        // Permission Management
        .route(
            "/api/v1/permissions",
            post(permission_handlers::grant_permission).get(permission_handlers::list_permissions),
        )
        .route(
            "/api/v1/permissions/:id",
            delete(permission_handlers::revoke_permission),
        )
        // User Role Assignment
        .route(
            "/api/v1/users/:id/roles",
            post(permission_handlers::assign_role).get(permission_handlers::get_user_roles),
        )
        .route(
            "/api/v1/users/:id/roles/:role_id",
            delete(permission_handlers::revoke_role),
        )
        // Service User Management
        .route(
            "/api/v1/service-users",
            get(service_user_handlers::list_service_users)
                .post(service_user_handlers::create_service_user),
        )
        .route(
            "/api/v1/service-users/:id",
            get(service_user_handlers::get_service_user)
                .put(service_user_handlers::update_service_user)
                .delete(service_user_handlers::delete_service_user),
        )
        .route(
            "/api/v1/service-users/:id/rotate",
            post(service_user_handlers::rotate_api_key),
        )
        // Token Generation
        .route("/api/v1/tokens", post(token_handlers::generate_token))
        // OAuth
        .route(
            "/oauth/authorize/:provider",
            get(oauth_handlers::oauth_authorize),
        )
        .route(
            "/oauth/callback/:provider",
            get(oauth_handlers::oauth_callback),
        )
        .route(
            "/api/v1/oauth/exchange",
            post(oauth_handlers::oauth_exchange),
        )
        .route(
            // B33: the login page needs to know which providers are configured
            // *before* anyone is authenticated; without this it rendered all
            // four buttons unconditionally, several of them dead.
            "/api/v1/oauth/providers",
            get(oauth_handlers::list_oauth_providers),
        )
        // Business Metadata (by asset id)
        .route(
            "/api/v1/business-metadata/:asset_id",
            get(business_metadata_handlers::get_business_metadata)
                .delete(business_metadata_handlers::delete_business_metadata),
        )
        // Dashboard & Statistics
        .route(
            "/api/v1/dashboard/stats",
            get(dashboard_handlers::get_dashboard_stats),
        )
        .route(
            "/api/v1/catalogs/:name/summary",
            get(dashboard_handlers::get_catalog_summary),
        )
        // Search & Filter
        .route("/api/v1/search", get(optimization_handlers::unified_search))
        .route(
            "/api/v1/search/assets",
            get(optimization_handlers::search_assets_by_name),
        )
        // Bulk Operations
        .route(
            "/api/v1/bulk/assets/delete",
            post(optimization_handlers::bulk_delete_assets),
        )
        // Validation
        .route(
            "/api/v1/validate/names",
            post(optimization_handlers::validate_names),
        )
        .layer(axum::middleware::from_fn_with_state(
            {
                let s: Arc<dyn CatalogStore + Send + Sync> = store.clone();
                s
            },
            auth_middleware::auth_middleware,
        ))
        // Resource safety. There were previously no limits of any kind (A-20):
        // a single large POST could be buffered without bound and a slow
        // backend request had no deadline.
        //
        // Layer order matters, and `.layer()` applies *outermost last*, so this
        // list reads inside-out: body limit is innermost, then the concurrency
        // limiter, then the timeout, then load shedding.
        //
        // B16l: the timeout used to sit *inside* the concurrency limiter, so a
        // request queued for one of the permits had no deadline at all - the
        // 30s clock only started once it was admitted. Under sustained overload
        // the queue grew without bound and clients saw latencies far past
        // `PANGOLIN_REQUEST_TIMEOUT_SECS`. With the timeout outside, the
        // deadline covers queueing time, which is what a client's timeout budget
        // actually cares about.
        .layer(DefaultBodyLimit::max(options.body_limit_bytes))
        // GlobalConcurrencyLimitLayer rather than ConcurrencyLimitLayer: the
        // latter's service is not `Clone`, which axum requires.
        .layer(tower::limit::GlobalConcurrencyLimitLayer::new(
            options.concurrency_limit,
        ))
        .layer(tower_http::timeout::TimeoutLayer::new(
            options.request_timeout,
        ))
        // B0m: token issuance could be driven to panic by a request-controlled
        // `expires_in_hours`, and with no catch-panic layer the panic tore down
        // the whole connection task rather than failing one request. The
        // arithmetic is now total, but a panic anywhere else should still be a
        // 500 for one caller rather than a dropped connection for several.
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        // Request IDs, access logging and metrics. `tower-http` was already
        // built with the `trace` feature but `TraceLayer` was never applied.
        .layer(axum::middleware::from_fn(observability::track_request))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(store)
}

/// Build the CORS layer.
///
/// `allow_origin(Any)` was hardcoded, with the origin-specific line commented
/// out just above it (A-32). Any origin is still the default because Pangolin
/// is normally deployed behind a gateway, but it is now configurable via
/// `PANGOLIN_CORS_ALLOWED_ORIGINS`.
fn build_cors(allowed: Option<&[String]>) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::HeaderName::from_static("x-pangolin-tenant"),
            axum::http::HeaderName::from_static("x-api-key"),
        ]);

    match allowed {
        None => base.allow_origin(Any),
        Some(origins) => {
            let parsed: Vec<HeaderValue> = origins
                .iter()
                .filter_map(|o| match o.parse::<HeaderValue>() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        tracing::warn!(origin = %o, "ignoring unparseable CORS origin");
                        None
                    }
                })
                .collect();
            if parsed.is_empty() {
                base.allow_origin(Any)
            } else {
                base.allow_origin(parsed)
            }
        }
    }
}
