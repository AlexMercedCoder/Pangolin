use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub storage_config: std::collections::HashMap<String, String>,
    pub use_sts: bool, // If true, vend STS credentials; if false, pass through static creds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub name: String,
    pub warehouse_name: Option<String>, // Reference to warehouse for credential vending
    pub storage_location: Option<String>, // Base path for this catalog in the warehouse
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_serialization() {
        let asset = Asset {
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
