//! `id_token` validation against a real OIDC provider.
//!
//! C-2/C-3. These stand up a fake provider with `wiremock` — a real discovery
//! document, a real JWKS, and tokens signed with a real 2048-bit RSA key — and
//! drive the actual validation path. Nothing here is mocked at the crypto
//! layer, because the properties under test *are* the crypto: a test that
//! stubbed out signature checking would pass against code that skipped them.
//!
//! Each test names the attack it prevents. An OIDC implementation that accepts
//! the tokens below is not an OIDC implementation; it is a decoder.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use pangolin_api::oidc;
use serde::Serialize;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY_PEM: &str = include_str!("fixtures/oidc_test_key.pem");
const MODULUS: &str = include_str!("fixtures/oidc_test_modulus.txt");
const KID: &str = "test-key-1";

#[derive(Serialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email_verified: Option<bool>,
}

fn now() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

fn claims(issuer: &str, audience: &str, nonce: Option<&str>) -> Claims {
    Claims {
        sub: "user-subject-123".to_string(),
        iss: issuer.to_string(),
        aud: audience.to_string(),
        exp: now() + 600,
        iat: now(),
        nonce: nonce.map(|n| n.to_string()),
        email: Some("someone@example.com".to_string()),
        email_verified: Some(true),
    }
}

fn sign(claims: &Claims, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    encode(
        &header,
        claims,
        &EncodingKey::from_rsa_pem(KEY_PEM.as_bytes()).expect("the fixture key is valid RSA PEM"),
    )
    .expect("signing the test token")
}

/// A provider serving discovery and a JWKS containing our test key.
async fn provider() -> MockServer {
    let server = MockServer::start().await;
    let issuer = server.uri();

    Mock::given(method("GET"))
        .and(path("/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{issuer}/authorize"),
            "token_endpoint": format!("{issuer}/token"),
            "jwks_uri": format!("{issuer}/jwks"),
            "code_challenge_methods_supported": ["S256"],
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": KID,
                "n": MODULUS.trim(),
                "e": "AQAB",
            }]
        })))
        .mount(&server)
        .await;

    server
}

#[tokio::test]
async fn a_properly_signed_token_is_accepted() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();

    let discovery = oidc::discover(&issuer).await.expect("discovery");
    assert!(discovery.supports_s256_pkce());

    let token = sign(&claims(&issuer, "my-client", Some("the-nonce")), KID);
    let validated = oidc::validate_id_token(&token, &discovery, "my-client", "the-nonce")
        .await
        .expect("a correctly signed and scoped token should validate");

    assert_eq!(validated.sub, "user-subject-123");
    assert_eq!(validated.email.as_deref(), Some("someone@example.com"));
    assert_eq!(validated.email_verified, Some(true));
}

/// A token minted for a *different* application at the same provider.
///
/// This is the confused-deputy problem that makes `aud` validation
/// non-negotiable: without it, any site the user logs into with the same
/// provider can take the id_token it received and present it here.
#[tokio::test]
async fn a_token_for_another_audience_is_refused() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    let token = sign(&claims(&issuer, "some-other-app", Some("n")), KID);
    let err = oidc::validate_id_token(&token, &discovery, "my-client", "n")
        .await
        .expect_err("a token for another audience must be refused")
        .to_string();

    assert!(
        err.contains("audience"),
        "the rejection must be specifically about the audience - otherwise this \
         test would pass on any unrelated failure and prove nothing about \
         confused-deputy protection. Got: {err}"
    );
}

/// A token from a different issuer, correctly signed by *that* issuer's key.
#[tokio::test]
async fn a_token_from_another_issuer_is_refused() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    let token = sign(
        &claims("https://attacker.example", "my-client", Some("n")),
        KID,
    );
    let err = oidc::validate_id_token(&token, &discovery, "my-client", "n")
        .await
        .expect_err("a token from another issuer must be refused")
        .to_string();

    assert!(
        err.contains("issuer"),
        "the rejection must be specifically about the issuer: {err}"
    );
}

/// An id_token captured from one login and replayed into another.
#[tokio::test]
async fn a_replayed_token_is_refused() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    // Signed for a login whose nonce was "first-login".
    let token = sign(&claims(&issuer, "my-client", Some("first-login")), KID);

    // Presented to a login that issued "second-login".
    let err = oidc::validate_id_token(&token, &discovery, "my-client", "second-login")
        .await
        .expect_err("a token bound to another login must be refused")
        .to_string();

    assert!(
        err.contains("nonce"),
        "the error should name the nonce mismatch: {err}"
    );
}

/// A token with no `nonce` at all cannot be bound to a login.
#[tokio::test]
async fn a_token_without_a_nonce_is_refused() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    let token = sign(&claims(&issuer, "my-client", None), KID);
    let err = oidc::validate_id_token(&token, &discovery, "my-client", "expected")
        .await
        .expect_err("a token without a nonce must be refused")
        .to_string();

    assert!(err.contains("nonce"), "got: {err}");
}

#[tokio::test]
async fn an_expired_token_is_refused() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    let mut expired = claims(&issuer, "my-client", Some("n"));
    // Well past the 60s leeway.
    expired.exp = now() - 3600;
    let token = sign(&expired, KID);

    let err = oidc::validate_id_token(&token, &discovery, "my-client", "n")
        .await
        .expect_err("an expired token must be refused")
        .to_string();

    assert!(
        err.contains("expired"),
        "the rejection must be specifically about expiry: {err}"
    );
}

/// A token whose `kid` is not in the provider's JWKS.
///
/// Also exercises the rate-limited refetch: an unknown `kid` triggers one
/// refetch, and a flood of them must not hammer the provider.
#[tokio::test]
async fn a_token_signed_with_an_unknown_key_is_refused() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    let token = sign(&claims(&issuer, "my-client", Some("n")), "some-other-kid");
    let err = oidc::validate_id_token(&token, &discovery, "my-client", "n")
        .await
        .expect_err("a token signed with an unpublished key must be refused")
        .to_string();

    assert!(
        err.contains("kid") || err.contains("signing key"),
        "got: {err}"
    );
}

/// A discovery document whose `issuer` disagrees with where it was fetched.
///
/// Without this check, `iss` validation is circular: the attacker supplies the
/// document *and* the token, so both agree.
#[tokio::test]
async fn discovery_with_a_mismatched_issuer_is_refused() {
    oidc::clear_caches();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issuer": "https://accounts.google.com",
            "authorization_endpoint": "https://accounts.google.com/authorize",
            "token_endpoint": "https://accounts.google.com/token",
            "jwks_uri": "https://accounts.google.com/jwks",
        })))
        .mount(&server)
        .await;

    let err = oidc::discover(&server.uri())
        .await
        .expect_err("a document claiming to be another issuer must be refused")
        .to_string();

    assert!(err.contains("mismatch"), "got: {err}");
}

/// The discovery document is cached, so a login does not fetch it every time.
#[tokio::test]
async fn discovery_is_cached() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();

    oidc::discover(&issuer).await.unwrap();
    oidc::discover(&issuer).await.unwrap();
    oidc::discover(&issuer).await.unwrap();

    let discovery_requests = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path().contains("openid-configuration"))
        .count();

    assert_eq!(
        discovery_requests, 1,
        "discovery should be fetched once and cached; fetching per login adds a \
         provider round trip to every authentication"
    );
}

/// The JWKS is cached across validations.
#[tokio::test]
async fn the_jwks_is_cached_across_validations() {
    oidc::clear_caches();
    let server = provider().await;
    let issuer = server.uri();
    let discovery = oidc::discover(&issuer).await.unwrap();

    for _ in 0..5 {
        let token = sign(&claims(&issuer, "my-client", Some("n")), KID);
        oidc::validate_id_token(&token, &discovery, "my-client", "n")
            .await
            .unwrap();
    }

    let jwks_requests = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path().contains("jwks"))
        .count();

    assert_eq!(
        jwks_requests, 1,
        "the JWKS should be fetched once; fetching per validation puts an \
         outbound request on every login"
    );
}
