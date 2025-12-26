// SQLite submodules
mod main;
pub mod tenants;
pub mod warehouses;
pub mod catalogs;
pub mod namespaces;
pub mod assets;
pub mod branches;
pub mod tags;
pub mod commits;
pub mod users;
pub mod roles;
pub mod permissions;
pub mod tokens;
pub mod audit_logs;
pub mod system_settings;
pub mod federated_catalogs;
pub mod maintenance;
pub mod io;
pub mod access_requests;
pub mod signer;
pub mod service_users;
pub mod merge_operations;

// Re-export SqliteStore
pub use main::SqliteStore;
pub mod business_metadata;
