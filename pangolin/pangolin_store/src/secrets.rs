//! Envelope encryption for the secrets inside a warehouse's `storage_config`.
//!
//! C-11. A warehouse holds the credentials Pangolin uses to reach a customer's
//! object storage - AWS secret access keys, Azure account keys, GCP service
//! account JSON. They were stored in the catalog database as plaintext JSON, so
//! anything that could read one row of the `warehouses` table - a backup, a
//! read replica, a SQL injection elsewhere, an operator with analyst access -
//! held every tenant's cloud credentials.
//!
//! What this does and does not protect:
//!
//! * **Does** protect against disclosure of the database contents alone: a
//!   dump, a stolen backup, a snapshot, a curious `SELECT`.
//! * **Does not** protect against an attacker who has both the database and
//!   the key. The key lives in the server's environment, so a full compromise
//!   of a running server still yields the credentials. That is the normal
//!   limit of envelope encryption without an HSM or a KMS, and it is worth
//!   stating plainly rather than implying more.
//!
//! ## Format
//!
//! A sealed value is `enc:v1:<base64(nonce ‖ ciphertext ‖ tag)>`, AES-256-GCM
//! with a random 96-bit nonce per value. The version is in the string so a
//! future scheme can be introduced without guessing at what old rows contain.
//!
//! ## Reading is deliberately tolerant
//!
//! [`open`] decrypts anything carrying the prefix and passes everything else
//! through untouched. That is what makes this deployable: a database written
//! before this existed is full of plaintext, and refusing to read it would turn
//! a security improvement into an outage. The cost is that a value which was
//! never sealed stays unsealed until something rewrites it - see
//! `docs/operations/encryption.md` for how to force that.

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use std::collections::HashMap;

/// Marks a sealed value. Also how [`open`] tells sealed from legacy plaintext.
const PREFIX: &str = "enc:v1:";

/// The `storage_config` keys whose values are credentials.
///
/// An allowlist rather than "encrypt everything": the same map carries the
/// bucket name, region and endpoint, which callers compare, log and use to
/// build URLs. Encrypting those would break the object-store factory and the
/// UI for no security gain, because they are not secrets.
///
/// Both the dotted and undotted spellings appear in real configurations (the UI
/// writes `secret_access_key`, parts of the server read
/// `s3.secret-access-key`), so both are listed. Missing a spelling here means a
/// credential silently stays in plaintext, which is why `is_sensitive` is
/// tested against the exact keys the create-warehouse form writes.
const SENSITIVE_KEYS: &[&str] = &[
    "secret_access_key",
    "s3.secret-access-key",
    "access_key_id",
    "s3.access-key-id",
    "session_token",
    "s3.session-token",
    "account_key",
    "azure.account-key",
    "adls.account-key",
    "client_secret",
    "azure.client-secret",
    "service_account_json",
    "gcs.service-account-key",
    "external_id",
];

pub fn is_sensitive(key: &str) -> bool {
    SENSITIVE_KEYS.contains(&key)
}

/// The configured data key, or `None` when the operator has not set one.
///
/// Read on every call rather than cached in a `OnceLock`: the tests set and
/// clear it around individual cases, and a cached key would make the first test
/// to run decide the behaviour of the rest.
fn data_key() -> Result<Option<LessSafeKey>> {
    let Some(raw) = std::env::var("PANGOLIN_ENCRYPTION_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty())
    else {
        return Ok(None);
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw.trim())
        .context(
            "PANGOLIN_ENCRYPTION_KEY must be base64. Generate one with: \
             openssl rand -base64 32",
        )?;

    if bytes.len() != 32 {
        return Err(anyhow!(
            "PANGOLIN_ENCRYPTION_KEY decodes to {} bytes; AES-256-GCM needs 32. \
             Generate one with: openssl rand -base64 32",
            bytes.len()
        ));
    }

    let unbound = UnboundKey::new(&AES_256_GCM, &bytes)
        .map_err(|_| anyhow!("PANGOLIN_ENCRYPTION_KEY is not a usable AES-256 key"))?;
    Ok(Some(LessSafeKey::new(unbound)))
}

/// Whether at-rest encryption is configured.
pub fn is_enabled() -> bool {
    matches!(data_key(), Ok(Some(_)))
}

fn seal_value(key: &LessSafeKey, plaintext: &str) -> Result<String> {
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow!("could not draw a nonce from the system RNG"))?;

    let mut buffer = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(nonce_bytes),
        Aad::empty(),
        &mut buffer,
    )
    .map_err(|_| anyhow!("could not encrypt a warehouse credential"))?;

    let mut payload = Vec::with_capacity(NONCE_LEN + buffer.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&buffer);

    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(payload)
    ))
}

fn open_value(key: &LessSafeKey, sealed: &str) -> Result<String> {
    let encoded = sealed
        .strip_prefix(PREFIX)
        .ok_or_else(|| anyhow!("value is not sealed"))?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("a sealed warehouse credential is not valid base64")?;

    if payload.len() <= NONCE_LEN {
        return Err(anyhow!("a sealed warehouse credential is truncated"));
    }

    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(nonce_bytes);

    let mut buffer = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut buffer,
        )
        .map_err(|_| {
            anyhow!(
                "could not decrypt a warehouse credential. The most likely cause is \
                 that PANGOLIN_ENCRYPTION_KEY is not the key this row was written \
                 with."
            )
        })?;

    String::from_utf8(plaintext.to_vec())
        .context("a decrypted warehouse credential is not valid UTF-8")
}

/// Encrypt the credential-bearing entries of a `storage_config`, in place.
///
/// A no-op when no key is configured, so upgrading without setting one keeps
/// working exactly as before. The server logs a warning at startup in that
/// case; silently doing nothing is the failure mode this whole audit has been
/// about, so it is said out loud there rather than only here.
pub fn seal(config: &mut HashMap<String, String>) -> Result<()> {
    let Some(key) = data_key()? else {
        return Ok(());
    };
    seal_with(&key, config)
}

/// [`seal`] against an explicit key.
///
/// Split out so the crypto is testable without setting a process-wide
/// environment variable - which the tests were doing, and which raced when they
/// ran in parallel: one test's `unset` decided another's behaviour.
pub(crate) fn seal_with(key: &LessSafeKey, config: &mut HashMap<String, String>) -> Result<()> {
    for (name, value) in config.iter_mut() {
        // Already sealed: re-sealing would double-encrypt and the value could
        // never be read back.
        if !is_sensitive(name) || value.starts_with(PREFIX) || value.is_empty() {
            continue;
        }
        *value = seal_value(key, value)?;
    }
    Ok(())
}

/// Decrypt any sealed entries, in place. Values without the prefix are left
/// alone, which is what lets a database written before 0.8.0 still be read.
pub fn open(config: &mut HashMap<String, String>) -> Result<()> {
    // Nothing sealed: avoid demanding a key just to read a legacy row.
    if !config.values().any(|v| v.starts_with(PREFIX)) {
        return Ok(());
    }

    let Some(key) = data_key()? else {
        return Err(anyhow!(
            "this warehouse has encrypted credentials but PANGOLIN_ENCRYPTION_KEY \
             is not set. Set it to the key the credentials were written with; \
             without it they cannot be recovered."
        ));
    };
    open_with(&key, config)
}

/// [`open`] against an explicit key.
pub(crate) fn open_with(key: &LessSafeKey, config: &mut HashMap<String, String>) -> Result<()> {
    for value in config.values_mut() {
        if value.starts_with(PREFIX) {
            *value = open_value(key, value)?;
        }
    }
    Ok(())
}

/// True when the map still holds an unencrypted credential.
///
/// Used by the operations tooling to report what a re-seal would cover, and by
/// the cross-backend test that asserts nothing reaches storage in the clear.
pub fn has_plaintext_secret(config: &HashMap<String, String>) -> bool {
    config
        .iter()
        .any(|(k, v)| is_sensitive(k) && !v.is_empty() && !v.starts_with(PREFIX))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sets the key for one test and restores the previous value afterwards.
    /// The suite runs in one process, so leaking this would decide unrelated
    /// tests' behaviour.
    struct KeyGuard(Option<String>);

    impl KeyGuard {
        fn set(value: &str) -> Self {
            let previous = std::env::var("PANGOLIN_ENCRYPTION_KEY").ok();
            std::env::set_var("PANGOLIN_ENCRYPTION_KEY", value);
            Self(previous)
        }
        fn unset() -> Self {
            let previous = std::env::var("PANGOLIN_ENCRYPTION_KEY").ok();
            std::env::remove_var("PANGOLIN_ENCRYPTION_KEY");
            Self(previous)
        }
    }

    impl Drop for KeyGuard {
        fn drop(&mut self) {
            match &self.0 {
                Some(v) => std::env::set_var("PANGOLIN_ENCRYPTION_KEY", v),
                None => std::env::remove_var("PANGOLIN_ENCRYPTION_KEY"),
            }
        }
    }

    /// A key built directly, with no environment variable anywhere near it.
    /// Everything that tests the *crypto* uses this; only the handful of cases
    /// that test environment handling touch the process env, and those are
    /// serialised.
    fn key_from(byte: u8) -> LessSafeKey {
        LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &[byte; 32]).unwrap())
    }

    fn config_with_secret() -> HashMap<String, String> {
        HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "customer-data".to_string()),
            ("region".to_string(), "us-east-1".to_string()),
            ("secret_access_key".to_string(), "SUPER-SECRET".to_string()),
        ])
    }

    #[test]
    fn a_sealed_value_round_trips() {
        let key = key_from(7);
        let mut config = config_with_secret();
        seal_with(&key, &mut config).unwrap();
        assert_ne!(config["secret_access_key"], "SUPER-SECRET");
        open_with(&key, &mut config).unwrap();
        assert_eq!(config["secret_access_key"], "SUPER-SECRET");
    }

    #[test]
    fn only_credentials_are_encrypted() {
        let key = key_from(7);
        let mut config = config_with_secret();
        seal_with(&key, &mut config).unwrap();

        // The object-store factory compares and concatenates these; encrypting
        // them would break every storage operation for no security gain.
        assert_eq!(config["bucket"], "customer-data");
        assert_eq!(config["region"], "us-east-1");
        assert_eq!(config["type"], "s3");
        assert!(config["secret_access_key"].starts_with(PREFIX));
    }

    #[test]
    fn the_ciphertext_does_not_contain_the_plaintext() {
        let key = key_from(7);
        let mut config = config_with_secret();
        seal_with(&key, &mut config).unwrap();
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(
            !serialized.contains("SUPER-SECRET"),
            "the serialized form still contains the secret: {serialized}"
        );
    }

    #[test]
    fn the_same_secret_seals_differently_each_time() {
        let key = key_from(7);
        let mut a = config_with_secret();
        let mut b = config_with_secret();
        seal_with(&key, &mut a).unwrap();
        seal_with(&key, &mut b).unwrap();
        assert_ne!(
            a["secret_access_key"], b["secret_access_key"],
            "a fresh nonce per value is what stops an observer seeing that two \
             warehouses share a credential"
        );
    }

    #[test]
    fn sealing_twice_does_not_double_encrypt() {
        let key = key_from(7);
        let mut config = config_with_secret();
        seal_with(&key, &mut config).unwrap();
        let once = config["secret_access_key"].clone();
        seal_with(&key, &mut config).unwrap();
        assert_eq!(config["secret_access_key"], once);
        open_with(&key, &mut config).unwrap();
        assert_eq!(config["secret_access_key"], "SUPER-SECRET");
    }

    #[test]
    fn legacy_plaintext_is_readable() {
        // A row written before encryption existed.
        let key = key_from(7);
        let mut config = config_with_secret();
        open_with(&key, &mut config).unwrap();
        assert_eq!(
            config["secret_access_key"], "SUPER-SECRET",
            "refusing to read pre-encryption rows would turn this into an outage"
        );
    }

    #[test]
    #[serial_test::serial(encryption_key_env)]
    fn without_a_key_sealing_is_a_no_op() {
        let _guard = KeyGuard::unset();
        let mut config = config_with_secret();
        seal(&mut config).unwrap();
        assert_eq!(config["secret_access_key"], "SUPER-SECRET");
        assert!(!is_enabled());
    }

    #[test]
    #[serial_test::serial(encryption_key_env)]
    fn sealed_data_without_the_key_fails_loudly() {
        let mut config = config_with_secret();
        seal_with(&key_from(7), &mut config).unwrap();
        let _guard = KeyGuard::unset();
        let err = open(&mut config).unwrap_err().to_string();
        assert!(
            err.contains("PANGOLIN_ENCRYPTION_KEY"),
            "the error must name the missing key, not fail obscurely: {err}"
        );
    }

    #[test]
    fn the_wrong_key_does_not_silently_return_rubbish() {
        let mut config = config_with_secret();
        seal_with(&key_from(7), &mut config).unwrap();
        let err = open_with(&key_from(9), &mut config)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not the key this row was written with"),
            "GCM authentication must reject the wrong key with a usable message: {err}"
        );
    }

    #[test]
    #[serial_test::serial(encryption_key_env)]
    fn a_malformed_key_is_rejected_with_guidance() {
        let _guard = KeyGuard::set("not-base64!!");
        let err = data_key().unwrap_err().to_string();
        assert!(err.contains("openssl rand -base64 32"), "got: {err}");

        let _short = KeyGuard::set(&base64::engine::general_purpose::STANDARD.encode([1u8; 16]));
        let err = data_key().unwrap_err().to_string();
        assert!(err.contains("needs 32"), "got: {err}");
    }

    #[test]
    fn every_credential_the_ui_writes_is_covered() {
        // These are the exact keys pangolin_ui's create-warehouse form writes.
        // A spelling missing from SENSITIVE_KEYS means that credential stays in
        // plaintext with no error anywhere.
        for key in [
            "secret_access_key",
            "access_key_id",
            "account_key",
            "client_secret",
            "service_account_json",
            "external_id",
        ] {
            assert!(
                is_sensitive(key),
                "{key} is a credential and must be sealed"
            );
        }
        for key in [
            "type",
            "bucket",
            "region",
            "endpoint",
            "container",
            "account_name",
            "project_id",
        ] {
            assert!(
                !is_sensitive(key),
                "{key} is not a secret and must stay readable"
            );
        }
    }

    #[test]
    fn plaintext_detection_reports_what_needs_resealing() {
        let key = key_from(7);
        let mut config = config_with_secret();
        assert!(has_plaintext_secret(&config));
        seal_with(&key, &mut config).unwrap();
        assert!(!has_plaintext_secret(&config));
    }
}
