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

pub fn app(store: Arc<dyn CatalogStore + Send + Sync>) -> Router {
    Router::new()
        .route("/v1/config", get(get_config))
        .route("/v1/:prefix/namespaces", get(iceberg_handlers::list_namespaces).post(iceberg_handlers::create_namespace))
        .route("/v1/:prefix/namespaces/:namespace", delete(iceberg_handlers::delete_namespace))
        .route("/v1/:prefix/namespaces/:namespace/properties", post(iceberg_handlers::update_namespace_properties))
        .route("/v1/:prefix/namespaces/:namespace/tables", get(iceberg_handlers::list_tables).post(iceberg_handlers::create_table))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table", get(iceberg_handlers::load_table).post(iceberg_handlers::update_table).delete(iceberg_handlers::delete_table))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table/metrics", post(iceberg_handlers::report_metrics))
        .route("/v1/:prefix/tables/rename", post(iceberg_handlers::rename_table))
        // Pangolin Extended APIs
        .route("/api/v1/branches", get(pangolin_handlers::list_branches).post(pangolin_handlers::create_branch))
        .route("/api/v1/branches/:name", get(pangolin_handlers::get_branch))
        // Tenant Management
        .route("/api/v1/tenants", get(tenant_handlers::list_tenants).post(tenant_handlers::create_tenant))
        .route("/api/v1/tenants/:id", get(tenant_handlers::get_tenant))
        // Warehouse Management
        .route("/api/v1/warehouses", get(warehouse_handlers::list_warehouses).post(warehouse_handlers::create_warehouse))
        .route("/api/v1/warehouses/:name", get(warehouse_handlers::get_warehouse))
        // Asset Management (Views)
        .route("/v1/:prefix/namespaces/:namespace/views", post(asset_handlers::create_view))
        .route("/v1/:prefix/namespaces/:namespace/views/:view", get(asset_handlers::get_view))
        .layer(axum::middleware::from_fn(auth::auth_middleware))
        .with_state(store)
}

async fn get_config() -> &'static str {
    "{\"defaults\": {}, \"overrides\": {}}"
}
