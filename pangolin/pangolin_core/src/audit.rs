use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
}

impl AuditLogEntry {
    pub fn new(tenant_id: Uuid, actor: String, action: String, resource: String, details: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            timestamp: Utc::now(),
            actor,
            action,
            resource,
            details,
        }
    }
}
