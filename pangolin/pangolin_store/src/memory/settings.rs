use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn get_system_settings_internal(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        if let Some(s) = self.system_settings.get(&tenant_id) {
            Ok(s.value().clone())
        } else {
             // Return default settings if not found
             Ok(SystemSettings::default())
        }
    }
    
    pub(crate) async fn update_system_settings_internal(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> {
        self.system_settings.insert(tenant_id, settings.clone());
        Ok(settings)
    }
}
