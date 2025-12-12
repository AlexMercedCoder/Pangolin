use std::net::SocketAddr;
use std::sync::Arc;
use std::env;
use pangolin_store::{CatalogStore, MemoryStore, S3Store, PostgresStore, MongoStore};
use pangolin_api::app;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize store based on env
    let storage_type = std::env::var("PANGOLIN_STORAGE_TYPE").unwrap_or_else(|_| "memory".to_string());
    
    let store: Arc<dyn CatalogStore> = match storage_type.as_str() {
        "s3" => {
            tracing::info!("Using S3 Storage");
            Arc::new(S3Store::new().expect("Failed to create S3Store"))
        }
        "postgres" => {
            tracing::info!("Using Postgres Storage");
            let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set for postgres storage");
            Arc::new(PostgresStore::new(&connection_string).await.expect("Failed to create PostgresStore"))
        }
        "mongo" => {
            tracing::info!("Using Mongo Storage");
            let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set for mongo storage");
            Arc::new(MongoStore::new(&connection_string).await.expect("Failed to create MongoStore"))
        }
        _ => {
            tracing::info!("Using Memory Storage");
            Arc::new(MemoryStore::new())
        }
    };

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
