use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::SystemSettings;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // System Settings
    pub async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        let row = sqlx::query("SELECT settings FROM system_settings WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(serde_json::from_value(row.get("settings"))?)
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
        sqlx::query("INSERT INTO system_settings (tenant_id, settings) VALUES ($1, $2) ON CONFLICT(tenant_id) DO UPDATE SET settings = $3")
            .bind(tenant_id)
            .bind(serde_json::to_value(&settings)?)
            .bind(serde_json::to_value(&settings)?)
            .execute(&self.pool)
            .await?;
        Ok(settings)
    }
}
