use axum::Router;
use std::sync::Arc;
use pangolin_store::CatalogStore;
use pangolin_store::memory::MemoryStore;
use axum::routing::{get, post, delete};

pub mod iceberg_handlers;
pub mod pangolin_handlers;
pub mod tenant_handlers;
pub mod warehouse_handlers;
pub mod asset_handlers;
pub mod auth;
pub mod signing_handlers;
pub mod token_handlers;

pub fn app(store: Arc<dyn CatalogStore + Send + Sync>) -> Router {
    Router::new()
        .route("/v1/config", get(get_config))
        .route("/v1/:prefix/config", get(get_config))
        .route("/v1/:prefix/namespaces", get(iceberg_handlers::list_namespaces).post(iceberg_handlers::create_namespace))
        .route("/v1/:prefix/namespaces/:namespace", delete(iceberg_handlers::delete_namespace))
        .route("/v1/:prefix/namespaces/:namespace/properties", post(iceberg_handlers::update_namespace_properties))
        .route("/v1/:prefix/namespaces/:namespace/tables", get(iceberg_handlers::list_tables).post(iceberg_handlers::create_table))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table", get(iceberg_handlers::load_table).post(iceberg_handlers::update_table).delete(iceberg_handlers::delete_table).head(iceberg_handlers::table_exists))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/maintenance", post(iceberg_handlers::perform_maintenance))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/metrics", post(iceberg_handlers::report_metrics))
        .route("/v1/:prefix/tables/rename", post(iceberg_handlers::rename_table))
        // Pangolin Extended APIs
        .route("/api/v1/branches", get(pangolin_handlers::list_branches).post(pangolin_handlers::create_branch))
        .route("/api/v1/branches/merge", post(pangolin_handlers::merge_branch))
        .route("/api/v1/branches/:name", get(pangolin_handlers::get_branch))
        .route("/api/v1/branches/:name/commits", get(pangolin_handlers::list_commits))
        // Tag Management
        .route("/api/v1/tags", get(pangolin_handlers::list_tags).post(pangolin_handlers::create_tag))
        .route("/api/v1/tags/:name", delete(pangolin_handlers::delete_tag))
        // Audit Logs
        .route("/api/v1/audit", get(pangolin_handlers::list_audit_events))
        // Tenant Management
        .route("/api/v1/tenants", get(tenant_handlers::list_tenants).post(tenant_handlers::create_tenant))
        .route("/api/v1/tenants/:id", get(tenant_handlers::get_tenant))
        // Warehouse Management
        .route("/api/v1/warehouses", get(warehouse_handlers::list_warehouses).post(warehouse_handlers::create_warehouse))
        .route("/api/v1/warehouses/:name", get(warehouse_handlers::get_warehouse))
        // Catalog Management
        .route("/api/v1/catalogs", get(pangolin_handlers::list_catalogs).post(pangolin_handlers::create_catalog))
        .route("/api/v1/catalogs/:name", get(pangolin_handlers::get_catalog))
        // Asset Management (Views)
        .route("/v1/:prefix/namespaces/:namespace/views", post(asset_handlers::create_view))
        .route("/v1/:prefix/namespaces/:namespace/views/:view", get(asset_handlers::get_view))
        // Signing APIs
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/credentials", post(signing_handlers::get_table_credentials))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/presign", get(signing_handlers::get_presigned_url))
        // Token Generation
        .route("/api/v1/tokens", post(token_handlers::generate_token))
        .layer(axum::middleware::from_fn(auth::auth_middleware))
        .with_state(store)
}

async fn get_config() -> &'static str {
    "{\"defaults\": {}, \"overrides\": {}}"
}
