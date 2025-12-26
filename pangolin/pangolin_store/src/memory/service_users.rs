use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::user::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_service_user_internal(&self, service_user: pangolin_core::user::ServiceUser) -> Result<()> {
            self.service_users.insert(service_user.id, service_user);
            Ok(())
        }
    pub(crate) async fn get_service_user_internal(&self, id: Uuid) -> Result<Option<pangolin_core::user::ServiceUser>> {
            Ok(self.service_users.get(&id).map(|r| r.value().clone()))
        }
    pub(crate) async fn list_service_users_internal(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::user::ServiceUser>> {
            Ok(self.service_users
                .iter()
                .filter(|entry| entry.value().tenant_id == tenant_id)
                .map(|entry| entry.value().clone())
                .collect())
        }
    pub(crate) async fn update_service_user_internal(
            &self,
            id: Uuid,
            name: Option<String>,
            description: Option<String>,
            active: Option<bool>,
        ) -> Result<()> {
            if let Some(mut service_user) = self.service_users.get_mut(&id) {
                if let Some(n) = name {
                    service_user.name = n;
                }
                if let Some(d) = description {
                    service_user.description = Some(d);
                }
                if let Some(a) = active {
                    service_user.active = a;
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Service user not found"))
            }
        }
    pub(crate) async fn delete_service_user_internal(&self, id: Uuid) -> Result<()> {
            self.service_users.remove(&id);
            Ok(())
        }
}
