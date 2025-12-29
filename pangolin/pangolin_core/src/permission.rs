use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashSet;
use utoipa::ToSchema;

/// Permission scope - where the permission applies
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum PermissionScope {
    /// Applies to entire catalog
    Catalog { catalog_id: Uuid },
    /// Applies to namespace and all assets within
    Namespace { catalog_id: Uuid, namespace: String },
    /// Applies to specific asset
    Asset { catalog_id: Uuid, namespace: String, asset_id: Uuid },
    /// Applies to all assets with specific tag
    Tag { tag_name: String },
    /// Applies to everything in the tenant
    Tenant,
}

impl PermissionScope {
    pub fn covers(&self, other: &PermissionScope) -> bool {
        match (self, other) {
            // Exact match
            (a, b) if a == b => true,

            // Tenant covers everything
            (PermissionScope::Tenant, _) => true,
            
            // Catalog covers everything in it
            (PermissionScope::Catalog { catalog_id: id1 }, PermissionScope::Catalog { catalog_id: id2 }) => id1 == id2,
            (PermissionScope::Catalog { catalog_id: id1 }, PermissionScope::Namespace { catalog_id: id2, .. }) => id1 == id2,
            (PermissionScope::Catalog { catalog_id: id1 }, PermissionScope::Asset { catalog_id: id2, .. }) => id1 == id2,
            
            // Namespace covers assets in it (and sub-namespaces?)
            (PermissionScope::Namespace { catalog_id: id1, namespace: ns1 }, PermissionScope::Namespace { catalog_id: id2, namespace: ns2 }) => {
                id1 == id2 && (ns1 == ns2 || ns2.starts_with(&format!("{}.", ns1)))
            },
            (PermissionScope::Namespace { catalog_id: id1, namespace: ns1 }, PermissionScope::Asset { catalog_id: id2, namespace: ns2, .. }) => {
                id1 == id2 && (ns1 == ns2 || ns2.starts_with(&format!("{}.", ns1)))
            },
            
            _ => false,
        }
    }
}

/// Actions that can be performed
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Read,
    Write,
    Delete,
    Create,
    Update,
    List,
    All,
    /// Can create ingest branches (can be merged back)
    IngestBranching,
    /// Can create experimental branches (cannot be merged back)
    ExperimentalBranching,
    /// Can mark assets as discoverable in a catalog
    ManageDiscovery,
}

impl Action {
    /// Check if this action implies another action
    pub fn implies(&self, other: &Action) -> bool {
        if self == other { return true; }
        
        match self {
            Action::All => true,
            Action::Write => matches!(other, Action::Read | Action::Update | Action::Delete | Action::Create | Action::List),
            Action::Read => matches!(other, Action::List),
            _ => false,
        }
    }
}

/// A single permission grant
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Permission {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(default)]
    pub tenant_id: Uuid,
    pub scope: PermissionScope,
    pub actions: HashSet<Action>,
    pub granted_by: Uuid,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

impl Permission {
    pub fn new(user_id: Uuid, tenant_id: Uuid, scope: PermissionScope, actions: HashSet<Action>, granted_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            tenant_id,
            scope,
            actions,
            granted_by,
            granted_at: chrono::Utc::now(),
        }
    }

    /// Check if this permission allows a specific action
    pub fn allows(&self, action: &Action) -> bool {
        self.actions.iter().any(|a| a.implies(action))
    }
}

/// A role that groups permissions
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Uuid,
    pub permissions: Vec<PermissionGrant>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Permission grant within a role
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PermissionGrant {
    pub scope: PermissionScope,
    pub actions: HashSet<Action>,
}

impl Role {
    pub fn new(name: String, description: Option<String>, tenant_id: Uuid, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            tenant_id,
            permissions: Vec::new(),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn add_permission(&mut self, scope: PermissionScope, actions: HashSet<Action>) {
        self.permissions.push(PermissionGrant { scope, actions });
        self.updated_at = chrono::Utc::now();
    }
}

/// User-Role assignment
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

impl UserRole {
    pub fn new(user_id: Uuid, role_id: Uuid, assigned_by: Uuid) -> Self {
        Self {
            user_id,
            role_id,
            assigned_by,
            assigned_at: chrono::Utc::now(),
        }
    }
}
