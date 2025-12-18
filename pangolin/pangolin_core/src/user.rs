use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// User in the system
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    /// Password hash (bcrypt) - None for OAuth users
    pub password_hash: Option<String>,
    /// OAuth provider if using OAuth
    pub oauth_provider: Option<OAuthProvider>,
    /// OAuth subject ID from provider
    pub oauth_subject: Option<String>,
    /// None for root user, Some(tenant_id) for tenant users
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
}

/// User role in the system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    /// Super admin - can manage everything
    Root,
    /// Tenant administrator - can manage within their tenant
    TenantAdmin,
    /// Regular tenant user - has granular permissions
    TenantUser,
}

/// OAuth provider
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Microsoft,
    GitHub,
    Okta,
}

/// User session for JWT or OAuth
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub tenant_id: Option<Uuid>,
    pub role: UserRole,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Service user for API key authentication
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceUser {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Uuid,
    pub api_key_hash: String,  // bcrypt hash of API key
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,  // User who created this service user
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

/// API Key response (only shown once on creation)
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub service_user_id: Uuid,
    pub name: String,
    pub api_key: String,  // Plain text, only shown once
    pub expires_at: Option<DateTime<Utc>>,
}

impl ServiceUser {
    pub fn new(
        name: String,
        description: Option<String>,
        tenant_id: Uuid,
        api_key_hash: String,
        role: UserRole,
        created_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            tenant_id,
            api_key_hash,
            role,
            created_at: Utc::now(),
            created_by,
            last_used: None,
            expires_at,
            active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.active && !self.is_expired()
    }
}

impl User {
    pub fn new_root(username: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash: Some(password_hash),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: None,
            role: UserRole::Root,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            active: true,
        }
    }

    pub fn new_tenant_admin(
        username: String,
        email: String,
        password_hash: String,
        tenant_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash: Some(password_hash),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: Some(tenant_id),
            role: UserRole::TenantAdmin,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            active: true,
        }
    }

    pub fn new_tenant_user(
        username: String,
        email: String,
        password_hash: String,
        tenant_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash: Some(password_hash),
            oauth_provider: None,
            oauth_subject: None,
            tenant_id: Some(tenant_id),
            role: UserRole::TenantUser,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            active: true,
        }
    }

    pub fn new_oauth(
        username: String,
        email: String,
        provider: OAuthProvider,
        subject: String,
        tenant_id: Option<Uuid>,
        role: UserRole,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash: None,
            oauth_provider: Some(provider),
            oauth_subject: Some(subject),
            tenant_id,
            role,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            active: true,
        }
    }

    pub fn is_root(&self) -> bool {
        self.role == UserRole::Root
    }

    pub fn is_tenant_admin(&self) -> bool {
        self.role == UserRole::TenantAdmin
    }

    pub fn is_tenant_user(&self) -> bool {
        self.role == UserRole::TenantUser
    }

    pub fn can_manage_tenant(&self, tenant_id: &Uuid) -> bool {
        self.is_root() || (self.is_tenant_admin() && self.tenant_id.as_ref() == Some(tenant_id))
    }
}
