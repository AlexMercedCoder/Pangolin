//! Typed, validated application configuration.
//!
//! Before this module existed, 39 distinct environment variables were read from
//! 88 scattered `std::env::var` call sites, several of them on every request,
//! and none of them validated at startup. The most dangerous consequence was a
//! working fallback for `PANGOLIN_JWT_SECRET` (`"default_secret_for_dev"`),
//! which is published in this repository and therefore lets anyone forge a
//! `Root` token against a deployment that forgot one environment variable.
//!
//! The rules this module enforces:
//!
//! * The server binary refuses to start without a strong `PANGOLIN_JWT_SECRET`
//!   unless `PANGOLIN_DEV_MODE=true` is set explicitly, in which case a random
//!   ephemeral secret is generated and a loud warning is logged.
//! * Known placeholder secrets (`change-me-please`, `default_secret_for_dev`,
//!   …) are rejected outright, so the Helm chart's old defaults cannot be used.
//! * Nothing has a silently-insecure default.

use std::sync::OnceLock;
use std::time::Duration;

/// Secrets that must never be accepted, because they have shipped as defaults
/// in this repository, its Helm chart, or its documentation at some point.
pub const KNOWN_WEAK_SECRETS: &[&str] = &[
    "default_secret_for_dev",
    "change-me-please",
    "change-me",
    "changeme",
    "password",
    "password123",
    "secret",
    "test_secret",
    "your-secret-key",
    "supersecret",
];

/// Minimum accepted length for a production JWT signing secret.
pub const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(
        "PANGOLIN_JWT_SECRET is not set. Generate one with `openssl rand -base64 48` and set it \
         in the environment. For local experimentation only, set PANGOLIN_DEV_MODE=true to run \
         with a random ephemeral secret (all sessions are invalidated on restart)."
    )]
    MissingJwtSecret,

    #[error(
        "PANGOLIN_JWT_SECRET is a known placeholder value ({0:?}). It is published in public \
         source and would let anyone forge a Root token. Generate a real secret with \
         `openssl rand -base64 48`."
    )]
    WeakJwtSecret(String),

    #[error(
        "PANGOLIN_JWT_SECRET is too short ({0} bytes); at least {min} bytes are required.",
        min = MIN_JWT_SECRET_LEN
    )]
    ShortJwtSecret(usize),

    #[error(
        "PANGOLIN_ROOT_PASSWORD is a known placeholder value. Set a real password or unset \
         PANGOLIN_ROOT_USER to disable root basic auth entirely."
    )]
    WeakRootPassword,

    #[error(
        "PANGOLIN_NO_AUTH=true disables all authentication and may only be combined with a \
         loopback bind address. Refusing to start with PANGOLIN_BIND_ADDRESS={0}."
    )]
    NoAuthOnPublicBind(String),

    #[error("invalid value for {0}: {1}")]
    Invalid(&'static str, String),
}

/// Fully-resolved server configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// HS256 signing secret for session JWTs.
    pub jwt_secret: String,
    /// True when the secret was generated at random for this process.
    pub jwt_secret_is_ephemeral: bool,
    /// Disables authentication entirely. Development only.
    pub no_auth: bool,
    /// Development mode relaxes secret requirements. Never set in production.
    pub dev_mode: bool,
    /// Root basic-auth username, if root basic auth is enabled.
    pub root_user: Option<String>,
    /// Root basic-auth password. Compared in constant time.
    pub root_password: Option<String>,
    /// Where the UI lives; used as the default OAuth landing page.
    pub frontend_url: String,
    /// Exact URLs an OAuth flow is permitted to hand control back to.
    pub oauth_redirect_allowlist: Vec<String>,
    /// Whether API keys minted before the key-id format are still accepted.
    pub allow_legacy_api_keys: bool,
    /// Whether JWTs carrying no `jti` are still accepted (off by default).
    ///
    /// `Claims.jti` is `Option<String>` "for compatibility", and the middleware
    /// used to skip the revocation check for such tokens - making them
    /// unrevocable for their full lifetime (B0o). They are now rejected unless
    /// an operator opts in for a migration window.
    pub allow_tokens_without_jti: bool,
    /// Bind address and port.
    pub bind_address: String,
    pub port: u16,
    /// Log encoding.
    pub log_format: LogFormat,
    /// Per-request deadline.
    pub request_timeout: Duration,
    /// Maximum accepted request body size, in bytes.
    pub body_limit_bytes: usize,
    /// Maximum number of requests processed concurrently.
    pub concurrency_limit: usize,
    /// How long to let in-flight requests drain after SIGTERM.
    pub shutdown_grace: Duration,
    /// Whether `/metrics` is exposed.
    pub metrics_enabled: bool,
    /// TTL for the in-process warehouse cache. Short by default because the
    /// cache is node-local and holds storage credentials.
    pub warehouse_cache_ttl: Duration,
    /// Session lifetime for newly issued tokens.
    pub session_ttl: Duration,
    /// CORS origins. `None` means "allow any", which is only safe behind a
    /// trusted gateway and is no longer the default in production.
    pub cors_allowed_origins: Option<Vec<String>>,

    /// Failed authentication attempts allowed per window, per source address
    /// and separately per account. 0 disables throttling. C-5: the login
    /// endpoint had no throttle of any kind and was brute-forceable.
    pub auth_rate_limit: u32,
    /// The window those attempts are counted over.
    pub auth_rate_window: Duration,
    /// Honour `X-Forwarded-For` when deriving the client address.
    ///
    /// Off by default, and that default matters: trusting the header when you
    /// are *not* behind a proxy lets a caller set it per request and bypass the
    /// per-address limit entirely.
    pub trust_forwarded_for: bool,
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();
static EPHEMERAL_SECRET: OnceLock<String> = OnceLock::new();

fn env_opt(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => None,
    }
}

fn env_bool(key: &str) -> bool {
    env_opt(key)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "true" | "1" | "yes" | "on"))
        .unwrap_or(false)
}

fn env_parsed<T: std::str::FromStr>(key: &'static str, default: T) -> Result<T, ConfigError> {
    match env_opt(key) {
        Some(v) => v
            .parse::<T>()
            .map_err(|_| ConfigError::Invalid(key, v.clone())),
        None => Ok(default),
    }
}

/// A random secret, stable for the lifetime of this process.
///
/// Used only when no secret is configured and the caller has opted into
/// development mode (or is a test binary embedding the library). Sessions do
/// not survive a restart, which is exactly the intended signal.
fn ephemeral_secret() -> &'static str {
    EPHEMERAL_SECRET.get_or_init(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..48).map(|_| rng.gen::<u8>()).collect();
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    })
}

/// True if `secret` is one of the placeholder values that has shipped publicly.
pub fn is_weak_secret(secret: &str) -> bool {
    let lowered = secret.trim().to_ascii_lowercase();
    KNOWN_WEAK_SECRETS.iter().any(|w| *w == lowered)
}

impl AppConfig {
    /// Load configuration from the environment, validating strictly.
    ///
    /// This is what the server binary calls. It fails rather than falling back
    /// to anything insecure.
    pub fn from_env_strict() -> Result<Self, ConfigError> {
        let dev_mode = env_bool("PANGOLIN_DEV_MODE");
        let no_auth = env_bool("PANGOLIN_NO_AUTH");

        let (jwt_secret, jwt_secret_is_ephemeral) = match env_opt("PANGOLIN_JWT_SECRET") {
            Some(secret) => {
                if is_weak_secret(&secret) {
                    return Err(ConfigError::WeakJwtSecret(secret));
                }
                if secret.len() < MIN_JWT_SECRET_LEN && !dev_mode {
                    return Err(ConfigError::ShortJwtSecret(secret.len()));
                }
                (secret, false)
            }
            None => {
                if !(dev_mode || no_auth) {
                    return Err(ConfigError::MissingJwtSecret);
                }
                (ephemeral_secret().to_string(), true)
            }
        };

        let root_user = env_opt("PANGOLIN_ROOT_USER");
        let root_password = env_opt("PANGOLIN_ROOT_PASSWORD");
        if root_user.is_some() {
            match root_password.as_deref() {
                Some(p) if is_weak_secret(p) && !dev_mode => {
                    return Err(ConfigError::WeakRootPassword)
                }
                _ => {}
            }
        }

        let bind_address =
            env_opt("PANGOLIN_BIND_ADDRESS").unwrap_or_else(|| "0.0.0.0".to_string());
        // B0h: the guard used to read `no_auth && !dev_mode && !is_loopback(..)`.
        // That `!dev_mode` term meant `PANGOLIN_NO_AUTH=true PANGOLIN_DEV_MODE=true`
        // started happily on the default `0.0.0.0` bind and treated every
        // anonymous request as `TenantAdmin` - and those two flags are routinely
        // set together in compose and dev setups, so the escape hatch was the
        // common case. Dev mode relaxes secret strength, never network exposure:
        // if auth is off, the listener must be loopback, unconditionally.
        if no_auth && !is_loopback(&bind_address) {
            return Err(ConfigError::NoAuthOnPublicBind(bind_address));
        }

        let log_format = match env_opt("LOG_FORMAT").as_deref() {
            Some("json") | Some("JSON") => LogFormat::Json,
            Some("pretty") | None => LogFormat::Pretty,
            Some(other) => return Err(ConfigError::Invalid("LOG_FORMAT", other.to_string())),
        };

        let frontend_url =
            env_opt("FRONTEND_URL").unwrap_or_else(|| "http://localhost:5173".to_string());

        // The frontend URL is always an acceptable OAuth landing page; anything
        // else must be listed explicitly.
        let mut oauth_redirect_allowlist: Vec<String> = env_opt("PANGOLIN_OAUTH_REDIRECT_URIS")
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        if !oauth_redirect_allowlist.contains(&frontend_url) {
            oauth_redirect_allowlist.push(frontend_url.clone());
        }

        Ok(Self {
            jwt_secret,
            jwt_secret_is_ephemeral,
            no_auth,
            dev_mode,
            root_user,
            root_password,
            frontend_url,
            oauth_redirect_allowlist,
            allow_legacy_api_keys: env_bool("PANGOLIN_ALLOW_LEGACY_API_KEYS"),
            allow_tokens_without_jti: env_bool("PANGOLIN_ALLOW_TOKENS_WITHOUT_JTI"),
            bind_address,
            port: env_parsed("PORT", 8080u16)?,
            log_format,
            request_timeout: Duration::from_secs(env_parsed("PANGOLIN_REQUEST_TIMEOUT_SECS", 30)?),
            body_limit_bytes: env_parsed("PANGOLIN_MAX_BODY_BYTES", 16 * 1024 * 1024)?,
            concurrency_limit: env_parsed("PANGOLIN_MAX_CONCURRENT_REQUESTS", 512)?,
            shutdown_grace: Duration::from_secs(env_parsed("PANGOLIN_SHUTDOWN_GRACE_SECS", 25)?),
            auth_rate_limit: env_parsed("PANGOLIN_AUTH_RATE_LIMIT", 10u32)?,
            auth_rate_window: Duration::from_secs(env_parsed(
                "PANGOLIN_AUTH_RATE_WINDOW_SECS",
                60,
            )?),
            trust_forwarded_for: env_bool("PANGOLIN_TRUST_FORWARDED_FOR"),
            metrics_enabled: env_opt("PANGOLIN_METRICS_ENABLED")
                .map(|v| matches!(v.to_ascii_lowercase().as_str(), "true" | "1" | "yes"))
                .unwrap_or(true),
            warehouse_cache_ttl: Duration::from_secs(env_parsed(
                "PANGOLIN_WAREHOUSE_CACHE_TTL_SECS",
                5,
            )?),
            session_ttl: Duration::from_secs(env_parsed("PANGOLIN_SESSION_TTL_SECS", 86_400)?),
            cors_allowed_origins: env_opt("PANGOLIN_CORS_ALLOWED_ORIGINS").map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }),
        })
    }

    /// Install this configuration as the process-wide configuration.
    ///
    /// Returns `Err(self)` if configuration was already installed.
    pub fn install(self) -> Result<(), AppConfig> {
        CONFIG.set(self)
    }

    /// The installed process-wide configuration, if the binary installed one.
    pub fn get() -> Option<&'static AppConfig> {
        CONFIG.get()
    }
}

fn is_loopback(addr: &str) -> bool {
    matches!(addr, "127.0.0.1" | "::1" | "localhost")
}

/// The JWT signing secret for this process.
///
/// Prefers the configuration installed at startup. Library and test embedders
/// that never call [`AppConfig::install`] fall back to reading the environment
/// on each call, so that tests which swap `PANGOLIN_JWT_SECRET` mid-run keep
/// working. There is deliberately **no** hardcoded fallback: an unset secret
/// yields a random per-process value, which fails closed rather than allowing
/// forged tokens.
pub fn jwt_secret() -> String {
    if let Some(cfg) = CONFIG.get() {
        return cfg.jwt_secret.clone();
    }
    match env_opt("PANGOLIN_JWT_SECRET") {
        Some(s) => s,
        None => ephemeral_secret().to_string(),
    }
}

/// Whether authentication is disabled process-wide.
pub fn no_auth_enabled() -> bool {
    if let Some(cfg) = CONFIG.get() {
        return cfg.no_auth;
    }
    env_bool("PANGOLIN_NO_AUTH")
}

/// Root basic-auth credentials, if configured.
pub fn root_credentials() -> Option<(String, String)> {
    if let Some(cfg) = CONFIG.get() {
        return match (&cfg.root_user, &cfg.root_password) {
            (Some(u), Some(p)) => Some((u.clone(), p.clone())),
            _ => None,
        };
    }
    match (
        env_opt("PANGOLIN_ROOT_USER"),
        env_opt("PANGOLIN_ROOT_PASSWORD"),
    ) {
        (Some(u), Some(p)) => Some((u, p)),
        _ => None,
    }
}

/// Whether pre-key-id API keys are still honoured (off by default).
pub fn allow_legacy_api_keys() -> bool {
    if let Some(cfg) = CONFIG.get() {
        return cfg.allow_legacy_api_keys;
    }
    env_bool("PANGOLIN_ALLOW_LEGACY_API_KEYS")
}

/// Whether JWTs with no `jti` are still honoured (off by default, see B0o).
pub fn allow_tokens_without_jti() -> bool {
    if let Some(cfg) = CONFIG.get() {
        return cfg.allow_tokens_without_jti;
    }
    env_bool("PANGOLIN_ALLOW_TOKENS_WITHOUT_JTI")
}

/// Constant-time string comparison, for credential checks.
///
/// `==` on `&str` short-circuits on the first differing byte and therefore
/// leaks the length of the matching prefix through timing.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    // Fold the length difference into the result rather than returning early.
    let mut diff = (a.len() ^ b.len()) as u8;
    let n = a.len().max(b.len());
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_default_secrets_are_rejected() {
        assert!(is_weak_secret("default_secret_for_dev"));
        assert!(is_weak_secret("change-me-please"));
        assert!(is_weak_secret("CHANGE-ME-PLEASE"));
        assert!(is_weak_secret("password123"));
        assert!(!is_weak_secret(
            "6Uu0Xf0K1p9nO2rT4vY7wZ3aB5cD8eF1gH4iJ7kL0mN"
        ));
    }

    #[test]
    fn constant_time_eq_matches_semantics_of_eq() {
        assert!(constant_time_eq("hunter2", "hunter2"));
        assert!(!constant_time_eq("hunter2", "hunter3"));
        assert!(!constant_time_eq("hunter2", "hunter22"));
        assert!(!constant_time_eq("", "x"));
        assert!(constant_time_eq("", ""));
    }

    #[test]
    fn ephemeral_secret_is_stable_and_strong() {
        let a = ephemeral_secret();
        let b = ephemeral_secret();
        assert_eq!(a, b, "must be stable within a process");
        assert!(a.len() >= MIN_JWT_SECRET_LEN);
        assert!(!is_weak_secret(a));
    }
}
