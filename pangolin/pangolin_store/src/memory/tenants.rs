use super::MemoryStore;
use anyhow::Result;
use pangolin_core::model::*;
use uuid::Uuid;

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
    pub(crate) async fn list_tenants_internal(
        &self,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Tenant>> {
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
    pub(crate) async fn update_tenant_internal(
        &self,
        tenant_id: Uuid,
        updates: pangolin_core::model::TenantUpdate,
    ) -> Result<Tenant> {
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
    /// Delete a tenant and everything scoped to it.
    ///
    /// B30: the cascade was a `// TODO`. Warehouses (with their cloud
    /// credentials), catalogs, namespaces, assets, branches, tags, audit
    /// history, permissions and cached tokens all survived tenant deletion on
    /// the memory backend - so a "deleted" tenant's storage credentials were
    /// still vendable, and a recreated tenant with the same id inherited the old
    /// one's data. The retain-based pattern here is the one `delete_catalog`
    /// already used.
    pub(crate) async fn delete_tenant_internal(&self, tenant_id: Uuid) -> Result<()> {
        if self.tenants.remove(&tenant_id).is_none() {
            return Err(anyhow::anyhow!("Tenant not found"));
        }

        // Keyed by (tenant, ..) - drop everything whose first key element is
        // this tenant.
        self.warehouses.retain(|k, _| k.0 != tenant_id);
        self.catalogs.retain(|k, _| k.0 != tenant_id);
        self.namespaces.retain(|k, _| k.0 != tenant_id);
        self.assets.retain(|k, _| k.0 != tenant_id);
        self.branches.retain(|k, _| k.0 != tenant_id);
        self.tags.retain(|k, _| k.0 != tenant_id);
        self.commits.retain(|k, _| k.0 != tenant_id);
        self.federated_stats.retain(|k, _| k.0 != tenant_id);

        // Keyed by their own id, with the tenant in the value.
        self.assets_by_id.retain(|_, v| v.0 != tenant_id);
        self.audit_events.remove(&tenant_id);
        self.system_settings.remove(&tenant_id);
        self.users.retain(|_, u| u.tenant_id != Some(tenant_id));
        self.roles.retain(|_, r| r.tenant_id != tenant_id);
        self.permissions.retain(|_, p| p.tenant_id != tenant_id);
        self.service_users.retain(|_, s| s.tenant_id != tenant_id);
        self.active_tokens.retain(|_, t| t.tenant_id != tenant_id);

        Ok(())
    }
}
