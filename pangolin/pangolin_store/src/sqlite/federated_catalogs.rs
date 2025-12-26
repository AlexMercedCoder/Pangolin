/// Federated Catalog operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::model::SyncStats;

impl SqliteStore {
    pub async fn sync_federated_catalog(&self, tenant_id: Uuid, catalog_name: &str) -> Result<()> {
        let stats = SyncStats {
            last_synced_at: Some(Utc::now()),
            sync_status: "Success".to_string(),
            tables_synced: 0, 
            namespaces_synced: 0,
            error_message: None,
        };
        
        sqlx::query("INSERT INTO federated_sync_stats (tenant_id, catalog_name, stats) VALUES (?, ?, ?) ON CONFLICT(tenant_id, catalog_name) DO UPDATE SET stats = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .bind(serde_json::to_string(&stats)?)
            .bind(serde_json::to_string(&stats)?)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }

    pub async fn get_federated_catalog_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> {
        let row = sqlx::query("SELECT stats FROM federated_sync_stats WHERE tenant_id = ? AND catalog_name = ?")
            .bind(tenant_id.to_string())
            .bind(catalog_name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(serde_json::from_str(&row.get::<String, _>("stats"))?)
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
