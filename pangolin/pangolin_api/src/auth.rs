use chrono::DateTime;
use pangolin_core::user::{UserRole, UserSession};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,         // user_id
    pub jti: Option<String>, // JWT ID for token revocation (optional for backward compatibility)
    pub username: String,
    pub tenant_id: Option<String>,
    pub role: UserRole,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
}

impl From<UserSession> for Claims {
    fn from(session: UserSession) -> Self {
        Self {
            sub: session.user_id.to_string(),
            jti: Some(uuid::Uuid::new_v4().to_string()),
            username: session.username,
            tenant_id: session.tenant_id.map(|id| id.to_string()),
            role: session.role,
            exp: session.expires_at.timestamp(),
            iat: session.issued_at.timestamp(),
        }
    }
}

impl Claims {
    pub fn to_session(&self) -> Result<UserSession, String> {
        Ok(UserSession {
            user_id: Uuid::parse_str(&self.sub).map_err(|e| e.to_string())?,
            username: self.username.clone(),
            tenant_id: self
                .tenant_id
                .as_ref()
                .map(|id| Uuid::parse_str(id))
                .transpose()
                .map_err(|e| e.to_string())?,
            role: self.role.clone(),
            issued_at: DateTime::from_timestamp(self.iat, 0)
                .ok_or("Invalid issued_at timestamp")?,
            expires_at: DateTime::from_timestamp(self.exp, 0)
                .ok_or("Invalid expires_at timestamp")?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RootUser;
