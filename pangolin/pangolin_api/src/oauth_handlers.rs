use crate::auth_middleware::{create_session, generate_token};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use pangolin_core::auth::OAuthConfig;
use pangolin_core::user::{OAuthProvider, User, UserRole};
use pangolin_store::CatalogStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// OAuth callback query parameters
#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthCallback {
    pub code: String,
    pub state: Option<String>,
}

/// OAuth user info from provider
#[derive(Debug, Deserialize)]
pub struct OAuthUserInfo {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    /// Whether the provider asserts the address has been verified.
    ///
    /// Absent on providers that do not report it - notably GitHub's
    /// `/user` endpoint - which is why an absent value is treated as
    /// *unverified* (B0l).
    #[serde(default)]
    pub email_verified: Option<bool>,
}

impl OAuthUserInfo {
    fn email_is_verified(&self) -> bool {
        self.email_verified.unwrap_or(false)
    }
}

/// Domains whose *verified* addresses may link to a pre-existing local account.
///
/// Empty by default: with no allowlist configured, email never links an account,
/// and only a `(provider, subject)` match does.
fn email_link_domain_allowlist() -> Vec<String> {
    std::env::var("PANGOLIN_OAUTH_EMAIL_LINK_DOMAINS")
        .ok()
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// May `email` be used to adopt an existing local account?
fn email_may_link(user_info: &OAuthUserInfo, allowlist: &[String]) -> bool {
    if !user_info.email_is_verified() {
        return false;
    }
    let Some(domain) = user_info
        .email
        .rsplit_once('@')
        .map(|(_, d)| d.to_ascii_lowercase())
    else {
        return false;
    };
    allowlist.iter().any(|allowed| *allowed == domain)
}

#[derive(Deserialize, ToSchema)]
pub struct AuthorizeParams {
    pub redirect_uri: Option<String>,
}

/// Initiate OAuth flow
#[utoipa::path(
    get,
    path = "/api/v1/oauth/{provider}/authorize",
    tag = "OAuth",
    params(
        ("provider" = String, Path, description = "OAuth provider (google, microsoft, github, okta)")
    ),
    responses(
        (status = 302, description = "Redirect to OAuth provider"),
        (status = 400, description = "Invalid provider")
    )
)]
pub async fn oauth_authorize(
    State(_store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(provider): Path<String>,
    Query(params): Query<AuthorizeParams>,
) -> Response {
    let Some(config) = get_oauth_config(&provider) else {
        return (StatusCode::BAD_REQUEST, "Invalid OAuth provider").into_response();
    };

    // The client may only ask to be returned to a URL the operator has
    // allowlisted. Resolving it to an index here means the URL itself never
    // travels inside `state`, so it cannot be tampered with (A-8).
    let allowlist = redirect_allowlist();
    let redirect_index = match crate::oauth_state::resolve_redirect(
        params.redirect_uri.as_deref(),
        &allowlist,
    ) {
        Ok(idx) => idx,
        Err(e) => {
            tracing::warn!(error = %e, "rejected OAuth authorize with a non-allowlisted redirect_uri");
            return (
                StatusCode::BAD_REQUEST,
                "redirect_uri is not allowed; add it to PANGOLIN_OAUTH_REDIRECT_URIS",
            )
                .into_response();
        }
    };

    // Signed, single-use state (A-9).
    let state = crate::oauth_state::issue(&crate::config::jwt_secret(), &provider, redirect_index);
    let auth_url = build_auth_url(&config, &state);

    Redirect::to(&auth_url).into_response()
}

/// The URLs an OAuth flow may hand control back to.
///
/// Index 0 is always the configured frontend URL, so a request that asks for no
/// particular redirect lands somewhere sane.
fn redirect_allowlist() -> Vec<String> {
    if let Some(cfg) = crate::config::AppConfig::get() {
        return cfg.oauth_redirect_allowlist.clone();
    }
    let frontend =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let mut list: Vec<String> = std::env::var("PANGOLIN_OAUTH_REDIRECT_URIS")
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if !list.contains(&frontend) {
        list.insert(0, frontend);
    }
    list
}

/// OAuth callback handler
#[utoipa::path(
    get,
    path = "/api/v1/oauth/{provider}/callback",
    tag = "OAuth",
    params(
        ("provider" = String, Path, description = "OAuth provider (google, microsoft, github, okta)")
    ),
    responses(
        (status = 302, description = "Redirect to frontend with token"),
        (status = 400, description = "Invalid provider or callback failed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn oauth_callback(
    State(store): State<Arc<dyn CatalogStore + Send + Sync>>,
    Path(provider): Path<String>,
    Query(callback): Query<OAuthCallback>,
) -> Response {
    let oauth_config = get_oauth_config(&provider);

    if oauth_config.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid OAuth provider").into_response();
    }

    let config = oauth_config.unwrap();

    // 0. Verify `state` before spending anything on the provider round-trip.
    //    The nonce is single-use, so a captured callback URL cannot be replayed,
    //    and the signature means an attacker cannot author one (A-9).
    let state_payload = match crate::oauth_state::verify_and_consume(
        &crate::config::jwt_secret(),
        &provider,
        callback.state.as_deref(),
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, provider = %provider, "rejected OAuth callback with invalid state");
            return (StatusCode::BAD_REQUEST, "Invalid or expired OAuth state").into_response();
        }
    };

    // 1. Exchange code for access token
    let access_token = match exchange_code_for_token(&config, &callback.code).await {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to exchange token: {}", e),
            )
                .into_response()
        }
    };

    // 2. Fetch user info from provider
    let user_info = match fetch_user_info(&config, &access_token).await {
        Ok(info) => info,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to fetch user info: {}", e),
            )
                .into_response()
        }
    };

    // 3. Map provider string to Enum
    let provider_enum = match provider.as_str() {
        "google" => OAuthProvider::Google,
        "microsoft" => OAuthProvider::Microsoft,
        "github" => OAuthProvider::GitHub,
        "okta" => OAuthProvider::Okta,
        _ => return (StatusCode::BAD_REQUEST, "Unknown provider").into_response(),
    };

    // 4. Find or Create User
    //
    // B0l: the match used to include `|| u.email == user_info.email`, with no
    // `email_verified` check and no provider binding. Anyone who could set a
    // matching address on *any* configured provider - GitHub happily reports
    // unverified addresses - logged in as that Pangolin user, including the
    // seeded `TenantAdmin`. Identity is `(provider, subject)`; an address is
    // only allowed to adopt a pre-existing account when the provider says it is
    // verified *and* its domain is one the operator listed.
    let all_users = store.list_users(None, None).await.unwrap_or_default();
    let allowlist = email_link_domain_allowlist();
    let may_link_by_email = email_may_link(&user_info, &allowlist);

    let existing_user = all_users.into_iter().find(|u| {
        let subject_match = u.oauth_provider == Some(provider_enum.clone())
            && u.oauth_subject == Some(user_info.sub.clone());
        let email_match = may_link_by_email && u.email == user_info.email;
        subject_match || email_match
    });

    if !may_link_by_email && !allowlist.is_empty() && !user_info.email_is_verified() {
        tracing::warn!(
            provider = %provider,
            "OAuth provider reported an unverified email; not linking by address"
        );
    }

    let user = match existing_user {
        Some(u) => {
            // Update last login or details if needed
            // For now just use it
            u
        }
        None => {
            // Create new user
            // Default to TenantUser for now, or Root if it's the very first user?
            // Let's safe default to TenantUser, but we need a tenant ID.
            // If we support auto-provisioning, we might need a default tenant or create one.
            // For now, let's error if no tenant context, OR create a "personal" tenant for them?
            // To keep it simple: First user ever -> Root. Others -> Error (must be invited) OR Pending.
            // Actually, for this MVP, let's just create them as a TenantUser in a specific "default" tenant if it exists,
            // or just stand-alone if we support users without tenants (Root users).

            // Checking if any users exist to decide if Root
            let user_count = store.list_users(None, None).await.unwrap_or_default().len();
            let role = if user_count == 0 {
                UserRole::Root
            } else {
                UserRole::TenantUser
            };

            // We need a tenant if not root.
            let tenant_id = if role == UserRole::Root {
                None
            } else {
                // Try to find a default tenant "default"
                let tenants = store.list_tenants(None).await.unwrap_or_default();
                if let Some(t) = tenants.into_iter().find(|t| t.name == "default") {
                    Some(t.id)
                } else {
                    // Create a default tenant? Or fail.
                    // Let's fail for now to enforce setup.
                    return (
                        StatusCode::BAD_REQUEST,
                        "No default tenant found for new user",
                    )
                        .into_response();
                }
            };

            let new_user = User::new_oauth(
                user_info.email.clone(), // Use email as username for OAuth?
                user_info.email.clone(),
                provider_enum,
                user_info.sub,
                tenant_id,
                role,
            );

            if let Err(e) = store.create_user(new_user.clone()).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create user: {}", e),
                )
                    .into_response();
            }
            new_user
        }
    };

    // 5. Create Session & Token
    let session = create_session(
        user.id,
        user.username.clone(),
        user.tenant_id,
        user.role.clone(),
        86400, // 24 hours
    );

    let secret = crate::config::jwt_secret();
    let token = match generate_token(session, &secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "failed to generate session token after OAuth callback");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate session token",
            )
                .into_response();
        }
    };

    // 6. Hand control back to the client.
    //
    // The session token is never placed in the redirect URL. A URL travels
    // through browser history, `Referer` headers, proxy logs and the target
    // host's access log, and the old code took the destination straight out of
    // an unsigned `state` parameter, so an attacker-supplied host received a
    // valid token for the victim (A-8). Instead the token is parked
    // server-side and the client redeems a short-lived, single-use code over
    // POST /api/v1/oauth/exchange.
    let allowlist = redirect_allowlist();
    let base_url = allowlist
        .get(state_payload.redirect_index)
        .cloned()
        .unwrap_or_else(|| {
            allowlist
                .first()
                .cloned()
                .unwrap_or_else(|| "http://localhost:5173".to_string())
        });

    let code = crate::oauth_state::stash_token(token);
    let separator = if base_url.contains('?') { '&' } else { '?' };
    let redirect_url = format!("{base_url}{separator}code={code}");

    Redirect::to(&redirect_url).into_response()
}

/// Request body for exchanging a one-time OAuth code for a session token.
#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthExchangeRequest {
    pub code: String,
}

/// Response for a successful code exchange.
#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthExchangeResponse {
    pub token: String,
    pub token_type: String,
}

/// Exchange a one-time OAuth code for the session token it stands for.
///
/// The code is single-use and short-lived, and the token is returned in a
/// response body rather than a URL so it is never logged in transit.
#[utoipa::path(
    post,
    path = "/api/v1/oauth/exchange",
    tag = "OAuth",
    request_body = OAuthExchangeRequest,
    responses(
        (status = 200, description = "Session token", body = OAuthExchangeResponse),
        (status = 400, description = "Unknown, expired or already-redeemed code")
    )
)]
pub async fn oauth_exchange(Json(req): Json<OAuthExchangeRequest>) -> Response {
    match crate::oauth_state::redeem_code(&req.code) {
        Some(token) => (
            StatusCode::OK,
            Json(OAuthExchangeResponse {
                token,
                token_type: "Bearer".to_string(),
            }),
        )
            .into_response(),
        None => (
            StatusCode::BAD_REQUEST,
            "Unknown, expired or already-redeemed exchange code",
        )
            .into_response(),
    }
}

/// Get OAuth configuration for provider
fn get_oauth_config(provider: &str) -> Option<OAuthConfig> {
    // TODO: Load from environment variables or config file
    match provider {
        "google" => {
            let client_id = std::env::var("PANGOLIN_GOOGLE_CLIENT_ID").ok()?;
            let client_secret = std::env::var("PANGOLIN_GOOGLE_CLIENT_SECRET").ok()?;
            let redirect_uri = std::env::var("PANGOLIN_GOOGLE_REDIRECT_URI").ok()?;
            Some(OAuthConfig::google(client_id, client_secret, redirect_uri))
        }
        "microsoft" => {
            let client_id = std::env::var("PANGOLIN_MICROSOFT_CLIENT_ID").ok()?;
            let client_secret = std::env::var("PANGOLIN_MICROSOFT_CLIENT_SECRET").ok()?;
            let redirect_uri = std::env::var("PANGOLIN_MICROSOFT_REDIRECT_URI").ok()?;
            let tenant_id = std::env::var("PANGOLIN_MICROSOFT_TENANT_ID").ok()?;
            Some(OAuthConfig::microsoft(
                client_id,
                client_secret,
                redirect_uri,
                tenant_id,
            ))
        }
        "github" => {
            let client_id = std::env::var("PANGOLIN_GITHUB_CLIENT_ID").ok()?;
            let client_secret = std::env::var("PANGOLIN_GITHUB_CLIENT_SECRET").ok()?;
            let redirect_uri = std::env::var("PANGOLIN_GITHUB_REDIRECT_URI").ok()?;
            Some(OAuthConfig::github(client_id, client_secret, redirect_uri))
        }
        "okta" => {
            let client_id = std::env::var("PANGOLIN_OKTA_CLIENT_ID").ok()?;
            let client_secret = std::env::var("PANGOLIN_OKTA_CLIENT_SECRET").ok()?;
            let redirect_uri = std::env::var("PANGOLIN_OKTA_REDIRECT_URI").ok()?;
            let domain = std::env::var("PANGOLIN_OKTA_DOMAIN").ok()?;
            Some(OAuthConfig::okta(
                client_id,
                client_secret,
                redirect_uri,
                domain,
            ))
        }
        _ => None,
    }
}

/// Build the provider authorization URL around an already-signed `state`.
fn build_auth_url(config: &OAuthConfig, state: &str) -> String {
    let scopes = config.scopes.join(" ");

    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        config.get_auth_url(),
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&scopes),
        urlencoding::encode(state)
    )
}

/// Exchange authorization code for access token
async fn exchange_code_for_token(config: &OAuthConfig, code: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
        ("code", &code.to_string()),
        ("redirect_uri", &config.redirect_uri),
        ("grant_type", &"authorization_code".to_string()),
    ];

    let response = client
        .post(config.get_token_url())
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Token request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token request failed: {}", error_text));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        // we might handle refresh_token later
    }

    let token_res: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(token_res.access_token)
}

/// Fetch user info from OAuth provider
async fn fetch_user_info(
    config: &OAuthConfig,
    access_token: &str,
) -> Result<OAuthUserInfo, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(config.get_userinfo_url())
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "Pangolin-Catalog") // GitHub requires User-Agent
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("User info request failed: {}", error_text));
    }

    // Provider-specific response handling could be done here if schemas diverge significantly
    // For now, we assume standard OIDC fields or map them in the struct
    let user_info: OAuthUserInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    Ok(user_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_auth_url_builder() {
        let config = OAuthConfig::google(
            "client_id_val".to_string(),
            "client_secret_val".to_string(),
            "http://localhost/callback".to_string(),
        );
        let state =
            crate::oauth_state::issue("a-test-signing-secret-of-adequate-length-0", "google", 0);
        let url = build_auth_url(&config, &state);

        assert!(url.contains("client_id=client_id_val"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%2Fcallback"));
        assert!(url.contains("scope="));
        assert!(url.contains("state="));
        // The state must not carry a caller-supplied redirect target (A-8).
        assert!(!url.contains("frontend"));
    }

    /// Regression test for A-8: the authorize endpoint refuses a redirect
    /// target the operator has not allowlisted, instead of echoing it back.
    #[test]
    fn redirect_uri_outside_the_allowlist_is_rejected() {
        let allowlist = redirect_allowlist();
        assert!(
            crate::oauth_state::resolve_redirect(Some("https://evil.example/"), &allowlist)
                .is_err()
        );
    }

    #[test]
    fn test_get_oauth_config_from_env() {
        // Set env vars
        std::env::set_var("PANGOLIN_GOOGLE_CLIENT_ID", "test_id");
        std::env::set_var("PANGOLIN_GOOGLE_CLIENT_SECRET", "test_secret");
        std::env::set_var("PANGOLIN_GOOGLE_REDIRECT_URI", "test_uri");

        let config = get_oauth_config("google").expect("Should return config");
        assert_eq!(config.client_id, "test_id");
        assert_eq!(config.client_secret, "test_secret");
        assert_eq!(config.redirect_uri, "test_uri");

        std::env::remove_var("PANGOLIN_GOOGLE_CLIENT_ID");
        std::env::remove_var("PANGOLIN_GOOGLE_CLIENT_SECRET");
        std::env::remove_var("PANGOLIN_GOOGLE_REDIRECT_URI");
    }
}
