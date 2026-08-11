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
    allowlist.contains(&domain)
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
    let Some(state_nonce) = crate::oauth_state::peek_nonce(&crate::config::jwt_secret(), &state)
    else {
        tracing::error!("could not read back the nonce from a state we just issued");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
    };

    // C-2/C-3: PKCE and an OIDC nonce, for providers that are OIDC.
    //
    // The verifier is kept server-side, never in `state`; see `PendingLogin`.
    // A provider with no issuer (GitHub) gets neither, because it has no
    // id_token to bind a nonce into and its token endpoint would reject the
    // extra parameters.
    let issuer = crate::oidc::issuer_for(&provider);
    let mut oidc_params: Vec<(String, String)> = Vec::new();

    if let Some(issuer) = &issuer {
        let discovery = match crate::oidc::discover(issuer).await {
            Ok(d) => Some(d),
            Err(e) => {
                // Discovery is best-effort at authorize time. Failing the login
                // because a metadata document is briefly unreachable would be a
                // worse outcome than proceeding without PKCE - but if the
                // operator has demanded OIDC, proceeding is not acceptable.
                if crate::oidc::require_oidc() {
                    tracing::error!(error = %e, provider = %provider, "OIDC discovery failed and PANGOLIN_OIDC_REQUIRE is set");
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "OIDC discovery failed; cannot start a login while \
                         PANGOLIN_OIDC_REQUIRE is set",
                    )
                        .into_response();
                }
                tracing::warn!(error = %e, provider = %provider, "OIDC discovery failed; continuing without PKCE");
                None
            }
        };

        let (pkce, nonce) = match (crate::oidc::generate_pkce(), crate::oidc::generate_nonce()) {
            (Ok(p), Ok(n)) => (p, n),
            _ => {
                tracing::error!("could not generate PKCE or nonce material");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .into_response();
            }
        };

        if discovery
            .as_ref()
            .map(|d| d.supports_s256_pkce())
            .unwrap_or(false)
        {
            oidc_params.push(("code_challenge".to_string(), pkce.challenge.clone()));
            oidc_params.push(("code_challenge_method".to_string(), "S256".to_string()));
        }
        oidc_params.push(("nonce".to_string(), nonce.clone()));

        crate::oauth_state::remember_login(&state_nonce, pkce.verifier, nonce);
    } else if crate::oidc::require_oidc() {
        tracing::warn!(
            provider = %provider,
            "refused a non-OIDC provider because PANGOLIN_OIDC_REQUIRE is set"
        );
        return (
            StatusCode::BAD_REQUEST,
            "this provider does not support OpenID Connect, and \
             PANGOLIN_OIDC_REQUIRE is set",
        )
            .into_response();
    }

    let auth_url = build_auth_url(&config, &state, &oidc_params);

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

    // 0b. Recover the PKCE verifier and OIDC nonce for this login. Absent for a
    //     non-OIDC provider, and absent if the state was issued before these
    //     existed - both are handled below rather than assumed.
    let pending = crate::oauth_state::take_login(&state_payload.nonce);
    let issuer = crate::oidc::issuer_for(&provider);

    if issuer.is_none() && crate::oidc::require_oidc() {
        tracing::warn!(provider = %provider, "refused a non-OIDC callback because PANGOLIN_OIDC_REQUIRE is set");
        return (
            StatusCode::BAD_REQUEST,
            "this provider does not support OpenID Connect, and \
             PANGOLIN_OIDC_REQUIRE is set",
        )
            .into_response();
    }

    // 1. Exchange code for tokens, presenting the PKCE verifier.
    let tokens = match exchange_code_for_token(
        &config,
        &callback.code,
        pending.as_ref().map(|p| p.pkce_verifier.as_str()),
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to exchange token: {}", e),
            )
                .into_response()
        }
    };
    let access_token = tokens.access_token.clone();

    // 2. Establish who the user is.
    //
    // For an OIDC provider the answer comes from the *signed* id_token, not
    // from the userinfo endpoint. That is the substantive difference between
    // this and what came before: previously identity was whatever an HTTP
    // response said, and any holder of any access token for this client could
    // have elicited it. Now it is an assertion the provider signed, bound to
    // this login by a nonce and to this application by `aud`.
    let mut user_info = match validate_oidc_identity(
        &provider,
        issuer.as_deref(),
        pending.as_ref(),
        tokens.id_token.as_deref(),
        &config.client_id,
    )
    .await
    {
        Ok(Some(claims)) => claims,
        Ok(None) => {
            // Not an OIDC provider. Fall back to userinfo, which is all such a
            // provider offers.
            match fetch_user_info(&config, &access_token).await {
                Ok(info) => info,
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to fetch user info: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, provider = %provider, "id_token validation failed");
            return (
                StatusCode::UNAUTHORIZED,
                format!("OpenID Connect validation failed: {e}"),
            )
                .into_response();
        }
    };

    // An OIDC id_token need not carry `email`; the userinfo endpoint fills the
    // gap. The *subject* still comes from the signed token - only the
    // display-level attributes are topped up here.
    if user_info.email.is_empty() {
        if let Ok(extra) = fetch_user_info(&config, &access_token).await {
            user_info.email = extra.email;
            if user_info.name.is_none() {
                user_info.name = extra.name;
            }
        }
    }

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
#[serde(deny_unknown_fields)]
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

/// The providers this deployment has actually configured.
///
/// B33: the UI called `GET /api/v1/oauth/providers`, which did not exist, so it
/// 404'd - which is why the login page fell back to rendering all four provider
/// buttons unconditionally, including ones the operator had never configured.
/// Clicking those went nowhere.
///
/// This endpoint is public (see `public_paths`): the login page has to render
/// before anyone is authenticated. It reveals only which provider *names* are
/// enabled - never a client id, and never a secret.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct OAuthProvidersResponse {
    pub providers: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/oauth/providers",
    tag = "Authentication",
    responses((status = 200, description = "Configured OAuth providers", body = OAuthProvidersResponse))
)]
pub async fn list_oauth_providers() -> impl IntoResponse {
    let providers = ["google", "microsoft", "github", "okta"]
        .into_iter()
        .filter(|p| get_oauth_config(p).is_some())
        .map(|p| p.to_string())
        .collect();

    (StatusCode::OK, Json(OAuthProvidersResponse { providers }))
}

/// Establish identity from a validated `id_token`, when the provider is OIDC.
///
/// Returns `Ok(None)` for a provider that is not an OIDC provider, which is the
/// signal to fall back to the userinfo endpoint. Every other absence is an
/// error: if a provider *is* OIDC and no id_token arrived, or the login has no
/// remembered nonce, something is wrong and proceeding would mean skipping the
/// validation while appearing to have done it.
async fn validate_oidc_identity(
    provider: &str,
    issuer: Option<&str>,
    pending: Option<&crate::oauth_state::PendingLogin>,
    id_token: Option<&str>,
    client_id: &str,
) -> Result<Option<OAuthUserInfo>, String> {
    let Some(issuer) = issuer else {
        return Ok(None); // not an OIDC provider
    };

    let Some(id_token) = id_token else {
        return Err(format!(
            "{provider} is configured as an OpenID Connect provider but returned \
             no id_token. Check that `openid` is among the requested scopes."
        ));
    };

    let Some(pending) = pending else {
        return Err(
            "this login has no remembered nonce, so the id_token cannot be bound \
             to it. Start the login again."
                .to_string(),
        );
    };

    let discovery = crate::oidc::discover(issuer)
        .await
        .map_err(|e| format!("OIDC discovery failed: {e}"))?;

    let claims =
        crate::oidc::validate_id_token(id_token, &discovery, client_id, &pending.oidc_nonce)
            .await
            .map_err(|e| e.to_string())?;

    tracing::info!(
        provider = %provider,
        subject = %claims.sub,
        "authenticated a user from a validated id_token"
    );

    Ok(Some(OAuthUserInfo {
        sub: claims.sub,
        email: claims.email.unwrap_or_default(),
        name: claims.name,
        email_verified: claims.email_verified,
    }))
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
///
/// `extra` carries the OIDC parameters - `code_challenge`,
/// `code_challenge_method`, `nonce` - which are absent for a provider that is
/// not an OIDC provider.
fn build_auth_url(config: &OAuthConfig, state: &str, extra: &[(String, String)]) -> String {
    let scopes = config.scopes.join(" ");

    let mut url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        config.get_auth_url(),
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&scopes),
        urlencoding::encode(state)
    );

    for (key, value) in extra {
        url.push('&');
        url.push_str(&urlencoding::encode(key));
        url.push('=');
        url.push_str(&urlencoding::encode(value));
    }

    url
}

/// Exchange authorization code for access token
/// What the provider returned from the token endpoint.
pub struct TokenExchange {
    pub access_token: String,
    /// Present only for OIDC providers. GitHub returns none.
    pub id_token: Option<String>,
}

async fn exchange_code_for_token(
    config: &OAuthConfig,
    code: &str,
    pkce_verifier: Option<&str>,
) -> Result<TokenExchange, String> {
    let client = reqwest::Client::new();

    let mut params: Vec<(&str, String)> = vec![
        ("client_id", config.client_id.clone()),
        ("client_secret", config.client_secret.clone()),
        ("code", code.to_string()),
        ("redirect_uri", config.redirect_uri.clone()),
        ("grant_type", "authorization_code".to_string()),
    ];

    // The other half of PKCE. The provider recomputes SHA-256 of this and
    // compares it to the challenge sent at authorize time; a code stolen
    // without the verifier cannot be redeemed.
    if let Some(verifier) = pkce_verifier {
        params.push(("code_verifier", verifier.to_string()));
    }

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
        #[serde(default)]
        id_token: Option<String>,
        // we might handle refresh_token later
    }

    let token_res: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(TokenExchange {
        access_token: token_res.access_token,
        id_token: token_res.id_token,
    })
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
        let url = build_auth_url(&config, &state, &[]);

        assert!(url.contains("client_id=client_id_val"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%2Fcallback"));
        assert!(url.contains("scope="));
        assert!(url.contains("state="));
        // The state must not carry a caller-supplied redirect target (A-8).
        assert!(!url.contains("frontend"));
    }

    #[test]
    fn the_auth_url_carries_pkce_and_nonce_when_supplied() {
        let config = OAuthConfig::google(
            "cid".to_string(),
            "secret".to_string(),
            "http://localhost/callback".to_string(),
        );
        let state =
            crate::oauth_state::issue("a-test-signing-secret-of-adequate-length-0", "google", 0);

        let pkce = crate::oidc::generate_pkce().unwrap();
        let nonce = crate::oidc::generate_nonce().unwrap();
        let url = build_auth_url(
            &config,
            &state,
            &[
                ("code_challenge".to_string(), pkce.challenge.clone()),
                ("code_challenge_method".to_string(), "S256".to_string()),
                ("nonce".to_string(), nonce.clone()),
            ],
        );

        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("nonce="));
        assert!(
            !url.contains(&pkce.verifier),
            "the PKCE *verifier* must never appear in the authorization URL - \
             the whole point is that only its hash travels"
        );
    }

    #[test]
    fn the_pkce_verifier_is_not_recoverable_from_the_state() {
        // `state` travels through the browser in the same URL as the
        // authorization code. If the verifier were in it, anyone able to steal
        // the code would also hold the verifier, and PKCE would protect nothing.
        let secret = "a-test-signing-secret-of-adequate-length-0";
        let state = crate::oauth_state::issue(secret, "google", 0);
        let nonce = crate::oauth_state::peek_nonce(secret, &state).unwrap();

        let pkce = crate::oidc::generate_pkce().unwrap();
        crate::oauth_state::remember_login(&nonce, pkce.verifier.clone(), "n".to_string());

        assert!(
            !state.contains(&pkce.verifier),
            "the verifier leaked into the state parameter"
        );

        // And it is single-use: a replayed callback finds nothing.
        assert!(crate::oauth_state::take_login(&nonce).is_some());
        assert!(
            crate::oauth_state::take_login(&nonce).is_none(),
            "a remembered login must be consumed exactly once"
        );
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
