use pangolin_api::auth_middleware::{create_session, generate_token};
use pangolin_api::config::{AppConfig, LogFormat};
use pangolin_api::{app_with_config, health};
use pangolin_core::model::Tenant;
use pangolin_core::user::{User, UserRole};
use pangolin_store::{CatalogStore, MemoryStore, MongoStore, PostgresStore, SqliteStore};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

/// Exit code used for any fatal configuration or startup problem.
const EXIT_STARTUP_FAILURE: i32 = 2;

#[tokio::main]
async fn main() {
    // Configuration is resolved and validated once, before anything else runs,
    // so a misconfiguration is a clear startup error rather than a silently
    // insecure default discovered in production.
    let config = match AppConfig::from_env_strict() {
        Ok(c) => c,
        Err(e) => {
            // Logging is not up yet; this has to reach stderr directly.
            eprintln!("FATAL: invalid configuration: {e}");
            std::process::exit(EXIT_STARTUP_FAILURE);
        }
    };

    init_tracing(&config);

    if config.jwt_secret_is_ephemeral {
        tracing::warn!(
            "PANGOLIN_JWT_SECRET is not set and development mode is enabled. A random signing \
             secret was generated for this process: every session is invalidated on restart, and \
             multiple replicas will not accept each other's tokens. Do not run this way in \
             production."
        );
    }
    if config.no_auth {
        tracing::warn!(
            "PANGOLIN_NO_AUTH is enabled: every unauthenticated request is treated as a tenant \
             administrator. For evaluation only."
        );
    }

    let store = match build_store().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("FATAL: could not initialise the metadata store: {e}");
            std::process::exit(EXIT_STARTUP_FAILURE);
        }
    };

    let store = Arc::new(pangolin_api::cached_store::CachedCatalogStore::with_ttl(
        store,
        config.warehouse_cache_ttl,
    ));

    // Default tenant, used when no tenant context is supplied.
    let default_tenant_id = Uuid::nil();
    let default_tenant = Tenant {
        id: default_tenant_id,
        name: "default".to_string(),
        properties: std::collections::HashMap::new(),
    };
    match store.create_tenant(default_tenant).await {
        Ok(_) => tracing::info!(tenant = %default_tenant_id, "created default tenant"),
        Err(_) => tracing::debug!("default tenant already exists"),
    }

    if config.no_auth || env_flag("PANGOLIN_SEED_ADMIN") {
        seed_admin(&store, default_tenant_id, &config).await;
    }

    let addr: SocketAddr = match format!("{}:{}", config.bind_address, config.port).parse() {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(
                "FATAL: PANGOLIN_BIND_ADDRESS/{}:{} is not a valid socket address: {e}",
                config.bind_address,
                config.port
            );
            std::process::exit(EXIT_STARTUP_FAILURE);
        }
    };

    let shutdown_grace = config.shutdown_grace;
    let store_for_health: Arc<dyn CatalogStore + Send + Sync> = store.clone();
    let app = app_with_config(store, config.clone());

    if config.install().is_err() {
        tracing::warn!("configuration was already installed; keeping the existing one");
    }
    health::set_store(store_for_health);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("FATAL: could not bind {addr}: {e}");
            std::process::exit(EXIT_STARTUP_FAILURE);
        }
    };

    tracing::info!(%addr, "pangolin listening");
    health::mark_ready();

    // Graceful shutdown. Without this, a Kubernetes rolling update severs every
    // in-flight request; a SIGTERM between writing a table's metadata file and
    // the compare-and-swap that publishes it leaks an orphaned metadata file.
    let serve = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(shutdown_grace));

    if let Err(e) = serve.await {
        tracing::error!("server error: {e}");
        std::process::exit(1);
    }
    tracing::info!("shutdown complete");
}

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "true" | "1" | "yes" | "on"))
        .unwrap_or(false)
}

/// Install the tracing subscriber.
///
/// `tracing_subscriber::fmt::init()` was used before, with `tracing-subscriber`
/// pulled in without the `env-filter` feature — so `RUST_LOG` was silently
/// ignored, even though the Dockerfile sets it and the Helm chart documents it
/// as a tunable (A-17). Operators believed they could raise verbosity during an
/// incident and could not.
fn init_tracing(config: &AppConfig) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,pangolin_api=info"));

    match config.log_format {
        LogFormat::Json => fmt()
            .json()
            .with_env_filter(filter)
            .with_current_span(true)
            .init(),
        LogFormat::Pretty => fmt().with_env_filter(filter).init(),
    }
}

/// Select and connect the metadata backend.
///
/// `PANGOLIN_STORAGE_TYPE` used to be read into a variable that was never used,
/// while `example.env`, `.env` and `values.yaml` all documented it as *the* way
/// to choose a backend (A-30). It is now honoured: it selects the backend, and
/// `DATABASE_URL`'s scheme is used when it is not set.
async fn build_store() -> anyhow::Result<Arc<dyn CatalogStore + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL").ok().filter(|u| !u.is_empty());
    let declared = std::env::var("PANGOLIN_STORAGE_TYPE")
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty());

    let backend = match (declared.as_deref(), database_url.as_deref()) {
        (Some(t), _) => t.to_string(),
        (None, Some(url)) if url.starts_with("postgres") => "postgres".to_string(),
        (None, Some(url)) if url.starts_with("mongodb") => "mongodb".to_string(),
        (None, Some(url)) if url.starts_with("sqlite://") || url.ends_with(".db") => {
            "sqlite".to_string()
        }
        (None, Some(url)) => {
            anyhow::bail!(
                "DATABASE_URL {url:?} has an unrecognised scheme. Set PANGOLIN_STORAGE_TYPE to one \
                 of: memory, sqlite, postgres, mongodb."
            )
        }
        (None, None) => "memory".to_string(),
    };

    match backend.as_str() {
        "memory" => {
            tracing::warn!(
                "using the in-memory metadata store: all catalog metadata is lost on restart. Set \
                 DATABASE_URL for a durable backend."
            );
            Ok(Arc::new(MemoryStore::new()))
        }
        "postgres" | "postgresql" => {
            let url = database_url.ok_or_else(|| {
                anyhow::anyhow!("PANGOLIN_STORAGE_TYPE=postgres requires DATABASE_URL")
            })?;
            tracing::info!("using the PostgreSQL metadata store");
            Ok(Arc::new(PostgresStore::new(&url).await?))
        }
        "mongodb" | "mongo" => {
            let url = database_url.ok_or_else(|| {
                anyhow::anyhow!("PANGOLIN_STORAGE_TYPE=mongodb requires DATABASE_URL")
            })?;
            let db_name = std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| "pangolin".to_string());
            tracing::info!("using the MongoDB metadata store");
            Ok(Arc::new(MongoStore::new(&url, &db_name).await?))
        }
        "sqlite" => {
            let url = database_url.ok_or_else(|| {
                anyhow::anyhow!("PANGOLIN_STORAGE_TYPE=sqlite requires DATABASE_URL")
            })?;
            tracing::info!("using the SQLite metadata store");
            let store = SqliteStore::new(&url).await?;
            store.run_migrations().await?;
            Ok(Arc::new(store))
        }
        other => anyhow::bail!(
            "unknown PANGOLIN_STORAGE_TYPE {other:?}; expected one of: memory, sqlite, postgres, \
             mongodb"
        ),
    }
}

/// Provision the initial administrator when explicitly asked to.
async fn seed_admin(
    store: &Arc<pangolin_api::cached_store::CachedCatalogStore>,
    default_tenant_id: Uuid,
    config: &AppConfig,
) {
    let admin_username =
        std::env::var("PANGOLIN_ADMIN_USER").unwrap_or_else(|_| "tenant_admin".to_string());

    // No default password. `password123` used to be the fallback here, which
    // meant "seed an admin" quietly meant "seed a publicly known admin".
    let Some(admin_password) = std::env::var("PANGOLIN_ADMIN_PASSWORD")
        .ok()
        .filter(|p| !p.trim().is_empty())
    else {
        tracing::error!(
            "admin seeding is enabled but PANGOLIN_ADMIN_PASSWORD is not set. Refusing to create \
             an administrator with a default password."
        );
        std::process::exit(EXIT_STARTUP_FAILURE);
    };
    if pangolin_api::config::is_weak_secret(&admin_password) {
        tracing::error!("PANGOLIN_ADMIN_PASSWORD is a known placeholder value; choose a real one.");
        std::process::exit(EXIT_STARTUP_FAILURE);
    }

    let password_hash = match pangolin_api::auth_middleware::hash_password(&admin_password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("could not hash the administrator password: {e}");
            std::process::exit(EXIT_STARTUP_FAILURE);
        }
    };

    let mut admin_user = User::new_tenant_admin(
        admin_username.clone(),
        format!("{}@example.com", admin_username),
        password_hash,
        default_tenant_id,
    );
    // Match the identity auth_middleware assigns in NO_AUTH mode.
    admin_user.id = Uuid::nil();
    let user_id = admin_user.id;

    match store.create_user(admin_user).await {
        Ok(_) => {
            tracing::info!(user = %admin_username, "provisioned the initial tenant administrator")
        }
        Err(_) => tracing::debug!(user = %admin_username, "tenant administrator already exists"),
    }

    // A one-hour convenience token, not the 365-day one this used to print.
    // Startup output is captured by container log aggregation, so a long-lived
    // credential printed here outlives any reasonable rotation window (C-5).
    let session = create_session(
        user_id,
        admin_username.clone(),
        Some(default_tenant_id),
        UserRole::TenantAdmin,
        3600,
    );
    let token = match generate_token(session, &config.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("could not generate the startup token: {e}");
            return;
        }
    };

    println!("\n========================================================");
    println!(" Pangolin: initial tenant administrator provisioned");
    println!("========================================================");
    println!(" Username:  {}", admin_username);
    println!(" Tenant ID: {}", default_tenant_id);
    println!(" A one-hour bootstrap token follows. Use it to create a");
    println!(" real account, then discard it.");
    println!("--------------------------------------------------------");
    println!("catalog = load_catalog(");
    println!("    \"local\",");
    println!("    **{{");
    println!("        \"type\": \"rest\",");
    println!("        \"uri\": \"http://127.0.0.1:8080/api/v1/catalogs/my_catalog/iceberg\",");
    println!("        \"token\": \"{}\",", token);
    println!(
        "        \"header.X-Pangolin-Tenant\": \"{}\"",
        default_tenant_id
    );
    println!("    }}");
    println!(")");
    println!("========================================================\n");
}

/// Resolve when the process is asked to stop, then allow a drain window.
async fn shutdown_signal(grace: std::time::Duration) {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!("could not install the SIGTERM handler: {e}");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received SIGINT, draining"),
        _ = terminate => tracing::info!("received SIGTERM, draining"),
    }

    // Fail readiness immediately so the load balancer stops sending new work
    // while in-flight requests finish.
    health::mark_draining();
    tracing::info!(grace_secs = grace.as_secs(), "draining in-flight requests");
}
