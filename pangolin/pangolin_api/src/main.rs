use std::net::SocketAddr;
use std::sync::Arc;
use pangolin_store::memory::MemoryStore;
use pangolin_store::CatalogStore;
use pangolin_api::app;

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
    let app = app(store);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
