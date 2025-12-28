use std::net::SocketAddr;
use std::sync::Arc;
use std::env;
use tower_http::cors::{CorsLayer, Any};
use axum::http::{HeaderValue, Method};
use pangolin_store::{CatalogStore, MemoryStore, PostgresStore, MongoStore, SqliteStore};
use pangolin_api::app;
use uuid::Uuid;
use pangolin_core::model::Tenant;
use pangolin_core::user::{User, UserRole}; // Import User and UserRole
use pangolin_api::auth_middleware::{create_session, generate_token};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize store based on env
    let storage_type = std::env::var("PANGOLIN_STORAGE_TYPE").unwrap_or_else(|_| "memory".to_string());
    
    let store: Arc<dyn CatalogStore + Send + Sync> = if let Ok(db_url) = std::env::var("DATABASE_URL") {
        if db_url.starts_with("postgresql://") || db_url.starts_with("postgres://") {
            tracing::info!("Using PostgreSQL storage backend");
            Arc::new(PostgresStore::new(&db_url).await.expect("Failed to connect to PostgreSQL"))
        } else if db_url.starts_with("mongodb://") || db_url.starts_with("mongodb+srv://") {
            tracing::info!("Using MongoDB storage backend");
            let db_name = std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| "pangolin".to_string());
            Arc::new(MongoStore::new(&db_url, &db_name).await.expect("Failed to connect to MongoDB"))
        } else if db_url.starts_with("sqlite://") || db_url.ends_with(".db") {
            tracing::info!("Using SQLite storage backend");
            let store = SqliteStore::new(&db_url).await.expect("Failed to connect to SQLite");
            let schema = include_str!("../../pangolin_store/sql/sqlite_schema.sql");
            store.apply_schema(schema).await.expect("Failed to apply schema");
            Arc::new(store)
        } else {
            tracing::warn!("Unknown DATABASE_URL format, falling back to Memory Storage");
            Arc::new(MemoryStore::new())
        }
    } else {
        tracing::info!("Using Memory Storage");
        Arc::new(MemoryStore::new())
    };
    
    // Wrap with Caching Layer (P3 Optimization)
    let store = Arc::new(pangolin_api::cached_store::CachedCatalogStore::new(store));

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


    // Check for NO_AUTH mode and auto-provision initial Tenant Admin
    let no_auth_enabled = std::env::var("PANGOLIN_NO_AUTH")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
        
    let seed_admin_enabled = std::env::var("PANGOLIN_SEED_ADMIN")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if no_auth_enabled || seed_admin_enabled {
        let admin_username = std::env::var("PANGOLIN_ADMIN_USER").unwrap_or_else(|_| "tenant_admin".to_string());
        let admin_password = std::env::var("PANGOLIN_ADMIN_PASSWORD").unwrap_or_else(|_| "password123".to_string());
        let jwt_secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "default_secret_for_dev".to_string());
        
        // Hash password
        let password_hash = pangolin_api::auth_middleware::hash_password(&admin_password)
            .expect("Failed to hash default admin password");

        let mut admin_user = User::new_tenant_admin(
            admin_username.clone(),
            format!("{}@example.com", admin_username),
            password_hash,
            default_tenant_id // Create inside default tenant
        );
        // Force ID to match auth_middleware's default NO_AUTH session ID (Uuid::nil())
        admin_user.id = Uuid::nil();

        // Try to create the user
        let user_id = admin_user.id; // Copy ID before moving
        match store.create_user(admin_user.clone()).await {
            Ok(_) => tracing::info!("Auto-provisioned initial Tenant Admin: {}", admin_username),
            Err(_) => tracing::debug!("Tenant Admin '{}' already exists", admin_username),
        }

        // Generate a long-lived token for display
        let session = create_session(
            user_id,
            admin_username.clone(),
            Some(default_tenant_id),
            UserRole::TenantAdmin,
            31536000, // 365 days
        );
        let token = generate_token(session, &jwt_secret).expect("Failed to generate startup token");

        // PRINT THE BANNER
        println!("\n========================================================");
        println!(" WARNING: NO_AUTH MODE ENABLED - FOR EVALUATION ONLY");
        println!("========================================================");
        println!("Initial Tenant Admin Auto-Provisioned:");
        println!(" Username: {}", admin_username);
        println!(" Password: {}", admin_password);
        println!(" Tenant ID: {}", default_tenant_id);
        println!("--------------------------------------------------------");
        println!("PyIceberg Configuration Snippet:");
        println!("--------------------------------------------------------");
        println!("catalog = load_catalog(");
        println!("    \"local\",");
        println!("    **{{");
        println!("        \"type\": \"rest\",");
        println!("        \"uri\": \"http://127.0.0.1:8080/api/v1/catalogs/my_catalog/iceberg\",");
        println!("        \"token\": \"{}\",", token);
        println!("        \"header.X-Pangolin-Tenant\": \"{}\"", default_tenant_id);
        println!("    }}");
        println!(")");
        println!("========================================================\n");
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
    // Run it
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
