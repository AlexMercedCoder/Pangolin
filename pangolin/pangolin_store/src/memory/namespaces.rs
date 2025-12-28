use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_namespace_internal(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
            let key = (tenant_id, catalog_name.to_string(), namespace.to_string());
            self.namespaces.insert(key, namespace);
            Ok(())
        }
    pub(crate) async fn list_namespaces_internal(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Namespace>> {
            let parent_prefix = parent.unwrap_or_default();
            tracing::info!("DEBUG_MEM: list_namespaces tid={} cat={} parent='{}'", tenant_id, catalog_name, parent_prefix);
            
            let iter = self.namespaces.iter()
                .filter(|entry| {
                    let (tid, cat, ns_str) = entry.key();
                    *tid == tenant_id && cat == catalog_name && (parent_prefix.is_empty() || ns_str.starts_with(&parent_prefix))
                })
                .map(|entry| entry.value().clone());

            let namespaces: Vec<Namespace> = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };

            tracing::info!("DEBUG_MEM: Found {} namespaces", namespaces.len());
            Ok(namespaces)
        }
    pub(crate) async fn get_namespace_internal(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
            let key = (tenant_id, catalog_name.to_string(), namespace.join("."));
            if let Some(n) = self.namespaces.get(&key) {
                Ok(Some(n.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn delete_namespace_internal(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
            let ns_str = namespace.join("\x1F");
            let key = (tenant_id, catalog_name.to_string(), ns_str);
            if self.namespaces.remove(&key).is_some() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Namespace not found"))
            }
        }
    pub(crate) async fn update_namespace_properties_internal(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
            let ns_str = namespace.join("\x1F");
            let key = (tenant_id, catalog_name.to_string(), ns_str);
        
            if let Some(mut ns) = self.namespaces.get_mut(&key) {
                ns.properties.extend(properties);
                Ok(())
            } else {
                 Err(anyhow::anyhow!("Namespace not found"))
            }
        }
    pub(crate) async fn count_namespaces_internal(&self, tenant_id: Uuid) -> Result<usize> {
        // Efficient counting for MemoryStore
        let count = self.namespaces.iter()
            .filter(|entry| entry.key().0 == tenant_id)
            .count();
        Ok(count)
    }

    pub(crate) async fn search_namespaces_internal(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> {
        let query = query.to_lowercase();
        let results = self.namespaces.iter()
            .filter(|entry| {
                entry.key().0 == tenant_id && 
                entry.value().name.join(".").to_lowercase().contains(&query)
            })
            .map(|entry| {
                let (_, catalog_name, _) = entry.key();
                (entry.value().clone(), catalog_name.clone())
            })
            .collect();
        Ok(results)
    }
}
