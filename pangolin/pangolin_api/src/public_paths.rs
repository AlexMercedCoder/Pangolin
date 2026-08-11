//! Exact matching for endpoints that may be reached without authentication.
//!
//! The previous implementation whitelisted any request whose path merely
//! *ended with* `/config`, and any path *containing* `/oauth/tokens`. Because
//! catalogs, namespaces and tables are user-named, a namespace literally called
//! `config` produced `/v1/{prefix}/namespaces/config`, which ended in `/config`
//! and therefore skipped authentication entirely — including its DELETE route.
//!
//! Matching here is segment-structural: a path is public only if its whole
//! shape matches a known public route, so a user-chosen resource name can never
//! widen the whitelist.

/// Split a URI path into non-empty segments.
fn segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

/// Returns true if `path` is one of the endpoints that must be reachable
/// without credentials.
///
/// The Iceberg REST spec requires `GET /v1/config` (and, under a prefix,
/// `GET /v1/{prefix}/config`) to be callable before a client has a token,
/// and the OAuth token endpoint is by definition unauthenticated.
pub fn is_public_path(path: &str) -> bool {
    let s = segments(path);

    match s.as_slice() {
        // Liveness / readiness / metrics and the login surface.
        ["health"]
        | ["health", "live"]
        | ["health", "ready"]
        | ["metrics"]
        | ["api", "v1", "users", "login"]
        | ["api", "v1", "app-config"] => true,

        // API documentation.
        ["swagger-ui", ..] | ["api-docs", ..] => true,

        // Iceberg catalog configuration: /v1/config, /v1/{prefix}/config and
        // the doubled-prefix variant PyIceberg sometimes emits.
        ["v1", "config"] => true,
        ["v1", _prefix, "config"] => true,
        ["v1", _prefix, "v1", "config"] => true,

        // Iceberg OAuth token endpoint, in all four routed spellings.
        ["v1", _prefix, "oauth", "tokens"] => true,
        ["v1", _prefix, "v1", "oauth", "tokens"] => true,
        ["api", "v1", "iceberg", _prefix, "oauth", "tokens"] => true,
        ["api", "v1", "iceberg", _prefix, "v1", "oauth", "tokens"] => true,

        // Interactive OAuth entry points. Exactly three segments: the provider
        // is the last one and cannot introduce further path structure.
        ["oauth", "authorize", _provider] => true,
        ["oauth", "callback", _provider] => true,

        // Redeeming the one-time code the OAuth callback hands back.
        //
        // B0k: this was missing, so the middleware demanded a bearer token on
        // the very endpoint whose job is to obtain the first one. The 0.6.0
        // callback -> one-time code -> POST exchange flow was therefore
        // unreachable in production: the browser landed with a `?code=...` it
        // could never redeem. The code itself is single-use and short-lived,
        // which is what makes this endpoint safe to expose unauthenticated.
        ["api", "v1", "oauth", "exchange"] => true,

        // Which OAuth providers are configured. The login page needs this
        // before anyone is authenticated (see B33); it reveals only provider
        // names, never secrets.
        ["api", "v1", "oauth", "providers"] => true,

        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genuine_public_routes_are_public() {
        for path in [
            "/health",
            "/health/live",
            "/health/ready",
            "/metrics",
            "/v1/config",
            "/v1/my_catalog/config",
            "/v1/my_catalog/v1/config",
            "/v1/my_catalog/oauth/tokens",
            "/v1/my_catalog/v1/oauth/tokens",
            "/api/v1/iceberg/my_catalog/oauth/tokens",
            "/api/v1/iceberg/my_catalog/v1/oauth/tokens",
            "/oauth/authorize/google",
            "/oauth/callback/github",
            "/api/v1/users/login",
            "/api/v1/app-config",
            "/swagger-ui",
            "/swagger-ui/index.html",
            "/api-docs/openapi.json",
        ] {
            assert!(is_public_path(path), "{path} should be public");
        }
    }

    /// Regression test for B0k: without this the OAuth login flow cannot
    /// complete, because the code-exchange endpoint demanded the token it
    /// exists to issue.
    #[test]
    fn oauth_exchange_and_providers_are_public() {
        assert!(is_public_path("/api/v1/oauth/exchange"));
        assert!(is_public_path("/api/v1/oauth/providers"));
        // ...but nothing deeper under the same prefix.
        assert!(!is_public_path("/api/v1/oauth/exchange/steal"));
        assert!(!is_public_path("/api/v1/oauth/tokens"));
    }

    /// Regression test for A-11: a resource named `config` must not bypass auth.
    #[test]
    fn resources_named_config_are_not_public() {
        for path in [
            "/v1/my_catalog/namespaces/config",
            "/v1/my_catalog/namespaces/sales/tables/config",
            "/v1/my_catalog/namespaces/config/tables/orders",
            "/api/v1/catalogs/config",
            "/api/v1/warehouses/config",
            "/v1/my_catalog/namespaces/sales/tables/config/metrics",
        ] {
            assert!(
                !is_public_path(path),
                "{path} must require authentication (A-11)"
            );
        }
    }

    /// Regression test for A-11: `contains("/oauth/tokens")` was looser still.
    #[test]
    fn nested_oauth_tokens_paths_are_not_public() {
        for path in [
            "/api/v1/catalogs/x/oauth/tokens/steal",
            "/v1/p/namespaces/oauth/tables/tokens",
            "/api/v1/users/oauth/tokens",
            "/v1/p/namespaces/ns/tables/t/oauth/tokens",
        ] {
            assert!(
                !is_public_path(path),
                "{path} must require authentication (A-11)"
            );
        }
    }

    #[test]
    fn ordinary_authenticated_routes_are_not_public() {
        for path in [
            "/api/v1/tenants",
            "/api/v1/users",
            "/api/v1/warehouses/wh/credentials",
            "/v1/my_catalog/namespaces",
            "/v1/my_catalog/namespaces/sales/tables/orders",
            "/",
        ] {
            assert!(!is_public_path(path), "{path} must require authentication");
        }
    }

    #[test]
    fn trailing_and_duplicate_slashes_do_not_confuse_matching() {
        assert!(is_public_path("/health/"));
        assert!(is_public_path("//v1//config"));
        assert!(!is_public_path("/v1/p/namespaces/config/"));
    }
}
