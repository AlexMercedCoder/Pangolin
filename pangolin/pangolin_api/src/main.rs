use std::net::SocketAddr;
use std::sync::Arc;
use std::env;
use pangolin_store::{CatalogStore, MemoryStore, S3Store, PostgresStore, MongoStore};
use pangolin_api::app;
use uuid::Uuid;
use pangolin_core::model::Tenant;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize store based on env
    let storage_type = std::env::var("PANGOLIN_STORAGE_TYPE").unwrap_or_else(|_| "memory".to_string());
    
    let store: Arc<dyn CatalogStore + Send + Sync> = if let Ok(db_url) = std::env::var("DATABASE_URL") {
        if db_url.starts_with("postgresql://") || db_url.starts_with("postgres://") {
            tracing::info!("Using Postgres Storage");
            Arc::new(PostgresStore::new(&db_url).await.expect("Failed to connect to Postgres"))
        } else if db_url.starts_with("mongodb://") || db_url.starts_with("mongodb+srv://") {
            tracing::info!("Using MongoDB Storage");
            Arc::new(MongoStore::new(&db_url).await.expect("Failed to connect to MongoDB"))
        } else if db_url.starts_with("s3://") {
            tracing::info!("Using S3 Storage");
            Arc::new(S3Store::new().expect("Failed to initialize S3 storage"))
        } else {
            tracing::warn!("Unknown DATABASE_URL format, falling back to Memory Storage");
            Arc::new(MemoryStore::new())
        }
    } else {
        tracing::info!("Using Memory Storage");
        Arc::new(MemoryStore::new())
    };

    // Create default tenant for testing/development
    // This tenant is used when no authentication is provided
    let default_tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")
        .expect("Failed to parse default tenant UUID");
    
    let default_tenant = Tenant {
        id: default_tenant_id,
        name: "default".to_string(),
        properties: std::collections::HashMap::new(),
    };
    
    // Try to create default tenant (ignore error if it already exists)
    match store.create_tenant(default_tenant.clone()).await {
        Ok(_) => tracing::info!("Created default tenant for testing: {}", default_tenant_id),
        Err(_) => tracing::debug!("Default tenant already exists"),
    }

    // Build our application with a route
    // The instruction seems to imply adding routes directly here, but the `app` function
    // from `pangolin_api` is responsible for defining routes.
    // To faithfully apply the change, we assume the user intends for the `app` function
    // to be modified to include the new route. However, since we only have this file,
    // and the instruction's snippet is syntactically incorrect for this location,
    // I will make no change to this file based on the provided snippet.
    // If the `app` function was defined in this file, I would insert the route there.
    // As it stands, the provided snippet cannot be correctly applied to this file.
    let app = app(store);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
