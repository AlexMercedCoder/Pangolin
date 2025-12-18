use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VendingStrategy {
    /// AWS S3 with STS temporary credentials
    AwsSts {
        role_arn: String,
        external_id: Option<String>,
    },
    /// AWS S3 with static credentials
    AwsStatic {
        access_key_id: String,
        secret_access_key: String,
    },
    /// Azure Blob Storage with SAS tokens
    AzureSas {
        account_name: String,
        account_key: String,
    },
    /// GCP with downscoped credentials
    GcpDownscoped {
        service_account_email: String,
        private_key: String,
    },
    /// No credential vending (client-provided only)
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Warehouse {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub storage_config: std::collections::HashMap<String, String>,
    pub use_sts: bool, // Deprecated in favor of vending_strategy, kept for backward compatibility
    pub vending_strategy: Option<VendingStrategy>,
}

// Federated Catalog Support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum CatalogType {
    Local,      // Native Pangolin catalog
    Federated,  // External Iceberg REST catalog (proxy)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum FederatedAuthType {
    None,           // No authentication required
    BasicAuth,      // Username/password
    BearerToken,    // JWT token
    ApiKey,         // X-API-Key header
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederatedCredentials {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederatedCatalogConfig {
    pub base_url: String,
    pub auth_type: FederatedAuthType,
    pub credentials: Option<FederatedCredentials>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Catalog {
    pub id: Uuid, // Added ID for permission scoping
    pub name: String,
    pub catalog_type: CatalogType, // Local or Federated
    pub warehouse_name: Option<String>, // Reference to warehouse for credential vending (Local only)
    pub storage_location: Option<String>, // Base path for this catalog in the warehouse (Local only)
    pub federated_config: Option<FederatedCatalogConfig>, // Configuration for federated catalogs
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub name: Vec<String>,
    pub properties: HashMap<String, String>,
}

impl Namespace {
    pub fn new(name: Vec<String>) -> Self {
        Self {
            name,
            properties: HashMap::new(),
        }
    }
    
    pub fn to_string(&self) -> String {
        self.name.join(".")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetType {
    IcebergTable,
    DeltaTable,
    HudiTable,
    ParquetTable,
    CsvTable,
    JsonTable,
    View,
    MlModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub kind: AssetType,
    pub location: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub timestamp: i64,
    pub author: String,
    pub message: String,
    pub operations: Vec<CommitOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommitOperation {
    Put { asset: Asset },
    Delete { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchType {
    Ingest,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub head_commit_id: Option<Uuid>,
    pub branch_type: BranchType,
    pub assets: Vec<String>, // List of asset names tracked by this branch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub commit_id: Uuid,
}

// Merge Conflict Resolution Models

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ConflictType {
    SchemaChange {
        asset_name: String,
        source_schema: serde_json::Value,
        target_schema: serde_json::Value,
    },
    DataOverlap {
        asset_name: String,
        overlapping_partitions: Vec<String>,
    },
    MetadataConflict {
        asset_name: String,
        conflicting_properties: Vec<String>,
    },
    DeletionConflict {
        asset_name: String,
        deleted_in: String, // "source" or "target"
        modified_in: String, // "source" or "target"
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ResolutionStrategy {
    AutoMerge,          // Automatically merge non-conflicting changes
    TakeSource,         // Use source branch version
    TakeTarget,         // Use target branch version
    Manual,             // Requires manual resolution
    ThreeWayMerge,      // Merge using base commit as reference
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictResolution {
    pub conflict_id: Uuid,
    pub strategy: ResolutionStrategy,
    pub resolved_value: Option<serde_json::Value>,
    pub resolved_by: Uuid,
    pub resolved_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MergeConflict {
    pub id: Uuid,
    pub merge_operation_id: Uuid,
    pub conflict_type: ConflictType,
    pub asset_id: Option<Uuid>,
    pub description: String,
    pub resolution: Option<ConflictResolution>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MergeConflict {
    pub fn new(
        merge_operation_id: Uuid,
        conflict_type: ConflictType,
        asset_id: Option<Uuid>,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            merge_operation_id,
            conflict_type,
            asset_id,
            description,
            resolution: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum MergeStatus {
    Pending,        // Merge initiated, conflicts being detected
    Conflicted,     // Conflicts detected, awaiting resolution
    Resolving,      // Manual resolution in progress
    Ready,          // All conflicts resolved, ready to complete
    Completed,      // Merge successfully completed
    Aborted,        // Merge aborted by user
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MergeOperation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub catalog_name: String,
    pub source_branch: String,
    pub target_branch: String,
    pub base_commit_id: Option<Uuid>, // Common ancestor for 3-way merge
    pub status: MergeStatus,
    pub conflicts: Vec<Uuid>, // IDs of MergeConflict records
    pub initiated_by: Uuid,
    pub initiated_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result_commit_id: Option<Uuid>, // Commit ID if merge completed
}

impl MergeOperation {
    pub fn new(
        tenant_id: Uuid,
        catalog_name: String,
        source_branch: String,
        target_branch: String,
        base_commit_id: Option<Uuid>,
        initiated_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            catalog_name,
            source_branch,
            target_branch,
            base_commit_id,
            status: MergeStatus::Pending,
            conflicts: Vec::new(),
            initiated_by,
            initiated_at: chrono::Utc::now(),
            completed_at: None,
            result_commit_id: None,
        }
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn can_complete(&self) -> bool {
        self.status == MergeStatus::Ready || (self.status == MergeStatus::Pending && !self.has_conflicts())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_serialization() {
        let asset = Asset {
            id: Uuid::new_v4(),
            name: "test_view".to_string(),
            kind: AssetType::View,
            location: "s3://bucket/path".to_string(),
            properties: HashMap::new(),
        };
        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: Asset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset.name, deserialized.name);
        assert!(matches!(deserialized.kind, AssetType::View));
    }

    #[test]
    fn test_tenant_serialization() {
        let tenant = Tenant {
            id: Uuid::new_v4(),
            name: "acme".to_string(),
            properties: HashMap::new(),
        };
        let json = serde_json::to_string(&tenant).unwrap();
        let deserialized: Tenant = serde_json::from_str(&json).unwrap();
        assert_eq!(tenant.name, deserialized.name);
    }
}

// Update structs for CRUD operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantUpdate {
    pub name: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WarehouseUpdate {
    pub name: Option<String>,
    pub storage_config: Option<HashMap<String, String>>,
    pub use_sts: Option<bool>,
    pub vending_strategy: Option<VendingStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogUpdate {
    pub warehouse_name: Option<String>,
    pub storage_location: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

