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
pub struct Catalog {
    pub name: String,
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
