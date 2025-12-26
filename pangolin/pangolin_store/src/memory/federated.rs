use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn get_federated_catalog_stats_internal(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> {
        if let Some(stats) = self.federated_stats.get(&(tenant_id, catalog_name.to_string())) {
            Ok(stats.value().clone())
        } else {
            Ok(SyncStats {
                last_synced_at: None,
                sync_status: "Never Synced".to_string(),
                tables_synced: 0,
                namespaces_synced: 0,
                error_message: None,
            })
        }
    }
}
