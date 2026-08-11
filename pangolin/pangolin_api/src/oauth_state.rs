//! Signed OAuth `state` values, single-use nonces, and redirect allowlisting.
//!
//! Two critical defects motivated this module.
//!
//! **A-8 — token exfiltration.** The callback used to base64-decode `state`,
//! read `redirect_uri` out of it, and append the freshly minted session JWT as
//! a query parameter. `state` was plain base64 JSON: unsigned, unencrypted, and
//! never stored server-side, with no allowlist on the destination. Sending a
//! victim an authorize link whose `state` decoded to
//! `{"redirect_uri":"https://evil.com/"}` caused Pangolin to 302 the victim's
//! browser to the attacker's host with a valid token in the URL — full account
//! takeover with no credential theft.
//!
//! **A-9 — decorative nonce.** A nonce was generated and embedded in `state`,
//! but nothing ever stored or verified it, so the callback accepted any `state`
//! at all. That is a textbook login-CSRF hole.
//!
//! What this module enforces:
//!
//! * `state` is HMAC-SHA256 signed with the server's secret and carries an
//!   expiry, so it cannot be forged or replayed indefinitely.
//! * The nonce inside it is registered server-side at authorize time and
//!   consumed exactly once at callback time; a second use is rejected.
//! * `redirect_uri` is validated against an operator-configured allowlist by
//!   exact match, and is *not* carried inside `state` as an arbitrary string —
//!   only an index into the allowlist is.
//! * The token is never placed in a redirect URL. The callback issues a
//!   short-lived, single-use exchange code; the client POSTs it to
//!   `/api/v1/oauth/exchange` to obtain the session token in a response body.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// How long an authorize request may sit before its `state` stops being valid.
pub const STATE_TTL: Duration = Duration::from_secs(600);
/// How long a callback's exchange code remains redeemable.
pub const EXCHANGE_CODE_TTL: Duration = Duration::from_secs(120);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The payload carried inside a signed `state` value.
///
/// `redirect_index` is an index into the server's configured allowlist, never a
/// URL supplied by the caller.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatePayload {
    pub nonce: String,
    pub provider: String,
    pub redirect_index: usize,
    pub expires_at: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StateError {
    #[error("state parameter is missing")]
    Missing,
    #[error("state parameter is malformed")]
    Malformed,
    #[error("state signature is invalid")]
    BadSignature,
    #[error("state has expired")]
    Expired,
    #[error("state nonce was not issued by this server, or has already been used")]
    UnknownNonce,
    #[error("state was issued for provider {issued:?} but presented to {presented:?}")]
    ProviderMismatch { issued: String, presented: String },
    #[error("redirect_uri {0:?} is not in PANGOLIN_OAUTH_REDIRECT_URIS")]
    RedirectNotAllowed(String),
}

/// Resolve a client-supplied redirect URI to its allowlist index.
///
/// `None` (no redirect requested) resolves to the configured frontend URL,
/// which is always index 0 of the effective allowlist.
pub fn resolve_redirect(
    requested: Option<&str>,
    allowlist: &[String],
) -> Result<usize, StateError> {
    match requested {
        None => Ok(0),
        Some(uri) => allowlist
            .iter()
            .position(|allowed| allowed == uri)
            .ok_or_else(|| StateError::RedirectNotAllowed(uri.to_string())),
    }
}

fn sign(secret: &str, message: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(message);
    B64.encode(mac.finalize().into_bytes())
}

/// Build a signed, single-use `state` value and register its nonce.
pub fn issue(secret: &str, provider: &str, redirect_index: usize) -> String {
    let payload = StatePayload {
        nonce: Uuid::new_v4().to_string(),
        provider: provider.to_string(),
        redirect_index,
        expires_at: now_secs() + STATE_TTL.as_secs(),
    };
    nonce_store().register(&payload.nonce, payload.expires_at);
    encode_signed(secret, &payload)
}

/// What an in-flight OIDC login needs to remember between authorize and
/// callback.
///
/// **This is deliberately server-side and not carried in `state`.** The PKCE
/// verifier is a secret: its entire purpose is that an attacker holding the
/// authorization code cannot redeem it. `state` travels to the provider and back
/// through the user's browser in the same URL as the code, so anyone positioned
/// to steal the code - a referrer header, a proxy log, shell history on a shared
/// machine - would also hold the verifier. Putting it there would make PKCE
/// decorative in exactly the situation it exists for.
///
/// In-process, like the nonce store beside it, which is why OAuth needs session
/// affinity across replicas. See docs/operations/running-multiple-replicas.md.
#[derive(Debug, Clone)]
pub struct PendingLogin {
    pub pkce_verifier: String,
    pub oidc_nonce: String,
    expires_at: u64,
}

fn pending_logins() -> &'static Mutex<HashMap<String, PendingLogin>> {
    static STORE: OnceLock<Mutex<HashMap<String, PendingLogin>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Remember the PKCE verifier and OIDC nonce for a login, keyed by its state
/// nonce.
pub fn remember_login(state_nonce: &str, pkce_verifier: String, oidc_nonce: String) {
    let expires_at = now_secs() + STATE_TTL.as_secs();
    if let Ok(mut guard) = pending_logins().lock() {
        // Opportunistic sweep: without it an abandoned authorize - a user who
        // closes the tab at the provider - leaks an entry forever.
        let now = now_secs();
        guard.retain(|_, pending| pending.expires_at > now);
        guard.insert(
            state_nonce.to_string(),
            PendingLogin {
                pkce_verifier,
                oidc_nonce,
                expires_at,
            },
        );
    }
}

/// Take the remembered login. Single-use, like the state it is keyed by.
pub fn take_login(state_nonce: &str) -> Option<PendingLogin> {
    let mut guard = pending_logins().lock().ok()?;
    let pending = guard.remove(state_nonce)?;
    if pending.expires_at <= now_secs() {
        return None;
    }
    Some(pending)
}

/// Read the nonce out of a `state` this server just issued, without consuming
/// it.
///
/// `issue` returns the encoded string, and the caller needs the nonce inside it
/// as the key for the pending-login store. Returning it from `issue` would be
/// tidier, but that signature is used in several places and a second return
/// value would be silently ignored at most of them.
///
/// This still verifies the signature: reading an unverified payload, even one we
/// believe we just wrote, is the habit that makes the next reader assume it is
/// safe elsewhere.
pub fn peek_nonce(secret: &str, state: &str) -> Option<String> {
    let (encoded, signature) = state.rsplit_once('.')?;
    if sign(secret, encoded.as_bytes()) != signature {
        return None;
    }
    let body = B64.decode(encoded).ok()?;
    let payload: StatePayload = serde_json::from_slice(&body).ok()?;
    Some(payload.nonce)
}

fn encode_signed(secret: &str, payload: &StatePayload) -> String {
    let body = serde_json::to_vec(payload).expect("StatePayload is always serializable");
    let encoded = B64.encode(&body);
    let signature = sign(secret, encoded.as_bytes());
    format!("{encoded}.{signature}")
}

/// Verify a `state` value: signature, expiry, provider binding, and nonce.
///
/// Consumes the nonce, so a replayed `state` fails.
pub fn verify_and_consume(
    secret: &str,
    provider: &str,
    state: Option<&str>,
) -> Result<StatePayload, StateError> {
    let state = state.ok_or(StateError::Missing)?;
    let (encoded, signature) = state.rsplit_once('.').ok_or(StateError::Malformed)?;

    let expected = sign(secret, encoded.as_bytes());
    // Both sides are base64 of a fixed-size MAC, so a length-aware constant-time
    // comparison is appropriate here.
    if !crate::config::constant_time_eq(&expected, signature) {
        return Err(StateError::BadSignature);
    }

    let raw = B64.decode(encoded).map_err(|_| StateError::Malformed)?;
    let payload: StatePayload = serde_json::from_slice(&raw).map_err(|_| StateError::Malformed)?;

    if payload.expires_at < now_secs() {
        return Err(StateError::Expired);
    }
    if payload.provider != provider {
        return Err(StateError::ProviderMismatch {
            issued: payload.provider.clone(),
            presented: provider.to_string(),
        });
    }
    if !nonce_store().consume(&payload.nonce) {
        return Err(StateError::UnknownNonce);
    }
    Ok(payload)
}

// ---------------------------------------------------------------------------
// Nonce and exchange-code stores
// ---------------------------------------------------------------------------

/// In-process single-use token store with expiry.
///
/// Node-local, which is correct for `state` (the same browser returns to the
/// node that issued the redirect only if the load balancer is sticky). For
/// multi-replica deployments without session affinity this should move to the
/// shared store; see the TODO in `docs/operations/oidc.md`.
#[derive(Default)]
struct SingleUseStore {
    entries: Mutex<HashMap<String, u64>>,
}

impl SingleUseStore {
    fn register(&self, key: &str, expires_at: u64) {
        let mut guard = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let now = now_secs();
        guard.retain(|_, exp| *exp > now);
        guard.insert(key.to_string(), expires_at);
    }

    fn consume(&self, key: &str) -> bool {
        let mut guard = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        match guard.remove(key) {
            Some(expires_at) => expires_at > now_secs(),
            None => false,
        }
    }
}

fn nonce_store() -> &'static SingleUseStore {
    static STORE: OnceLock<SingleUseStore> = OnceLock::new();
    STORE.get_or_init(SingleUseStore::default)
}

/// A minted session token held server-side until the client redeems its code.
struct PendingToken {
    token: String,
    expires_at: u64,
}

fn exchange_store() -> &'static Mutex<HashMap<String, PendingToken>> {
    static STORE: OnceLock<Mutex<HashMap<String, PendingToken>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Park a freshly minted session token and return the one-time code that
/// redeems it. The token itself never enters a URL.
pub fn stash_token(token: String) -> String {
    let code = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let mut guard = exchange_store().lock().unwrap_or_else(|e| e.into_inner());
    let now = now_secs();
    guard.retain(|_, pending| pending.expires_at > now);
    guard.insert(
        code.clone(),
        PendingToken {
            token,
            expires_at: now + EXCHANGE_CODE_TTL.as_secs(),
        },
    );
    code
}

/// Redeem a one-time exchange code for the session token it stands for.
pub fn redeem_code(code: &str) -> Option<String> {
    let mut guard = exchange_store().lock().unwrap_or_else(|e| e.into_inner());
    let pending = guard.remove(code)?;
    if pending.expires_at <= now_secs() {
        return None;
    }
    Some(pending.token)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "a-test-signing-secret-of-adequate-length-000000";

    #[test]
    fn issued_state_verifies_once() {
        let state = issue(SECRET, "google", 0);
        let payload = verify_and_consume(SECRET, "google", Some(&state)).expect("first use");
        assert_eq!(payload.redirect_index, 0);
        assert_eq!(payload.provider, "google");
    }

    /// Regression test for A-9: the nonce is single-use, so a captured `state`
    /// cannot be replayed.
    #[test]
    fn state_cannot_be_replayed() {
        let state = issue(SECRET, "github", 0);
        assert!(verify_and_consume(SECRET, "github", Some(&state)).is_ok());
        assert_eq!(
            verify_and_consume(SECRET, "github", Some(&state)),
            Err(StateError::UnknownNonce)
        );
    }

    /// Regression test for A-9: an attacker-authored `state` is rejected,
    /// where the old code accepted any base64 JSON blob.
    #[test]
    fn forged_state_is_rejected() {
        let forged = StatePayload {
            nonce: Uuid::new_v4().to_string(),
            provider: "google".to_string(),
            redirect_index: 0,
            expires_at: now_secs() + 600,
        };
        let unsigned = B64.encode(serde_json::to_vec(&forged).unwrap());
        assert_eq!(
            verify_and_consume(SECRET, "google", Some(&unsigned)),
            Err(StateError::Malformed)
        );

        // Correct shape, wrong key.
        let wrong_key = encode_signed("some-other-secret", &forged);
        assert_eq!(
            verify_and_consume(SECRET, "google", Some(&wrong_key)),
            Err(StateError::BadSignature)
        );
    }

    #[test]
    fn missing_state_is_rejected() {
        assert_eq!(
            verify_and_consume(SECRET, "google", None),
            Err(StateError::Missing)
        );
    }

    #[test]
    fn expired_state_is_rejected() {
        let payload = StatePayload {
            nonce: Uuid::new_v4().to_string(),
            provider: "okta".to_string(),
            redirect_index: 0,
            expires_at: now_secs() - 1,
        };
        nonce_store().register(&payload.nonce, payload.expires_at);
        let state = encode_signed(SECRET, &payload);
        assert_eq!(
            verify_and_consume(SECRET, "okta", Some(&state)),
            Err(StateError::Expired)
        );
    }

    #[test]
    fn state_is_bound_to_its_provider() {
        let state = issue(SECRET, "google", 0);
        assert!(matches!(
            verify_and_consume(SECRET, "github", Some(&state)),
            Err(StateError::ProviderMismatch { .. })
        ));
    }

    /// Regression test for A-8: an attacker-chosen redirect target is refused
    /// rather than being echoed back with a token attached.
    #[test]
    fn redirect_uri_must_be_allowlisted() {
        let allowlist = vec![
            "https://app.example.com/oauth".to_string(),
            "http://localhost:5173".to_string(),
        ];
        assert_eq!(resolve_redirect(None, &allowlist), Ok(0));
        assert_eq!(
            resolve_redirect(Some("http://localhost:5173"), &allowlist),
            Ok(1)
        );
        assert_eq!(
            resolve_redirect(Some("https://evil.com/"), &allowlist),
            Err(StateError::RedirectNotAllowed("https://evil.com/".into()))
        );
        // Prefix tricks must not pass either.
        assert!(resolve_redirect(Some("https://app.example.com.evil.com/"), &allowlist).is_err());
        assert!(resolve_redirect(Some("https://app.example.com/oauth/../x"), &allowlist).is_err());
    }

    #[test]
    fn exchange_code_is_single_use_and_carries_the_token() {
        let code = stash_token("the-session-jwt".to_string());
        assert_eq!(redeem_code(&code).as_deref(), Some("the-session-jwt"));
        assert_eq!(redeem_code(&code), None);
    }

    #[test]
    fn unknown_exchange_code_yields_nothing() {
        assert_eq!(redeem_code("not-a-real-code"), None);
    }
}
