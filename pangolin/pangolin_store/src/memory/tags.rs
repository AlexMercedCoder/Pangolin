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
    pub(crate) async fn list_tags_internal(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
            let mut tags = Vec::new();
            for r in self.tags.iter() {
                let (tid, cname, _) = r.key();
                if *tid == tenant_id && cname == catalog_name {
                    tags.push(r.value().clone());
                }
            }
            Ok(tags)
        }
    pub(crate) async fn delete_tag_internal(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
            let key = (tenant_id, catalog_name.to_string(), name);
            self.tags.remove(&key);
            Ok(())
        }
}
