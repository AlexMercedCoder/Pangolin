/// System Settings operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::SystemSettings;

impl SqliteStore {
    pub async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        let row = sqlx::query("SELECT settings FROM system_settings WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(serde_json::from_str(&row.get::<String, _>("settings"))?)
        } else {
            Ok(SystemSettings {
                allow_public_signup: None,
                default_warehouse_bucket: None,
                default_retention_days: None,
                smtp_host: None,
                smtp_port: None,
                smtp_user: None,
                smtp_password: None,
            })
        }
    }

    pub async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> {
        // Upsert
        sqlx::query("INSERT INTO system_settings (tenant_id, settings) VALUES (?, ?) ON CONFLICT(tenant_id) DO UPDATE SET settings = ?")
            .bind(tenant_id.to_string())
            .bind(serde_json::to_string(&settings)?)
            .bind(serde_json::to_string(&settings)?)
            .execute(&self.pool)
            .await?;
        Ok(settings)
    }
}
