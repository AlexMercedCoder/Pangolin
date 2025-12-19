use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenInfo {
    pub id: Uuid, // JTI
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub is_valid: bool, // Checked against revocation list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RevokedToken {
    pub id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl RevokedToken {
    pub fn new(id: Uuid, expires_at: DateTime<Utc>, reason: Option<String>) -> Self {
        Self { id, expires_at, reason }
    }
    
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
