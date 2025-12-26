use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::business_metadata::*;
use pangolin_core::model::{Asset, Catalog, Namespace, Branch};
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn upsert_business_metadata_internal(&self, metadata: pangolin_core::business_metadata::BusinessMetadata) -> Result<()> {
            self.business_metadata.insert(metadata.asset_id, metadata);
            Ok(())
        }
    pub(crate) async fn get_business_metadata_internal(&self, asset_id: Uuid) -> Result<Option<pangolin_core::business_metadata::BusinessMetadata>> {
            Ok(self.business_metadata.get(&asset_id).map(|m| m.value().clone()))
        }
    pub(crate) async fn delete_business_metadata_internal(&self, asset_id: Uuid) -> Result<()> {
            self.business_metadata.remove(&asset_id);
            Ok(())
        }

}
