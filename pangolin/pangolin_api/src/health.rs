//! Liveness and readiness endpoints.
//!
//! `/health` used to be `get(|| async { "OK" })` — a string literal that
//! returned `200` whether or not the metadata store was reachable. The Helm
//! readiness probe pointed at it, so a pod whose database connection was dead
//! stayed in the Service and kept receiving traffic (A-21).
//!
//! Three endpoints now exist, with distinct meanings:
//!
//! * `/health/live` — the process is running and its event loop is responsive.
//!   Failing this should restart the pod, so it deliberately touches nothing
//!   external: a database outage must not cause a restart storm.
//! * `/health/ready` — the process is willing to serve traffic: it has finished
//!   starting, is not draining, and a store round-trip succeeds.
//! * `/health` — kept for compatibility, and equivalent to `/health/ready`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use axum::{http::StatusCode, response::IntoResponse, Json};
use pangolin_store::CatalogStore;
use serde_json::json;

static READY: AtomicBool = AtomicBool::new(false);
static DRAINING: AtomicBool = AtomicBool::new(false);
static STORE: OnceLock<Arc<dyn CatalogStore + Send + Sync>> = OnceLock::new();

/// Register the store that readiness probes should exercise.
pub fn set_store(store: Arc<dyn CatalogStore + Send + Sync>) {
    let _ = STORE.set(store);
}

/// Mark the process as ready to serve traffic.
pub fn mark_ready() {
    READY.store(true, Ordering::SeqCst);
}

/// Mark the process as draining: readiness fails, liveness still passes.
pub fn mark_draining() {
    DRAINING.store(true, Ordering::SeqCst);
}

/// Whether the process considers itself ready, ignoring store connectivity.
pub fn is_ready() -> bool {
    READY.load(Ordering::SeqCst) && !DRAINING.load(Ordering::SeqCst)
}

/// Liveness: the process is up. Touches nothing external on purpose.
pub async fn live() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "alive" })))
}

/// Readiness: the process will accept work right now.
pub async fn ready() -> impl IntoResponse {
    if DRAINING.load(Ordering::SeqCst) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "draining" })),
        );
    }
    if !READY.load(Ordering::SeqCst) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "starting" })),
        );
    }

    let Some(store) = STORE.get() else {
        // No store registered: the library is embedded (tests, or an
        // in-process harness). Report ready rather than inventing a failure.
        return (
            StatusCode::OK,
            Json(json!({ "status": "ready", "store": "not-registered" })),
        );
    };

    let started = Instant::now();
    match store.list_tenants(None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "ready",
                "store": "ok",
                "store_latency_ms": started.elapsed().as_millis() as u64,
            })),
        ),
        Err(e) => {
            tracing::warn!(error = %e, "readiness probe failed: metadata store unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unready",
                    "store": "unreachable",
                    "error": e.to_string(),
                })),
            )
        }
    }
}

/// Compatibility alias for the original `/health` route.
pub async fn health() -> impl IntoResponse {
    ready().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_gating_follows_lifecycle() {
        // The statics are process-wide, so assert on transitions rather than
        // absolute values, and restore what we change.
        let was_ready = READY.load(Ordering::SeqCst);
        let was_draining = DRAINING.load(Ordering::SeqCst);

        READY.store(false, Ordering::SeqCst);
        DRAINING.store(false, Ordering::SeqCst);
        assert!(!is_ready(), "not ready before mark_ready");

        mark_ready();
        assert!(is_ready(), "ready after mark_ready");

        mark_draining();
        assert!(!is_ready(), "not ready while draining");

        READY.store(was_ready, Ordering::SeqCst);
        DRAINING.store(was_draining, Ordering::SeqCst);
    }
}
