//! OpenID Connect: discovery, JWKS, and `id_token` validation.
//!
//! C-2/C-3. What the OAuth flow did before this was *authorization*, not
//! authentication. It exchanged a code for an access token, called the
//! provider's userinfo endpoint, and believed whatever came back. That is
//! sufficient only if the access token cannot have come from anywhere else,
//! which is exactly what OIDC exists to establish.
//!
//! Three properties this adds, and what each one prevents:
//!
//! * **PKCE (RFC 7636).** The authorization code is bound to a secret the
//!   client generated and never transmitted. An attacker who intercepts the
//!   code - from a browser log, a referrer header, a shared machine - cannot
//!   redeem it without the verifier.
//! * **`id_token` signature validation.** The provider signs an assertion about
//!   who the user is. Verifying it against the provider's published keys means
//!   identity comes from something the provider signed, not from an HTTP
//!   response that any holder of *some* access token could have elicited.
//! * **Claim validation** - `iss`, `aud`, `exp`, `nonce`. Without `aud`, a token
//!   minted for a *different* application at the same provider is accepted here;
//!   this is the classic confused-deputy in OAuth logins. Without `nonce`, an
//!   `id_token` captured from one login can be replayed into another.
//!
//! ## Not every provider is an OIDC provider
//!
//! GitHub's OAuth is not OIDC: there is no `id_token` and no JWKS. Pretending
//! otherwise would mean either failing GitHub logins or silently skipping
//! validation while claiming to do it. Instead a provider declares whether it is
//! OIDC-capable, and `PANGOLIN_OIDC_REQUIRE` decides whether a non-OIDC provider
//! may be used at all. An operator who needs every login to be OIDC-validated
//! can have that; one who needs GitHub can have that; nobody gets it by accident.

use anyhow::{anyhow, Context, Result};
use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// How long a discovery document is trusted before refetching.
const DISCOVERY_TTL: Duration = Duration::from_secs(3600);
/// How long a JWKS is trusted before refetching on a routine lookup.
const JWKS_TTL: Duration = Duration::from_secs(3600);
/// Minimum gap between forced JWKS refetches triggered by an unknown `kid`.
///
/// Providers rotate keys, and the correct response to an unknown `kid` is to
/// refetch. Doing that unconditionally turns a stream of garbage tokens into a
/// denial-of-service against the provider's JWKS endpoint - and against our own
/// latency, since every such request would block on an outbound fetch.
const JWKS_REFETCH_COOLDOWN: Duration = Duration::from_secs(60);

/// The subset of an OIDC discovery document this needs.
#[derive(Debug, Clone, Deserialize)]
pub struct Discovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    #[serde(default)]
    pub userinfo_endpoint: Option<String>,
    /// Advertised PKCE methods. Absent means the provider does not say, which
    /// is not the same as "does not support" - many providers support S256 and
    /// omit the field.
    #[serde(default)]
    pub code_challenge_methods_supported: Option<Vec<String>>,
}

impl Discovery {
    /// Whether the provider advertises S256 PKCE.
    ///
    /// `None` is treated as "probably yes": PKCE with a provider that ignores it
    /// is harmless - the extra parameters are simply not used - whereas
    /// withholding PKCE from a provider that supports it but does not advertise
    /// it loses a real protection.
    pub fn supports_s256_pkce(&self) -> bool {
        match &self.code_challenge_methods_supported {
            Some(methods) => methods.iter().any(|m| m == "S256"),
            None => true,
        }
    }
}

struct Cached<T> {
    value: T,
    fetched_at: Instant,
}

#[derive(Default)]
struct Caches {
    discovery: HashMap<String, Cached<Discovery>>,
    jwks: HashMap<String, Cached<JwkSet>>,
    last_forced_refetch: HashMap<String, Instant>,
}

fn caches() -> &'static Mutex<Caches> {
    static CACHES: OnceLock<Mutex<Caches>> = OnceLock::new();
    CACHES.get_or_init(|| Mutex::new(Caches::default()))
}

/// Discard every cached document. For tests, and for an operator-triggered
/// reload after changing provider configuration.
pub fn clear_caches() {
    if let Ok(mut guard) = caches().lock() {
        guard.discovery.clear();
        guard.jwks.clear();
        guard.last_forced_refetch.clear();
    }
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        // A provider that hangs must not hang the login. Without a timeout the
        // request inherits the server's global request deadline, which is
        // longer than a user will wait and longer than a healthy provider should take.
        .timeout(Duration::from_secs(10))
        .build()
        .context("could not build an HTTP client for OIDC discovery")
}

/// Fetch (or reuse) a provider's discovery document.
///
/// `issuer_url` is the issuer, not the full `.well-known` path; the suffix is
/// appended here so configuration cannot drift between providers.
pub async fn discover(issuer_url: &str) -> Result<Discovery> {
    let key = issuer_url.to_string();

    if let Ok(guard) = caches().lock() {
        if let Some(entry) = guard.discovery.get(&key) {
            if entry.fetched_at.elapsed() < DISCOVERY_TTL {
                return Ok(entry.value.clone());
            }
        }
    }

    let url = format!(
        "{}/.well-known/openid-configuration",
        issuer_url.trim_end_matches('/')
    );
    let response = http_client()?
        .get(&url)
        .send()
        .await
        .with_context(|| format!("could not reach the OIDC discovery document at {url}"))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "OIDC discovery at {url} returned HTTP {}",
            response.status()
        ));
    }

    let discovery: Discovery = response
        .json()
        .await
        .with_context(|| format!("the OIDC discovery document at {url} is not valid"))?;

    // The issuer in the document is authoritative and must match where we asked.
    // A mismatch means either a misconfiguration or a provider impersonating
    // another, and it is the check that makes `iss` validation meaningful later.
    if discovery.issuer.trim_end_matches('/') != issuer_url.trim_end_matches('/') {
        return Err(anyhow!(
            "OIDC discovery mismatch: asked {issuer_url}, document declares issuer {}",
            discovery.issuer
        ));
    }

    if let Ok(mut guard) = caches().lock() {
        guard.discovery.insert(
            key,
            Cached {
                value: discovery.clone(),
                fetched_at: Instant::now(),
            },
        );
    }

    Ok(discovery)
}

async fn fetch_jwks(jwks_uri: &str) -> Result<JwkSet> {
    let response = http_client()?
        .get(jwks_uri)
        .send()
        .await
        .with_context(|| format!("could not reach the JWKS at {jwks_uri}"))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "JWKS at {jwks_uri} returned HTTP {}",
            response.status()
        ));
    }

    response
        .json::<JwkSet>()
        .await
        .with_context(|| format!("the JWKS at {jwks_uri} is not valid"))
}

/// The signing key for `kid`, refetching once if it is not already known.
///
/// Key rotation is the reason for the refetch: a provider publishes a new key
/// and starts signing with it, and a cache that only expires on a timer rejects
/// every login until the timer fires. The cooldown stops a stream of tokens
/// carrying unknown `kid`s from turning that recovery into a hammer.
async fn signing_key(jwks_uri: &str, kid: &str) -> Result<Jwk> {
    let cached = caches().lock().ok().and_then(|guard| {
        guard
            .jwks
            .get(jwks_uri)
            .filter(|entry| entry.fetched_at.elapsed() < JWKS_TTL)
            .map(|entry| entry.value.clone())
    });

    if let Some(set) = cached {
        if let Some(key) = set.find(kid) {
            return Ok(key.clone());
        }
        // Known set, unknown kid: either rotation or rubbish. Rate-limit the
        // distinction.
        let may_refetch = caches()
            .lock()
            .ok()
            .map(|mut guard| {
                let allowed = guard
                    .last_forced_refetch
                    .get(jwks_uri)
                    .map(|at| at.elapsed() >= JWKS_REFETCH_COOLDOWN)
                    .unwrap_or(true);
                if allowed {
                    guard
                        .last_forced_refetch
                        .insert(jwks_uri.to_string(), Instant::now());
                }
                allowed
            })
            .unwrap_or(false);

        if !may_refetch {
            return Err(anyhow!(
                "no signing key {kid} in the cached JWKS, and a refetch was \
                 attempted too recently. If the provider has just rotated keys \
                 this resolves within a minute."
            ));
        }
    }

    let set = fetch_jwks(jwks_uri).await?;
    let key = set
        .find(kid)
        .cloned()
        .ok_or_else(|| anyhow!("the provider's JWKS has no signing key with kid {kid}"))?;

    if let Ok(mut guard) = caches().lock() {
        guard.jwks.insert(
            jwks_uri.to_string(),
            Cached {
                value: set,
                fetched_at: Instant::now(),
            },
        );
    }

    Ok(key)
}

/// The claims this cares about. Providers send many more.
#[derive(Debug, Clone, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: Option<bool>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    pub iss: String,
    pub exp: usize,
}

/// Verify an `id_token` and return its claims.
///
/// Every check here has a specific attack behind it:
///
/// * **signature** against the provider's published key - otherwise the token is
///   just a base64 string the caller wrote;
/// * **`iss`** must match the discovery document - otherwise a token from any
///   issuer is accepted;
/// * **`aud`** must contain our `client_id` - otherwise a token minted for a
///   *different* application at the same provider logs its holder in here,
///   which is the confused-deputy problem OAuth logins are famous for;
/// * **`exp`** with a small leeway for clock skew;
/// * **`nonce`** must equal the one bound into this login - otherwise an
///   `id_token` observed in one flow can be replayed into another.
pub async fn validate_id_token(
    id_token: &str,
    discovery: &Discovery,
    client_id: &str,
    expected_nonce: &str,
) -> Result<IdTokenClaims> {
    let header =
        jsonwebtoken::decode_header(id_token).context("the id_token is not a well-formed JWT")?;

    let kid = header
        .kid
        .ok_or_else(|| anyhow!("the id_token has no `kid`, so its signing key cannot be found"))?;

    // `alg` comes from the token, which the attacker controls, so it is used
    // only to look up how to verify - never to decide *whether* to. `none` and
    // the HMAC family are rejected outright: accepting HS256 here would let
    // anyone who knows the (public) signing key material forge a token.
    if !matches!(
        header.alg,
        Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512
    ) {
        return Err(anyhow!(
            "the id_token is signed with {:?}, which is not an asymmetric \
             algorithm. Only provider-signed tokens are acceptable.",
            header.alg
        ));
    }

    let jwk = signing_key(&discovery.jwks_uri, &kid).await?;
    let key = DecodingKey::from_jwk(&jwk)
        .context("the provider's JWKS entry could not be used as a decoding key")?;

    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[discovery.issuer.as_str()]);
    validation.set_audience(&[client_id]);
    validation.validate_exp = true;
    // A minute of tolerance. Providers and this server rarely agree to the
    // second, and a token rejected for being one second early is a login
    // failure nobody can diagnose.
    validation.leeway = 60;

    // The underlying error kind is carried through rather than flattened to
    // "failed validation". Which check rejected the token is the difference
    // between a misconfigured `client_id` and an actual confused-deputy attempt,
    // and an operator reading a log needs to be able to tell them apart. It is
    // also what lets the tests assert that a *specific* check fired, instead of
    // passing because some unrelated check happened to reject the token.
    let data = jsonwebtoken::decode::<IdTokenClaims>(id_token, &key, &validation).map_err(|e| {
        let detail = match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                "audience mismatch: this id_token was minted for a different \
                 application at the same provider"
            }
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                "issuer mismatch: this id_token did not come from the configured \
                 provider"
            }
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => "the id_token has expired",
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                "signature mismatch: the id_token was not signed by the provider's key"
            }
            _ => "the id_token failed validation",
        };
        anyhow!("{detail} ({e})")
    })?;

    // `nonce` is not something `jsonwebtoken` knows about, so it is checked
    // here. Constant-time comparison is unnecessary - the nonce is not a secret
    // an attacker is trying to guess byte by byte - but an exact match is.
    match &data.claims.nonce {
        Some(nonce) if nonce == expected_nonce => {}
        Some(_) => {
            return Err(anyhow!(
                "the id_token's nonce does not match this login. This is what a \
                 replayed token looks like."
            ))
        }
        None => {
            return Err(anyhow!(
                "the id_token carries no nonce, so it cannot be bound to this \
                 login and may be a replay from another"
            ))
        }
    }

    Ok(data.claims)
}

/// PKCE parameters for one authorization request.
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

/// Generate an S256 PKCE pair.
///
/// The verifier is 32 random bytes, base64url-encoded - comfortably inside RFC
/// 7636's 43-128 character range and drawn from the system CSPRNG, because a
/// guessable verifier provides no protection at all.
pub fn generate_pkce() -> Result<Pkce> {
    use base64::Engine as _;
    use ring::rand::{SecureRandom, SystemRandom};
    use sha2::{Digest, Sha256};

    let mut bytes = [0u8; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| anyhow!("could not draw PKCE randomness from the system RNG"))?;

    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    Ok(Pkce {
        verifier,
        challenge,
    })
}

/// An opaque, URL-safe random string, for the OIDC `nonce`.
pub fn generate_nonce() -> Result<String> {
    use base64::Engine as _;
    use ring::rand::{SecureRandom, SystemRandom};

    let mut bytes = [0u8; 24];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| anyhow!("could not draw nonce randomness from the system RNG"))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

/// The issuer URL for a provider, if it is OIDC-capable.
///
/// GitHub is deliberately absent: its OAuth is not OIDC, there is no `id_token`
/// and no JWKS. Returning `None` is how the callback knows to fall back to the
/// userinfo endpoint - and how `require_oidc()` knows to refuse it when the
/// operator has demanded OIDC everywhere.
pub fn issuer_for(provider: &str) -> Option<String> {
    // An explicit override wins, so a self-hosted or non-standard deployment
    // (Okta, Keycloak, Auth0, an internal IdP) is configurable without a code
    // change.
    let explicit = std::env::var(format!("PANGOLIN_{}_ISSUER", provider.to_ascii_uppercase()))
        .ok()
        .filter(|v| !v.trim().is_empty());
    if explicit.is_some() {
        return explicit;
    }

    match provider {
        "google" => Some("https://accounts.google.com".to_string()),
        "microsoft" => std::env::var("PANGOLIN_MICROSOFT_TENANT_ID")
            .ok()
            .map(|tenant| format!("https://login.microsoftonline.com/{tenant}/v2.0")),
        "okta" => std::env::var("PANGOLIN_OKTA_DOMAIN")
            .ok()
            .map(|domain| format!("https://{domain}")),
        // GitHub OAuth is not OIDC.
        _ => None,
    }
}

/// Whether every login must be OIDC-validated.
///
/// Off by default: turning it on without warning would break a working GitHub
/// deployment on upgrade. On, a provider with no issuer is refused rather than
/// quietly downgraded, which is the property an operator turning this on is
/// asking for.
pub fn require_oidc() -> bool {
    std::env::var("PANGOLIN_OIDC_REQUIRE")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_pairs_are_random_and_well_formed() {
        let a = generate_pkce().unwrap();
        let b = generate_pkce().unwrap();

        assert_ne!(
            a.verifier, b.verifier,
            "a repeated verifier would make PKCE decorative"
        );
        // RFC 7636 section 4.1.
        assert!(
            (43..=128).contains(&a.verifier.len()),
            "verifier length {} is outside RFC 7636's 43-128",
            a.verifier.len()
        );
        assert!(
            !a.verifier.contains('+') && !a.verifier.contains('/') && !a.verifier.contains('='),
            "the verifier must be base64url without padding: {}",
            a.verifier
        );
        assert_ne!(
            a.challenge, a.verifier,
            "S256 means the challenge is the hash, not the verifier itself - \
             sending the verifier as the challenge would defeat the whole point"
        );
    }

    #[test]
    fn the_challenge_is_the_sha256_of_the_verifier() {
        use base64::Engine as _;
        use sha2::{Digest, Sha256};

        let pkce = generate_pkce().unwrap();
        let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(pkce.verifier.as_bytes()));
        assert_eq!(
            pkce.challenge, expected,
            "the provider recomputes this; a mismatch fails every login"
        );
    }

    #[test]
    fn nonces_are_unique() {
        let a = generate_nonce().unwrap();
        let b = generate_nonce().unwrap();
        assert_ne!(a, b);
        assert!(a.len() >= 32, "a short nonce is a guessable nonce: {a}");
    }

    #[test]
    fn github_is_not_treated_as_an_oidc_provider() {
        // GitHub OAuth issues no id_token. Claiming otherwise would mean either
        // failing every GitHub login or skipping validation while reporting it
        // as done.
        assert!(issuer_for("github").is_none());
    }

    #[test]
    fn google_has_a_known_issuer() {
        assert_eq!(
            issuer_for("google").as_deref(),
            Some("https://accounts.google.com")
        );
    }

    #[test]
    fn an_explicit_issuer_overrides_the_default() {
        // Self-hosted and non-standard deployments must be configurable without
        // a code change.
        std::env::set_var("PANGOLIN_GOOGLE_ISSUER", "https://idp.internal/realms/x");
        let issuer = issuer_for("google");
        std::env::remove_var("PANGOLIN_GOOGLE_ISSUER");
        assert_eq!(issuer.as_deref(), Some("https://idp.internal/realms/x"));
    }

    #[test]
    fn pkce_is_offered_when_the_provider_is_silent() {
        // Omitting the field is common and does not mean unsupported. Sending
        // PKCE to a provider that ignores it costs nothing; withholding it from
        // one that supports it loses a real protection.
        let discovery = Discovery {
            issuer: "https://i".into(),
            authorization_endpoint: "https://i/a".into(),
            token_endpoint: "https://i/t".into(),
            jwks_uri: "https://i/j".into(),
            userinfo_endpoint: None,
            code_challenge_methods_supported: None,
        };
        assert!(discovery.supports_s256_pkce());
    }

    #[test]
    fn plain_pkce_alone_is_not_treated_as_s256() {
        let discovery = Discovery {
            issuer: "https://i".into(),
            authorization_endpoint: "https://i/a".into(),
            token_endpoint: "https://i/t".into(),
            jwks_uri: "https://i/j".into(),
            userinfo_endpoint: None,
            code_challenge_methods_supported: Some(vec!["plain".into()]),
        };
        assert!(
            !discovery.supports_s256_pkce(),
            "`plain` offers no protection against an intercepted challenge; it \
             must not be mistaken for S256"
        );
    }

    #[tokio::test]
    async fn an_id_token_signed_with_hmac_is_refused() {
        // The `alg` header is attacker-controlled. Accepting HS256 would let
        // anyone who knows the provider's *public* key forge a token, because
        // for HMAC the verification key and the signing key are the same.
        use jsonwebtoken::{encode, EncodingKey, Header};

        #[derive(serde::Serialize)]
        struct Claims {
            sub: String,
            iss: String,
            aud: String,
            exp: usize,
            nonce: String,
        }

        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("attacker".into());
        let token = encode(
            &header,
            &Claims {
                sub: "victim".into(),
                iss: "https://issuer".into(),
                aud: "client".into(),
                exp: 9_999_999_999,
                nonce: "n".into(),
            },
            &EncodingKey::from_secret(b"public-key-material"),
        )
        .unwrap();

        let discovery = Discovery {
            issuer: "https://issuer".into(),
            authorization_endpoint: "https://issuer/a".into(),
            token_endpoint: "https://issuer/t".into(),
            jwks_uri: "https://issuer/jwks".into(),
            userinfo_endpoint: None,
            code_challenge_methods_supported: None,
        };

        let err = validate_id_token(&token, &discovery, "client", "n")
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not an asymmetric"),
            "an HMAC-signed id_token must be refused before any key lookup: {err}"
        );
    }

    #[tokio::test]
    async fn an_id_token_without_a_kid_is_refused() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        #[derive(serde::Serialize)]
        struct Claims {
            sub: String,
            exp: usize,
        }

        let token = encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                sub: "x".into(),
                exp: 9_999_999_999,
            },
            &EncodingKey::from_secret(b"k"),
        )
        .unwrap();

        let discovery = Discovery {
            issuer: "https://issuer".into(),
            authorization_endpoint: "https://issuer/a".into(),
            token_endpoint: "https://issuer/t".into(),
            jwks_uri: "https://issuer/jwks".into(),
            userinfo_endpoint: None,
            code_challenge_methods_supported: None,
        };

        let err = validate_id_token(&token, &discovery, "client", "n")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("kid"), "got: {err}");
    }

    #[test]
    fn oidc_is_not_required_by_default() {
        std::env::remove_var("PANGOLIN_OIDC_REQUIRE");
        assert!(
            !require_oidc(),
            "defaulting this on would break a working GitHub deployment on \
             upgrade, with no warning"
        );
    }
}
