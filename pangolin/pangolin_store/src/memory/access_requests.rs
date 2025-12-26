use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::business_metadata::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_access_request_internal(&self, request: AccessRequest) -> Result<()> {
            self.access_requests.insert(request.id, request);
            Ok(())
        }
    pub(crate) async fn get_access_request_internal(&self, id: Uuid) -> Result<Option<AccessRequest>> {
            Ok(self.access_requests.get(&id).map(|r| r.value().clone()))
        }
    pub(crate) async fn list_access_requests_internal(&self, tenant_id: Uuid) -> Result<Vec<AccessRequest>> {
            let mut requests = Vec::new();
            // Efficient scan filtering by tenant_id directly
            for req in self.access_requests.iter() {
                if req.value().tenant_id == tenant_id {
                    requests.push(req.value().clone());
                }
            }
            Ok(requests)
        }
}
