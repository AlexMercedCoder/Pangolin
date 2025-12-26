use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;
use pangolin_core::model::{
    Catalog, Namespace, Warehouse, Asset, Commit, Branch, Tag, Tenant,
    SystemSettings, SyncStats
};
use pangolin_core::user::User;
use pangolin_core::permission::{Role, UserRole, Permission};
use pangolin_core::audit::AuditLogEntry;

#[derive(Clone)]
pub struct MemoryStore {
    pub(crate) tenants: Arc<DashMap<Uuid, Tenant>>,
    pub(crate) warehouses: Arc<DashMap<(Uuid, String), Warehouse>>,
    pub(crate) catalogs: Arc<DashMap<(Uuid, String), Catalog>>,
    pub(crate) namespaces: Arc<DashMap<(Uuid, String, String), Namespace>>,
    pub(crate) assets: Arc<DashMap<(Uuid, String, String, String, String), Asset>>,
    pub(crate) branches: Arc<DashMap<(Uuid, String, String), Branch>>,
    pub(crate) tags: Arc<DashMap<(Uuid, String, String), Tag>>,
    pub(crate) commits: Arc<DashMap<(Uuid, Uuid), Commit>>,
    pub(crate) files: Arc<DashMap<String, Vec<u8>>>,
    pub(crate) audit_events: Arc<DashMap<Uuid, Vec<AuditLogEntry>>>,
    pub(crate) users: Arc<DashMap<Uuid, User>>,
    pub(crate) roles: Arc<DashMap<Uuid, Role>>,
    pub(crate) signer: crate::signer::SignerImpl,
    pub(crate) user_roles: Arc<DashMap<(Uuid, Uuid), UserRole>>,
    pub(crate) permissions: Arc<DashMap<Uuid, Permission>>,
    pub(crate) business_metadata: Arc<DashMap<Uuid, pangolin_core::business_metadata::BusinessMetadata>>,
    pub(crate) access_requests: Arc<DashMap<Uuid, pangolin_core::business_metadata::AccessRequest>>,
    pub(crate) service_users: Arc<DashMap<Uuid, pangolin_core::user::ServiceUser>>,
    pub(crate) merge_operations: Arc<DashMap<Uuid, pangolin_core::model::MergeOperation>>,
    pub(crate) merge_conflicts: Arc<DashMap<Uuid, pangolin_core::model::MergeConflict>>,
    pub(crate) assets_by_id: Arc<DashMap<Uuid, (String, Vec<String>, Option<String>, String)>>,
    pub(crate) revoked_tokens: Arc<DashMap<Uuid, pangolin_core::token::RevokedToken>>,
    pub(crate) active_tokens: Arc<DashMap<Uuid, pangolin_core::token::TokenInfo>>,
    pub(crate) system_settings: Arc<DashMap<Uuid, SystemSettings>>,
    pub(crate) federated_stats: Arc<DashMap<(Uuid, String), SyncStats>>,
    pub(crate) object_store_cache: crate::ObjectStoreCache,
    pub(crate) metadata_cache: crate::MetadataCache,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(DashMap::new()),
            warehouses: Arc::new(DashMap::new()),
            catalogs: Arc::new(DashMap::new()),
            namespaces: Arc::new(DashMap::new()),
            assets: Arc::new(DashMap::new()),
            branches: Arc::new(DashMap::new()),
            tags: Arc::new(DashMap::new()),
            commits: Arc::new(DashMap::new()),
            files: Arc::new(DashMap::new()),
            audit_events: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
            roles: Arc::new(DashMap::new()),
            signer: crate::signer::SignerImpl::new("memory_key".to_string()),
            user_roles: Arc::new(DashMap::new()),
            permissions: Arc::new(DashMap::new()),
            business_metadata: Arc::new(DashMap::new()),
            access_requests: Arc::new(DashMap::new()),
            service_users: Arc::new(DashMap::new()),
            merge_operations: Arc::new(DashMap::new()),
            merge_conflicts: Arc::new(DashMap::new()),
            assets_by_id: Arc::new(DashMap::new()),
            revoked_tokens: Arc::new(DashMap::new()),
            active_tokens: Arc::new(DashMap::new()),
            system_settings: Arc::new(DashMap::new()),
            federated_stats: Arc::new(DashMap::new()),
            object_store_cache: crate::ObjectStoreCache::new(),
            metadata_cache: crate::MetadataCache::default(),
        }
    }
}
