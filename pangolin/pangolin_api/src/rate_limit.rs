//! Throttling for the authentication endpoints.
//!
//! C-5. Before this, the login endpoint was brute-forceable: there were global
//! concurrency and body limits and a request timeout, but nothing that made the
//! thousandth password guess from one address any more expensive than the
//! first. Bcrypt makes each attempt slow, which raises the cost of a large
//! campaign but does nothing against a targeted guess at one weak password, and
//! it makes the endpoint an efficient way to burn the server's CPU.
//!
//! Two keys, deliberately:
//!
//! * **by source address**: bounds one attacker hammering many accounts;
//! * **by account**: bounds many sources hammering one account, which is what
//!   a credential-stuffing list looks like and which a per-IP limit alone
//!   cannot see.
//!
//! A fixed window rather than a token bucket. It is coarser, but the failure
//! mode of a fixed window under attack is that it lets through at most twice
//! the configured rate across a window boundary, whereas the failure mode of a
//! badly tuned bucket is a burst allowance that hands an attacker exactly the
//! sustained-guess budget the limit exists to remove.
//!
//! ## What this is not
//!
//! In-process, so the limit is **per replica**: with N replicas the effective
//! limit is N times the configured one. That is a real weakness and it is
//! stated in the operations documentation rather than hidden. A shared limiter
//! needs Redis or equivalent, which this project does not otherwise require;
//! adding a mandatory external dependency to the login path is a bigger
//! decision than this fix.

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use moka::future::Cache;
use serde_json::json;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

/// The paths this applies to.
///
/// Only the endpoints that accept a credential. Rate-limiting the whole API
/// would be a different feature with different tuning, and applying an
/// auth-shaped limit to ordinary catalog traffic would break normal use.
const THROTTLED_PATHS: &[&str] = &[
    "/api/v1/users/login",
    "/api/v1/tokens",
    "/api/v1/auth/oauth/callback",
];

pub fn is_throttled_path(path: &str) -> bool {
    THROTTLED_PATHS.contains(&path)
}

#[derive(Clone)]
pub struct RateLimiter {
    counters: Cache<String, u32>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            // Entries expire a window after they are created, which is what
            // makes this a fixed window: the count for a key lives exactly as
            // long as the window it belongs to.
            counters: Cache::builder()
                .time_to_live(window)
                .max_capacity(100_000)
                .build(),
            limit,
            window,
        }
    }

    /// Record an attempt against `key`. `Err(retry_after)` when over the limit.
    pub async fn check(&self, key: &str) -> Result<(), Duration> {
        if self.limit == 0 {
            return Ok(()); // disabled
        }
        let current = self.counters.get(key).await.unwrap_or(0);
        if current >= self.limit {
            return Err(self.window);
        }
        self.counters.insert(key.to_string(), current + 1).await;
        Ok(())
    }

    /// Forget a key. Called after a *successful* authentication so that a user
    /// who mistypes a password several times and then gets it right is not
    /// still carrying the failures.
    pub async fn clear(&self, key: &str) {
        self.counters.invalidate(key).await;
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }
}

/// Resolve the client address.
///
/// `X-Forwarded-For` is only honoured when the operator has said they run
/// behind a proxy. Trusting it unconditionally would make the limit trivially
/// bypassable - an attacker sets the header to a fresh value per request and
/// every attempt looks like a new client, which is worse than no limit at all
/// because it looks like protection.
pub fn client_key(headers: &HeaderMap, peer: Option<SocketAddr>, trust_forwarded: bool) -> String {
    if trust_forwarded {
        if let Some(forwarded) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            // Left-most entry is the originating client as recorded by the
            // first proxy.
            if let Some(first) = forwarded.split(',').next() {
                let candidate = first.trim();
                if let Ok(ip) = candidate.parse::<IpAddr>() {
                    return ip.to_string();
                }
            }
        }
    }
    peer.map(|s| s.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn too_many(retry_after: Duration) -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", retry_after.as_secs().to_string())],
        Json(json!({
            "error": {
                "message": "too many authentication attempts; try again later",
                "type": "TooManyRequests",
                "code": 429
            }
        })),
    )
        .into_response()
}

/// The limiter the login handler uses for its per-account half.
///
/// A `RwLock` rather than a `OnceLock` so each `app_with_options` installs its
/// own: several tests build an app in the same process, and a limiter shared
/// across them would make one test's login attempts throttle another's.
static AUTH_LIMITER: std::sync::RwLock<Option<Arc<RateLimiter>>> = std::sync::RwLock::new(None);

pub fn set_auth_limiter(limiter: Arc<RateLimiter>) {
    if let Ok(mut guard) = AUTH_LIMITER.write() {
        *guard = Some(limiter);
    }
}

pub fn auth_limiter() -> Option<Arc<RateLimiter>> {
    AUTH_LIMITER.read().ok().and_then(|g| g.clone())
}

/// Throttle key for one account. Tenant-scoped, because the same username can
/// exist in two tenants and they are different accounts.
pub fn account_key(username: &str, tenant: Option<uuid::Uuid>) -> String {
    match tenant {
        Some(t) => format!("acct:{t}:{username}"),
        None => format!("acct:-:{username}"),
    }
}

#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Arc<RateLimiter>,
    pub trust_forwarded: bool,
}

/// Per-source-address throttling for the authentication endpoints.
///
/// The per-account half lives in the login handler, which is the only place
/// that knows which account is being attempted - reading the body here would
/// mean buffering and re-attaching it for every request on these routes.
pub async fn throttle_auth(
    State(state): State<RateLimitState>,
    // Optional: `ConnectInfo` only exists when the app is served through
    // `into_make_service_with_connect_info`, which `main` does but an
    // in-process test calling the router directly does not. A required
    // extractor here would turn every such request into a 500.
    peer: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !is_throttled_path(request.uri().path()) {
        return next.run(request).await;
    }

    let key = format!(
        "ip:{}",
        client_key(
            request.headers(),
            peer.map(|ConnectInfo(addr)| addr),
            state.trust_forwarded
        )
    );

    if let Err(retry_after) = state.limiter.check(&key).await {
        tracing::warn!(
            path = request.uri().path(),
            "authentication attempts throttled for a source address"
        );
        crate::metrics::record_auth_throttled();
        return too_many(retry_after);
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[tokio::test]
    async fn allows_up_to_the_limit_then_refuses() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        for i in 0..3 {
            assert!(limiter.check("k").await.is_ok(), "attempt {i} should pass");
        }
        assert!(
            limiter.check("k").await.is_err(),
            "the fourth attempt must be refused"
        );
    }

    #[tokio::test]
    async fn keys_are_independent() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("a").await.is_ok());
        assert!(limiter.check("a").await.is_err());
        assert!(
            limiter.check("b").await.is_ok(),
            "one key's exhaustion must not affect another"
        );
    }

    #[tokio::test]
    async fn a_successful_login_clears_the_count() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("u").await.is_ok());
        assert!(limiter.check("u").await.is_ok());
        assert!(limiter.check("u").await.is_err());

        limiter.clear("u").await;
        assert!(
            limiter.check("u").await.is_ok(),
            "clearing after a success must reset the window"
        );
    }

    #[tokio::test]
    async fn the_window_expires() {
        let limiter = RateLimiter::new(1, Duration::from_millis(120));
        assert!(limiter.check("t").await.is_ok());
        assert!(limiter.check("t").await.is_err());
        tokio::time::sleep(Duration::from_millis(250)).await;
        // moka expires lazily; a get after the TTL sees nothing.
        limiter.counters.run_pending_tasks().await;
        assert!(
            limiter.check("t").await.is_ok(),
            "the counter must not outlive its window"
        );
    }

    #[tokio::test]
    async fn a_zero_limit_disables_throttling() {
        let limiter = RateLimiter::new(0, Duration::from_secs(60));
        for _ in 0..100 {
            assert!(limiter.check("k").await.is_ok());
        }
    }

    #[test]
    fn forwarded_for_is_ignored_unless_the_operator_trusts_it() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.9"));
        let peer: SocketAddr = "198.51.100.1:5000".parse().unwrap();

        assert_eq!(
            client_key(&headers, Some(peer), false),
            "198.51.100.1",
            "an untrusted X-Forwarded-For must not select the key, or the limit \
             is bypassable by setting a header"
        );
        assert_eq!(client_key(&headers, Some(peer), true), "203.0.113.9");
    }

    #[test]
    fn a_malformed_forwarded_for_falls_back_to_the_peer() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("not-an-ip"));
        let peer: SocketAddr = "198.51.100.1:5000".parse().unwrap();
        assert_eq!(client_key(&headers, Some(peer), true), "198.51.100.1");
    }

    #[test]
    fn only_the_credential_endpoints_are_throttled() {
        assert!(is_throttled_path("/api/v1/users/login"));
        assert!(is_throttled_path("/api/v1/tokens"));
        assert!(!is_throttled_path("/api/v1/catalogs"));
        assert!(!is_throttled_path("/health/ready"));
    }
}
