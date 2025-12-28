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
    pub(crate) async fn list_access_requests_internal(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<AccessRequest>> {
            let iter = self.access_requests.iter()
                .filter(|req| req.value().tenant_id == tenant_id)
                .map(|req| req.value().clone());

            let requests = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(requests)
        }
}
