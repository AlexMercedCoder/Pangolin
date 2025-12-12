use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaintenanceJob {
    pub id: Uuid,
    pub job_type: MaintenanceType,
    pub status: JobStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MaintenanceType {
    ExpireSnapshots { retention_ms: i64 },
    RemoveOrphanFiles { older_than_ms: i64 },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}
