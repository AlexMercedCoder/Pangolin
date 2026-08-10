// SQLite submodules
pub mod access_requests;
pub mod assets;
pub mod audit_logs;
pub mod branches;
pub mod catalogs;
pub mod commits;
pub mod federated_catalogs;
pub mod io;
mod main;
pub mod maintenance;
pub mod merge_operations;
pub mod namespaces;
pub mod permissions;
pub mod roles;
pub mod service_users;
pub mod signer;
pub mod system_settings;
pub mod tags;
pub mod tenants;
pub mod tokens;
pub mod users;
pub mod warehouses;

// Re-export SqliteStore
pub use main::{SqliteStore, SQLITE_SCHEMA_VERSION};
pub mod business_metadata;
