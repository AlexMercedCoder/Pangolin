use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

/// Comprehensive audit log entry for tracking all catalog operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct AuditLogEntry {
    /// Unique identifier for this audit log entry
    pub id: Uuid,

    /// Tenant that owns the resource being operated on
    pub tenant_id: Uuid,

    /// User who performed the action (None for system operations)
    pub user_id: Option<Uuid>,

    /// Username for readability (actor field for backward compatibility)
    pub username: String,

    /// Type of action performed
    pub action: AuditAction,

    /// Type of resource being operated on
    pub resource_type: ResourceType,

    /// UUID of the specific resource (if applicable)
    pub resource_id: Option<Uuid>,

    /// Human-readable resource name/path
    pub resource_name: String,

    /// When the action occurred (UTC)
    pub timestamp: DateTime<Utc>,

    /// Client IP address (if available)
    pub ip_address: Option<String>,

    /// Client user agent (if available)
    pub user_agent: Option<String>,

    /// Whether the operation succeeded or failed
    pub result: AuditResult,

    /// Error message if the operation failed
    pub error_message: Option<String>,

    /// Additional structured metadata about the operation
    pub metadata: Option<serde_json::Value>,
}

/// Types of actions that can be audited
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Catalog Operations
    CreateCatalog,
    UpdateCatalog,
    DeleteCatalog,

    // Namespace Operations
    CreateNamespace,
    UpdateNamespace,
    DeleteNamespace,

    // Table Operations
    CreateTable,
    UpdateTable,
    DropTable,
    RenameTable,
    CommitTable,
    LoadTable,

    // View Operations
    CreateView,
    UpdateView,
    DropView,
    RenameView,
    LoadView,

    // Branch Operations
    CreateBranch,
    DeleteBranch,
    MergeBranch,

    // User Management
    CreateUser,
    UpdateUser,
    DeleteUser,
    Login,
    Logout,

    // Service Users
    CreateServiceUser,
    UpdateServiceUser,
    DeleteServiceUser,
    RotateApiKey,

    // Permissions
    GrantPermission,
    RevokePermission,

    // Federated Catalogs
    CreateFederatedCatalog,
    UpdateFederatedCatalog,
    DeleteFederatedCatalog,
    TestFederatedConnection,

    // Warehouses
    CreateWarehouse,
    UpdateWarehouse,
    DeleteWarehouse,

    // Tenants
    CreateTenant,
    UpdateTenant,
    DeleteTenant,

    // Tags
    CreateTag,
    DeleteTag,

    // Merge Operations
    InitiateMerge,
    CompleteMerge,
    AbortMerge,
    ResolveConflict,
    ListConflicts,

    // Business Metadata
    AddMetadata,
    UpdateMetadata,
    DeleteMetadata,
    SearchAssets,

    // Access Requests
    RequestAccess,
    ApproveAccess,
    DenyAccess,
    UpdateAccessRequest,

    // Tokens
    GenerateToken,
    RevokeToken,

    // Commits
    CreateCommit,
    ListCommits,

    // Authentication and authorization events.
    //
    // Auth events are the first thing an incident responder looks for, and
    // none of them were recorded before (C-19).
    LoginFailed,
    ApiKeyAuthenticated,
    ApiKeyRejected,
    TokenRejected,
    PermissionDenied,
    /// A Root session assumed another tenant's context via X-Pangolin-Tenant.
    /// Privileged cross-tenant access is exactly what auditors want logged (C-7).
    TenantImpersonation,
}

/// Types of resources that can be operated on
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Catalog,
    Namespace,
    Table,
    View,
    Branch,
    Tag,
    User,
    ServiceUser,
    Permission,
    Role,
    FederatedCatalog,
    Warehouse,
    Tenant,
    MergeOperation,
    Conflict,
    AccessRequest,
    Metadata,
    Token,
    Commit,
}

/// Result of an audited operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
}

/// Serialize an audit enum to the spelling that goes into storage.
///
/// B22: the SQLite backend persisted `format!("{:?}", action)` - the Debug
/// spelling, `"CreateBranch"` - and then read it back by lowercasing to
/// `"createbranch"` and deserializing against serde's snake_case
/// (`"create_branch"`). Nothing matched, and the result was
/// `.unwrap_or(AuditAction::CreateCatalog)`, so nearly every multi-word action
/// in the SQLite audit trail was recorded as `CreateCatalog`. The audit log is
/// the one artefact that has to be right after an incident.
///
/// Both directions now go through serde, so the write and read spellings cannot
/// drift apart again.
pub fn audit_enum_to_stored<T: Serialize + std::fmt::Debug>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{:?}", value))
}

/// Parse an audit enum from its stored spelling.
///
/// Accepts the canonical serde name and, for rows written before B22 was fixed,
/// the legacy Debug name. An unrecognised value is an error rather than a silent
/// substitution.
pub fn audit_enum_from_stored<T>(stored: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    // Canonical snake_case.
    if let Ok(v) = serde_json::from_value::<T>(serde_json::Value::String(stored.to_string())) {
        return Ok(v);
    }

    // Legacy Debug spelling: "CreateBranch" -> "create_branch".
    let mut snake = String::with_capacity(stored.len() + 4);
    for (i, ch) in stored.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        } else {
            snake.push(ch);
        }
    }
    serde_json::from_value::<T>(serde_json::Value::String(snake))
        .map_err(|_| format!("unknown audit enum value {stored:?}"))
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct AuditLogFilter {
    /// Filter by user who performed the action
    pub user_id: Option<Uuid>,

    /// Filter by action type
    pub action: Option<AuditAction>,

    /// Filter by resource type
    pub resource_type: Option<ResourceType>,

    /// Filter by specific resource ID
    pub resource_id: Option<Uuid>,

    /// Filter by start time (inclusive)
    pub start_time: Option<DateTime<Utc>>,

    /// Filter by end time (inclusive)
    pub end_time: Option<DateTime<Utc>>,

    /// Filter by result (success/failure)
    pub result: Option<AuditResult>,

    /// Maximum number of results to return
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

impl AuditLogEntry {
    /// Create a new audit log entry with required fields
    pub fn new(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        username: String,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
        resource_name: String,
        result: AuditResult,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            username,
            action,
            resource_type,
            resource_id,
            resource_name,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            result,
            error_message: None,
            metadata: None,
        }
    }

    /// Add request context (IP address and user agent)
    pub fn with_context(mut self, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    /// Add error message for failed operations
    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self
    }

    /// Add structured metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Create a success audit log entry
    pub fn success(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        username: String,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
        resource_name: String,
    ) -> Self {
        Self::new(
            tenant_id,
            user_id,
            username,
            action,
            resource_type,
            resource_id,
            resource_name,
            AuditResult::Success,
        )
    }

    /// Create a failure audit log entry
    pub fn failure(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        username: String,
        action: AuditAction,
        resource_type: ResourceType,
        resource_name: String,
        error: String,
    ) -> Self {
        Self::new(
            tenant_id,
            user_id,
            username,
            action,
            resource_type,
            None,
            resource_name,
            AuditResult::Failure,
        )
        .with_error(error)
    }
}

impl Default for AuditLogFilter {
    fn default() -> Self {
        Self {
            user_id: None,
            action: None,
            resource_type: None,
            resource_id: None,
            start_time: None,
            end_time: None,
            result: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::success(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "test_user".to_string(),
            AuditAction::CreateTable,
            ResourceType::Table,
            Some(Uuid::new_v4()),
            "analytics.sales.transactions".to_string(),
        );

        assert_eq!(entry.result, AuditResult::Success);
        assert_eq!(entry.action, AuditAction::CreateTable);
        assert_eq!(entry.resource_type, ResourceType::Table);
    }

    #[test]
    fn test_audit_log_entry_with_context() {
        let entry = AuditLogEntry::success(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "test_user".to_string(),
            AuditAction::CreateTable,
            ResourceType::Table,
            Some(Uuid::new_v4()),
            "test_table".to_string(),
        )
        .with_context(
            Some("192.168.1.100".to_string()),
            Some("PyIceberg/0.5.0".to_string()),
        );

        assert_eq!(entry.ip_address, Some("192.168.1.100".to_string()));
        assert_eq!(entry.user_agent, Some("PyIceberg/0.5.0".to_string()));
    }

    #[test]
    fn test_audit_log_entry_failure() {
        let entry = AuditLogEntry::failure(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "test_user".to_string(),
            AuditAction::DropTable,
            ResourceType::Table,
            "analytics.sales.transactions".to_string(),
            "Table not found".to_string(),
        );

        assert_eq!(entry.result, AuditResult::Failure);
        assert_eq!(entry.error_message, Some("Table not found".to_string()));
    }

    #[test]
    fn test_audit_log_filter_default() {
        let filter = AuditLogFilter::default();
        assert_eq!(filter.limit, Some(100));
        assert_eq!(filter.offset, Some(0));
        assert!(filter.user_id.is_none());
    }
}
