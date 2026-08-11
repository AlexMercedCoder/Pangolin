//! Periodic removal of expired revocation records.
//!
//! Two things were wrong here, and the first makes the second moot.
//!
//! **It was never started.** `start_token_cleanup_job` was defined, the module
//! was declared, and nothing anywhere called it — a grep across the workspace
//! found only the definition and its own test. So the `revoked_tokens`
//! collection grew for the life of the deployment: every logout, every
//! administrative revocation, kept forever. The revocation check reads that
//! table on every authenticated request (A-28/C-14).
//!
//! **It was uncoordinated.** Every replica would have run it on the same
//! schedule. `cleanup_expired_tokens` is a `DELETE ... WHERE expires_at < now`,
//! so N replicas doing it concurrently is *correct* — the operation is
//! idempotent and the second one simply deletes nothing. What it is not is
//! free: replicas started by the same rolling deploy tick within milliseconds
//! of each other, so the database takes N identical delete scans at the same
//! instant, forever, on a schedule nobody staggered.
//!
//! The fix for that is jitter rather than leader election. Leader election
//! needs a lock table, a lease, and a story for what happens when the leader
//! dies mid-sweep; all of that is complexity spent to serialise an operation
//! that is safe to run concurrently. Spreading the replicas across the interval
//! removes the stampede, which is the actual cost.

use pangolin_store::CatalogStore;
use std::sync::Arc;
use std::time::Duration;

/// How often each replica sweeps.
pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(3600);

/// Pick a start delay somewhere inside the interval.
///
/// Without this, replicas from one deploy sweep in lockstep forever. The
/// randomness only has to be uneven, not unguessable, so this uses the process
/// id and the wall clock rather than pulling in an RNG.
fn stagger(interval: Duration) -> Duration {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = std::process::id() as u128
        ^ SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

    let millis = interval.as_millis().max(1);
    Duration::from_millis((seed % millis) as u64)
}

/// Sweep expired revocation records until cancelled.
///
/// Runs forever; spawn it. Errors are logged and the loop continues, because a
/// database blip should not permanently stop the sweep.
pub async fn start_token_cleanup_job(store: Arc<dyn CatalogStore + Send + Sync>) {
    run_with_interval(store, DEFAULT_INTERVAL).await
}

pub async fn run_with_interval(store: Arc<dyn CatalogStore + Send + Sync>, interval: Duration) {
    let delay = stagger(interval);
    tracing::info!(
        interval_secs = interval.as_secs(),
        first_sweep_in_secs = delay.as_secs(),
        "token cleanup job started"
    );
    tokio::time::sleep(delay).await;

    let mut ticker = tokio::time::interval(interval);
    // Without this, a sweep that overruns the interval makes tokio fire the
    // missed ticks back to back to catch up - turning one slow sweep into a
    // burst of them against a database that is evidently already struggling.
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;
        sweep_once(&store).await;
    }
}

/// One sweep. Separate so a test can drive it without waiting an hour.
pub async fn sweep_once(store: &Arc<dyn CatalogStore + Send + Sync>) {
    match store.cleanup_expired_tokens().await {
        Ok(0) => tracing::debug!("token cleanup: nothing expired"),
        Ok(count) => {
            tracing::info!(
                removed = count,
                "token cleanup: removed expired revocations"
            )
        }
        Err(e) => {
            // Logged, not fatal. The next tick tries again; a transient
            // database error should not stop the sweep for the life of the
            // process.
            tracing::error!(error = %e, "token cleanup failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration as ChronoDuration, Utc};
    use pangolin_store::MemoryStore;
    use uuid::Uuid;

    #[tokio::test]
    async fn a_sweep_removes_expired_revocations_and_keeps_live_ones() {
        let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;

        let expired = Uuid::new_v4();
        let past = Utc::now() - ChronoDuration::hours(1);
        store
            .revoke_token(expired, past, Some("expired".to_string()))
            .await
            .unwrap();

        let live = Uuid::new_v4();
        let future = Utc::now() + ChronoDuration::hours(24);
        store
            .revoke_token(live, future, Some("still valid".to_string()))
            .await
            .unwrap();

        sweep_once(&store).await;

        assert!(
            !store.is_token_revoked(expired).await.unwrap(),
            "an expired revocation should have been swept"
        );
        assert!(
            store.is_token_revoked(live).await.unwrap(),
            "a live revocation must survive the sweep, or logging out stops \
             working an hour later"
        );
    }

    #[tokio::test]
    async fn sweeping_twice_is_harmless() {
        // Every replica runs this. Concurrency safety is the reason jitter is
        // enough and leader election is not needed.
        let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
        let expired = Uuid::new_v4();
        store
            .revoke_token(expired, Utc::now() - ChronoDuration::hours(1), None)
            .await
            .unwrap();

        sweep_once(&store).await;
        sweep_once(&store).await;
        assert!(!store.is_token_revoked(expired).await.unwrap());
    }

    #[test]
    fn the_stagger_lands_inside_the_interval() {
        let interval = Duration::from_secs(3600);
        for _ in 0..50 {
            let delay = stagger(interval);
            assert!(
                delay < interval,
                "a stagger of {delay:?} is not inside {interval:?}; the first \
                 sweep would be skipped or doubled"
            );
        }
    }

    #[tokio::test]
    async fn the_job_actually_sweeps_when_run() {
        // The defect this module existed with: the function was never called,
        // so none of the above mattered. This drives the real loop.
        let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
        let expired = Uuid::new_v4();
        store
            .revoke_token(expired, Utc::now() - ChronoDuration::hours(1), None)
            .await
            .unwrap();

        let job_store = store.clone();
        let handle = tokio::spawn(async move {
            run_with_interval(job_store, Duration::from_millis(50)).await;
        });

        // The stagger is bounded by the interval, so this is enough for at
        // least one sweep.
        tokio::time::sleep(Duration::from_millis(400)).await;
        handle.abort();

        assert!(
            !store.is_token_revoked(expired).await.unwrap(),
            "the running job did not sweep"
        );
    }
}
