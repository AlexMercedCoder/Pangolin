//! Service-user API keys with a public key ID.
//!
//! The original scheme stored only a bcrypt hash of the key, so authenticating
//! one request meant listing every tenant, listing every service user in each
//! tenant, and running `bcrypt::verify` against each hash until one matched.
//! bcrypt at the default cost is deliberately ~100-250 ms, so a deployment with
//! 100 service users burned roughly 25 CPU-seconds per request — before any
//! rate limiting, and reachable with a bogus key. That is an unauthenticated
//! denial-of-service primitive, not merely a slow path.
//!
//! Keys now carry a public, non-secret key ID:
//!
//! ```text
//! pgl_<key_id>_<secret>
//! ```
//!
//! and the stored credential is `<key_id>$<bcrypt(secret)>`. Authentication
//! selects the single candidate whose key ID matches and runs **exactly one**
//! bcrypt verification — or zero, when no key ID matches. The key ID is stored
//! alongside the hash rather than in a new column so that no backend migration
//! is required; it is not secret and reveals nothing about the key material.

use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;

const KEY_PREFIX: &str = "pgl";
const KEY_ID_LEN: usize = 12;
const SECRET_LEN: usize = 48;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

/// A freshly generated API key: the plaintext shown to the user exactly once,
/// and the credential to persist in `ServiceUser::api_key_hash`.
pub struct GeneratedApiKey {
    /// Shown to the caller once, never stored.
    pub plaintext: String,
    /// Stored value, of the form `<key_id>$<bcrypt hash>`.
    pub stored: String,
    /// Public identifier, embedded in the plaintext key.
    pub key_id: String,
}

/// Generate a new API key and the value to persist for it.
pub fn generate() -> Result<GeneratedApiKey, bcrypt::BcryptError> {
    let key_id = random_string(KEY_ID_LEN);
    let secret = random_string(SECRET_LEN);
    let plaintext = format!("{KEY_PREFIX}_{key_id}_{secret}");
    let stored = format!("{key_id}${}", hash(&secret, DEFAULT_COST)?);
    Ok(GeneratedApiKey {
        plaintext,
        stored,
        key_id,
    })
}

/// The key ID embedded in a presented API key, if it uses the keyed format.
///
/// Returns `None` for legacy keys, which carry no key ID and can only be
/// checked by scanning.
pub fn key_id_of(presented: &str) -> Option<&str> {
    let rest = presented.strip_prefix(KEY_PREFIX)?.strip_prefix('_')?;
    let (key_id, secret) = rest.split_once('_')?;
    if key_id.is_empty() || secret.is_empty() {
        return None;
    }
    Some(key_id)
}

/// The key ID a stored credential belongs to, if it uses the keyed format.
pub fn stored_key_id(stored: &str) -> Option<&str> {
    let (key_id, hash) = stored.split_once('$')?;
    // A bare bcrypt hash also contains '$' (it starts with "$2b$"), so require a
    // non-empty prefix before the first separator to distinguish the two.
    if key_id.is_empty() || !hash.starts_with('$') {
        return None;
    }
    Some(key_id)
}

/// Verify a presented key against a stored credential.
///
/// Handles both the keyed format and legacy bare-bcrypt values, so that callers
/// can support legacy keys where the operator has explicitly opted in.
pub fn verify_against(presented: &str, stored: &str) -> bool {
    match (stored_key_id(stored), key_id_of(presented)) {
        // Keyed credential, keyed key: the IDs must match, then verify the
        // secret half only.
        (Some(stored_id), Some(presented_id)) => {
            if stored_id != presented_id {
                return false;
            }
            let Some(hash) = stored.split_once('$').map(|(_, h)| h) else {
                return false;
            };
            let Some(secret) = presented.rsplit_once('_').map(|(_, s)| s) else {
                return false;
            };
            verify(secret, hash).unwrap_or(false)
        }
        // Legacy credential: the whole presented key is the secret.
        (None, _) => verify(presented, stored).unwrap_or(false),
        // Keyed credential but a legacy-shaped key: cannot match.
        (Some(_), None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_round_trips() {
        let key = generate().expect("generate");
        assert!(key.plaintext.starts_with("pgl_"));
        assert_eq!(key_id_of(&key.plaintext), Some(key.key_id.as_str()));
        assert_eq!(stored_key_id(&key.stored), Some(key.key_id.as_str()));
        assert!(verify_against(&key.plaintext, &key.stored));
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let key = generate().expect("generate");
        let tampered = format!("pgl_{}_{}", key.key_id, "x".repeat(SECRET_LEN));
        assert!(!verify_against(&tampered, &key.stored));
    }

    #[test]
    fn wrong_key_id_is_rejected_without_bcrypt() {
        let a = generate().expect("generate");
        let b = generate().expect("generate");
        assert!(!verify_against(&a.plaintext, &b.stored));
    }

    /// The whole point of the key ID: a presented key selects at most one
    /// candidate, so a bogus key costs zero bcrypt verifications.
    #[test]
    fn key_id_selects_at_most_one_candidate() {
        let stored: Vec<String> = (0..8)
            .map(|_| generate().expect("generate").stored)
            .collect();
        let bogus = "pgl_notarealkeyid_deadbeef";
        let candidates = stored
            .iter()
            .filter(|s| stored_key_id(s) == key_id_of(bogus))
            .count();
        assert_eq!(candidates, 0);
    }

    #[test]
    fn legacy_bare_bcrypt_credentials_still_verify() {
        let legacy_plaintext = "someOldStyleKeyWithoutAnyKeyId";
        let legacy_stored = hash(legacy_plaintext, 4).expect("hash");
        assert_eq!(stored_key_id(&legacy_stored), None);
        assert!(verify_against(legacy_plaintext, &legacy_stored));
        assert!(!verify_against("wrong", &legacy_stored));
    }

    #[test]
    fn keyed_credential_rejects_legacy_shaped_presentation() {
        let key = generate().expect("generate");
        assert!(!verify_against("plain-old-key", &key.stored));
    }
}
