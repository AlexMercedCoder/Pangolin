use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;

mod iceberg_handlers;
mod pangolin_handlers;

mod tenant_handlers;
mod auth;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize store based on env
    let storage_type = std::env::var("PANGOLIN_STORAGE_TYPE").unwrap_or_else(|_| "memory".to_string());
    
    let store: Arc<dyn CatalogStore + Send + Sync> = if storage_type == "s3" {
        tracing::info!("Using S3 Storage");
        let s3_store = pangolin_store::S3Store::new().expect("Failed to initialize S3 store");
        Arc::new(s3_store)
    } else {
        tracing::info!("Using Memory Storage");
        Arc::new(MemoryStore::new())
    };

    // Build our application with a route
    let app = Router::new()
        .route("/v1/config", get(get_config))
        .route("/v1/:prefix/namespaces", get(iceberg_handlers::list_namespaces).post(iceberg_handlers::create_namespace))
        .route("/v1/:prefix/namespaces/:namespace/tables", get(iceberg_handlers::list_tables).post(iceberg_handlers::create_table))
        .route("/v1/:prefix/namespaces/:namespace/tables/:table", get(iceberg_handlers::load_table).post(iceberg_handlers::update_table))
        // Pangolin Extended APIs
        .route("/api/v1/branches", get(pangolin_handlers::list_branches).post(pangolin_handlers::create_branch))
        .route("/api/v1/branches/:name", get(pangolin_handlers::get_branch))
        // Tenant Management
        .route("/api/v1/tenants", get(tenant_handlers::list_tenants).post(tenant_handlers::create_tenant))
        .route("/api/v1/tenants/:id", get(tenant_handlers::get_tenant))
        .layer(axum::middleware::from_fn(auth::auth_middleware))
        .with_state(store);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_config() -> &'static str {
    "{\"defaults\": {}, \"overrides\": {}}"
}
