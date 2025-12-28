use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_core::auth::OAuthConfig;
use pangolin_store::CatalogStore;
use std::sync::Arc;
use crate::auth_middleware::{create_session, generate_token};
use base64::Engine;
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
    let oauth_config = get_oauth_config(&provider);
    
    if oauth_config.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid OAuth provider").into_response();
    }
    
    let config = oauth_config.unwrap();
    
    // Build authorization URL
    let auth_url = build_auth_url(&config, params.redirect_uri);
    
    // Redirect to OAuth provider
    Redirect::to(&auth_url).into_response()
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
    
    // 1. Exchange code for access token
    let access_token = match exchange_code_for_token(&config, &callback.code).await {
        Ok(token) => token,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Failed to exchange token: {}", e)).into_response(),
    };
    
    // 2. Fetch user info from provider
    let user_info = match fetch_user_info(&config, &access_token).await {
        Ok(info) => info,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Failed to fetch user info: {}", e)).into_response(),
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
    // We assume email is unique and can be used to link accounts or create new ones
    // For MVP, we'll create a new user if not found by email, or maybe by oauth_subject
    // Ideally look up by (provider, subject)
    
    // Since CatalogStore doesn't expose `get_user_by_oauth` yet, we'll use `get_user_by_username` as a fallback or iterate
    // But `CatalogStore` trait needs a method for this efficiently.
    // For now, let's list users and filter (inefficient but works for MemoryStore)
    // Or better, let's just stick to email for now if unique.
    
    // Let's implement a rudimentary lookup
    let all_users = store.list_users(None, None).await.unwrap_or_default();
    let existing_user = all_users.into_iter().find(|u| 
        (u.oauth_provider == Some(provider_enum.clone()) && u.oauth_subject == Some(user_info.sub.clone())) ||
        u.email == user_info.email
    );
    
    let user = match existing_user {
        Some(mut u) => {
            // Update last login or details if needed
            // For now just use it
            u
        },
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
            let role = if user_count == 0 { UserRole::Root } else { UserRole::TenantUser };
            
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
                     return (StatusCode::BAD_REQUEST, "No default tenant found for new user").into_response();
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
                 return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create user: {}", e)).into_response();
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
     
    let secret = std::env::var("PANGOLIN_JWT_SECRET").unwrap_or_else(|_| "default_secret_for_dev".to_string());
    let token = match generate_token(session, &secret) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate token: {}", e)).into_response(),
    };

    // 6. Return response (Redirect to frontend with token)
    // Extract redirect_uri from state
    let frontend_url = if let Some(state_str) = callback.state {
        // Try to decode state as JSON
        if let Ok(decoded) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&state_str) {
             if let Ok(state_json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                 state_json.get("redirect_uri").and_then(|s| s.as_str()).map(|s| s.to_string())
             } else { None }
        } else { None }
    } else { None };

    let base_url = frontend_url.unwrap_or_else(|| std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string()));
    
    // Append token to URL (handling query params)
    let redirect_url = if base_url.contains('?') {
        format!("{}&token={}", base_url, token)
    } else {
        format!("{}?token={}", base_url, token)
    };
    
    Redirect::to(&redirect_url).into_response()
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
            Some(OAuthConfig::microsoft(client_id, client_secret, redirect_uri, tenant_id))
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
            Some(OAuthConfig::okta(client_id, client_secret, redirect_uri, domain))
        }
        _ => None,
    }
}

/// Build OAuth authorization URL
fn build_auth_url(config: &OAuthConfig, client_redirect: Option<String>) -> String {
    let scopes = config.scopes.join(" ");
    
    // Create state JSON with nonce and redirect_uri
    let state_data = serde_json::json!({
        "nonce": Uuid::new_v4().to_string(),
        "redirect_uri": client_redirect
    });
    
    let state = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_data.to_string());
    
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        config.get_auth_url(),
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&scopes),
        state
    )
}

/// Exchange authorization code for access token
async fn exchange_code_for_token(
    config: &OAuthConfig,
    code: &str,
) -> Result<String, String> {
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
            "http://localhost/callback".to_string()
        );
        let url = build_auth_url(&config, Some("http://frontend/home".to_string()));
         
        assert!(url.contains("client_id=client_id_val"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%2Fcallback"));
        assert!(url.contains("scope="));
        assert!(url.contains("state="));
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
