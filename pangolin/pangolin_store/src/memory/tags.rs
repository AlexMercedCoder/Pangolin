use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_tag_internal(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
            let key = (tenant_id, catalog_name.to_string(), tag.name.clone());
            self.tags.insert(key, tag);
            Ok(())
        }
    pub(crate) async fn get_tag_internal(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
            let key = (tenant_id, catalog_name.to_string(), name);
            if let Some(tag) = self.tags.get(&key) {
                Ok(Some(tag.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn list_tags_internal(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tag>> {
            let iter = self.tags.iter()
                .filter(|r| {
                    let (tid, cname, _) = r.key();
                    *tid == tenant_id && cname == catalog_name
                })
                .map(|r| r.value().clone());

            let tags = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(tags)
        }
    pub(crate) async fn delete_tag_internal(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
            let key = (tenant_id, catalog_name.to_string(), name);
            self.tags.remove(&key);
            Ok(())
        }
}
