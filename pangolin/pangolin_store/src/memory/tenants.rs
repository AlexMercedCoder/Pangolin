use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_tenant_internal(&self, tenant: Tenant) -> Result<()> {
            self.tenants.insert(tenant.id, tenant);
            Ok(())
        }
    pub(crate) async fn get_tenant_internal(&self, tenant_id: Uuid) -> Result<Option<Tenant>> {
            if let Some(t) = self.tenants.get(&tenant_id) {
                Ok(Some(t.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn list_tenants_internal(&self, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tenant>> {
            let mut tenants: Vec<Tenant> = self.tenants.iter().map(|t| t.value().clone()).collect();
            tenants.sort_by(|a, b| a.name.cmp(&b.name));
            
            if let Some(p) = pagination {
                let offset = p.offset.unwrap_or(0);
                let limit = p.limit.unwrap_or(usize::MAX);
                
                if offset >= tenants.len() {
                    return Ok(Vec::new());
                }
                
                let end = std::cmp::min(offset + limit, tenants.len());
                Ok(tenants[offset..end].to_vec())
            } else {
                Ok(tenants)
            }
        }
    pub(crate) async fn update_tenant_internal(&self, tenant_id: Uuid, updates: pangolin_core::model::TenantUpdate) -> Result<Tenant> {
            if let Some(mut tenant) = self.tenants.get_mut(&tenant_id) {
                if let Some(name) = updates.name {
                    tenant.name = name;
                }
                if let Some(properties) = updates.properties {
                    tenant.properties.extend(properties);
                }
                Ok(tenant.clone())
            } else {
                Err(anyhow::anyhow!("Tenant not found"))
            }
        }
    pub(crate) async fn delete_tenant_internal(&self, tenant_id: Uuid) -> Result<()> {
            if self.tenants.remove(&tenant_id).is_some() {
                // TODO: Cascade delete warehouses and catalogs
                Ok(())
            } else {
                Err(anyhow::anyhow!("Tenant not found"))
            }
        }
}
