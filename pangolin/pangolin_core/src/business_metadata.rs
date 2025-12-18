use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// Business metadata attached to assets
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct BusinessMetadata {
    pub id: Uuid,
    pub asset_id: Uuid,
    /// Markdown description
    pub description: Option<String>,
    /// Tags/labels for categorization
    pub tags: Vec<String>,
    /// Custom key-value properties
    pub properties: HashMap<String, String>,
    /// Whether this asset appears in search for users without access
    pub discoverable: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}

impl BusinessMetadata {
    pub fn new(asset_id: Uuid, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            description: None,
            tags: Vec::new(),
            properties: HashMap::new(),
            discoverable: false,
            created_by,
            created_at: Utc::now(),
            updated_by: created_by,
            updated_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_discoverable(mut self, discoverable: bool) -> Self {
        self.discoverable = discoverable;
        self
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
        self.updated_at = Utc::now();
    }

    pub fn remove_property(&mut self, key: &str) {
        self.properties.remove(key);
        self.updated_at = Utc::now();
    }

    pub fn update(&mut self, updated_by: Uuid) {
        self.updated_by = updated_by;
        self.updated_at = Utc::now();
    }
}

/// Access request for discoverable assets
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AccessRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub reason: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub status: RequestStatus,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_comment: Option<String>,
}

/// Status of an access request
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RequestStatus {
    Pending,
    Approved,
    Rejected,
}

impl AccessRequest {
    pub fn new(tenant_id: Uuid, user_id: Uuid, asset_id: Uuid, reason: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            asset_id,
            reason,
            requested_at: Utc::now(),
            status: RequestStatus::Pending,
            reviewed_by: None,
            reviewed_at: None,
            review_comment: None,
        }
    }

    pub fn approve(&mut self, reviewed_by: Uuid, comment: Option<String>) {
        self.status = RequestStatus::Approved;
        self.reviewed_by = Some(reviewed_by);
        self.reviewed_at = Some(Utc::now());
        self.review_comment = comment;
    }

    pub fn reject(&mut self, reviewed_by: Uuid, comment: Option<String>) {
        self.status = RequestStatus::Rejected;
        self.reviewed_by = Some(reviewed_by);
        self.reviewed_at = Some(Utc::now());
        self.review_comment = comment;
    }

    pub fn is_pending(&self) -> bool {
        self.status == RequestStatus::Pending
    }
}
