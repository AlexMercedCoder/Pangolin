use serde::{Deserialize, Serialize};

/// OAuth configuration for different providers
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OAuthConfig {
    pub provider: OAuthProviderConfig,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    /// Additional scopes to request
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum OAuthProviderConfig {
    Google {
        /// Google OAuth endpoints
        auth_url: String,
        token_url: String,
        userinfo_url: String,
    },
    Microsoft {
        /// Microsoft OAuth endpoints
        tenant_id: String,
        auth_url: String,
        token_url: String,
        userinfo_url: String,
    },
    GitHub {
        /// GitHub OAuth endpoints
        auth_url: String,
        token_url: String,
        userinfo_url: String,
    },
    Okta {
        /// Okta OAuth endpoints
        domain: String,
        auth_url: String,
        token_url: String,
        userinfo_url: String,
    },
}

impl OAuthConfig {
    /// Create Google OAuth config
    pub fn google(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            provider: OAuthProviderConfig::Google {
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            },
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        }
    }

    /// Create Microsoft OAuth config
    pub fn microsoft(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        tenant_id: String,
    ) -> Self {
        Self {
            provider: OAuthProviderConfig::Microsoft {
                tenant_id: tenant_id.clone(),
                auth_url: format!(
                    "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                    tenant_id
                ),
                token_url: format!(
                    "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                    tenant_id
                ),
                userinfo_url: "https://graph.microsoft.com/v1.0/me".to_string(),
            },
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        }
    }

    /// Create GitHub OAuth config
    pub fn github(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            provider: OAuthProviderConfig::GitHub {
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                userinfo_url: "https://api.github.com/user".to_string(),
            },
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec!["read:user".to_string(), "user:email".to_string()],
        }
    }

    /// Create Okta OAuth config
    pub fn okta(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        domain: String,
    ) -> Self {
        Self {
            provider: OAuthProviderConfig::Okta {
                domain: domain.clone(),
                auth_url: format!("https://{}/oauth2/v1/authorize", domain),
                token_url: format!("https://{}/oauth2/v1/token", domain),
                userinfo_url: format!("https://{}/oauth2/v1/userinfo", domain),
            },
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        }
    }

    pub fn get_auth_url(&self) -> &str {
        match &self.provider {
            OAuthProviderConfig::Google { auth_url, .. } => auth_url,
            OAuthProviderConfig::Microsoft { auth_url, .. } => auth_url,
            OAuthProviderConfig::GitHub { auth_url, .. } => auth_url,
            OAuthProviderConfig::Okta { auth_url, .. } => auth_url,
        }
    }

    pub fn get_token_url(&self) -> &str {
        match &self.provider {
            OAuthProviderConfig::Google { token_url, .. } => token_url,
            OAuthProviderConfig::Microsoft { token_url, .. } => token_url,
            OAuthProviderConfig::GitHub { token_url, .. } => token_url,
            OAuthProviderConfig::Okta { token_url, .. } => token_url,
        }
    }

    pub fn get_userinfo_url(&self) -> &str {
        match &self.provider {
            OAuthProviderConfig::Google { userinfo_url, .. } => userinfo_url,
            OAuthProviderConfig::Microsoft { userinfo_url, .. } => userinfo_url,
            OAuthProviderConfig::GitHub { userinfo_url, .. } => userinfo_url,
            OAuthProviderConfig::Okta { userinfo_url, .. } => userinfo_url,
        }
    }
}

/// Authentication mode
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMode {
    /// No authentication (development only)
    NoAuth,
    /// JWT-based authentication
    Jwt,
    /// OAuth-based authentication
    OAuth,
}

/// Authentication configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct AuthConfig {
    pub mode: AuthMode,
    /// JWT secret for signing tokens (required for JWT mode)
    pub jwt_secret: Option<String>,
    /// JWT expiration in seconds (default: 24 hours)
    pub jwt_expiration_secs: Option<u64>,
    /// OAuth configuration (required for OAuth mode)
    pub oauth: Option<OAuthConfig>,
}

impl AuthConfig {
    pub fn no_auth() -> Self {
        Self {
            mode: AuthMode::NoAuth,
            jwt_secret: None,
            jwt_expiration_secs: None,
            oauth: None,
        }
    }

    pub fn jwt(secret: String, expiration_secs: Option<u64>) -> Self {
        Self {
            mode: AuthMode::Jwt,
            jwt_secret: Some(secret),
            jwt_expiration_secs: expiration_secs.or(Some(86400)), // 24 hours default
            oauth: None,
        }
    }

    pub fn oauth(oauth_config: OAuthConfig) -> Self {
        Self {
            mode: AuthMode::OAuth,
            jwt_secret: None,
            jwt_expiration_secs: None,
            oauth: Some(oauth_config),
        }
    }
}
