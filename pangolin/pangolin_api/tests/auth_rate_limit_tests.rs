//! The login endpoint must actually refuse a brute-force attempt.
//!
//! C-5. `rate_limit`'s unit tests prove the counter behaves; they say nothing
//! about whether it is wired to anything. These drive the real router, because
//! the interesting failure mode is a limiter that exists and is never
//! consulted. That is how this endpoint came to be brute-forceable with a
//! limiter crate already in the dependency tree for something else.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use pangolin_api::tests_common::EnvGuard;
use pangolin_api::{app_with_options, RouterOptions};
use pangolin_store::memory::MemoryStore;
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use std::time::Duration;
use tower::util::ServiceExt;

/// A router with throttling on. `RouterOptions::default()` deliberately leaves
/// it off so the rest of the suite is unaffected, so this opts in explicitly.
fn throttled_app(limit: u32, window: Duration) -> axum::Router {
    let store = Arc::new(MemoryStore::new());
    app_with_options(
        store,
        RouterOptions {
            auth_rate_limit: limit,
            auth_rate_window: window,
            ..Default::default()
        },
    )
}

fn login_request(username: &str, password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/users/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "username": username, "password": password }).to_string(),
        ))
        .unwrap()
}

#[tokio::test]
#[serial]
async fn repeated_bad_passwords_are_eventually_refused() {
    let _user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _pass = EnvGuard::new(
        "PANGOLIN_ROOT_PASSWORD",
        "a-real-password-not-a-placeholder",
    );
    let app = throttled_app(3, Duration::from_secs(60));

    let mut statuses = Vec::new();
    for _ in 0..6 {
        let response = app
            .clone()
            .oneshot(login_request("admin", "wrong-password"))
            .await
            .unwrap();
        statuses.push(response.status());
    }

    assert!(
        statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
        "six guesses against a limit of three must be throttled; got {statuses:?}"
    );
    assert_eq!(
        statuses[0],
        StatusCode::UNAUTHORIZED,
        "the first guess should be answered normally, not throttled"
    );
    assert_eq!(
        statuses[5],
        StatusCode::TOO_MANY_REQUESTS,
        "by the sixth the limiter must be refusing; got {:?}",
        statuses[5]
    );
}

#[tokio::test]
#[serial]
async fn a_throttled_response_says_when_to_retry() {
    let _user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _pass = EnvGuard::new(
        "PANGOLIN_ROOT_PASSWORD",
        "a-real-password-not-a-placeholder",
    );
    let app = throttled_app(1, Duration::from_secs(60));

    let mut last = None;
    for _ in 0..4 {
        last = Some(
            app.clone()
                .oneshot(login_request("admin", "nope"))
                .await
                .unwrap(),
        );
    }

    let response = last.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(
        response.headers().contains_key("retry-after"),
        "a 429 without Retry-After leaves a well-behaved client guessing"
    );
}

#[tokio::test]
#[serial]
async fn throttling_is_per_account_across_many_source_addresses() {
    let _user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _pass = EnvGuard::new(
        "PANGOLIN_ROOT_PASSWORD",
        "a-real-password-not-a-placeholder",
    );

    // Every request arrives from a different address, so the per-address half
    // never trips and anything refused here can only have been refused by the
    // per-account key. This is the credential-stuffing shape - one target, a
    // list of proxies - and it is precisely what a per-address limit alone
    // cannot see.
    let store = Arc::new(MemoryStore::new());
    let app = app_with_options(
        store,
        RouterOptions {
            auth_rate_limit: 4,
            auth_rate_window: Duration::from_secs(60),
            trust_forwarded_for: true,
            ..Default::default()
        },
    );

    let from = |ip: &str, user: &str| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", ip)
            .body(Body::from(
                json!({ "username": user, "password": "guess" }).to_string(),
            ))
            .unwrap()
    };

    for i in 0..4 {
        let r = app
            .clone()
            .oneshot(from(&format!("203.0.113.{i}"), "victim"))
            .await
            .unwrap();
        assert_ne!(
            r.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "attempt {i} came from a fresh address and should not be throttled yet"
        );
    }

    let refused = app
        .clone()
        .oneshot(from("203.0.113.99", "victim"))
        .await
        .unwrap();
    assert_eq!(
        refused.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "a fifth guess at the same account, from an address that has never been \
         seen, must still be refused"
    );

    // A different account from another fresh address must be unaffected.
    let other = app
        .clone()
        .oneshot(from("198.51.100.7", "someone-else"))
        .await
        .unwrap();
    assert_ne!(
        other.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "throttling one account must not lock out every other account"
    );
}

#[tokio::test]
#[serial]
async fn throttling_is_per_source_address() {
    let _user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _pass = EnvGuard::new(
        "PANGOLIN_ROOT_PASSWORD",
        "a-real-password-not-a-placeholder",
    );

    // The mirror image: one address, a different account every time, so the
    // per-account half never trips and only the per-address key can refuse.
    let store = Arc::new(MemoryStore::new());
    let app = app_with_options(
        store,
        RouterOptions {
            auth_rate_limit: 3,
            auth_rate_window: Duration::from_secs(60),
            trust_forwarded_for: true,
            ..Default::default()
        },
    );

    let from_one_address = |user: &str| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/users/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "203.0.113.5")
            .body(Body::from(
                json!({ "username": user, "password": "guess" }).to_string(),
            ))
            .unwrap()
    };

    for i in 0..3 {
        let r = app
            .clone()
            .oneshot(from_one_address(&format!("user{i}")))
            .await
            .unwrap();
        assert_ne!(r.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    let refused = app
        .clone()
        .oneshot(from_one_address("yet-another-user"))
        .await
        .unwrap();
    assert_eq!(
        refused.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "one address spraying many accounts must be throttled on the address"
    );
}

#[tokio::test]
#[serial]
async fn ordinary_endpoints_are_not_throttled() {
    let app = throttled_app(1, Duration::from_secs(60));

    // Well past the limit, on a path that is not a credential endpoint.
    for i in 0..10 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "request {i} to /health/ready was throttled; an auth-shaped limit \
             must not apply to ordinary traffic"
        );
    }
}

#[tokio::test]
#[serial]
async fn throttling_is_off_unless_configured() {
    let _user = EnvGuard::new("PANGOLIN_ROOT_USER", "admin");
    let _pass = EnvGuard::new(
        "PANGOLIN_ROOT_PASSWORD",
        "a-real-password-not-a-placeholder",
    );
    // `RouterOptions::default()` has the limit at 0. That is what the rest of
    // the suite builds with, so this pins the default rather than leaving the
    // suite's behaviour to depend on it implicitly.
    let store = Arc::new(MemoryStore::new());
    let app = app_with_options(store, RouterOptions::default());

    for i in 0..25 {
        let response = app
            .clone()
            .oneshot(login_request("admin", "wrong"))
            .await
            .unwrap();
        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "attempt {i} was throttled with the limit disabled"
        );
    }
}
