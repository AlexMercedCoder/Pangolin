use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pangolin-admin")]
#[command(multicall = true)]
pub struct AdminCli {
    #[command(subcommand)]
    pub command: AdminCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AdminCommand {
    /// Login to the Pangolin server
    Login {
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Switch tenant context (Root operations only)
    Use {
        name: String
    },
    /// List all tenants (Root Only)
    ListTenants,
    /// Create a new tenant (Root Only)
    CreateTenant {
        /// Name of the tenant
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        admin_username: Option<String>,
        #[arg(long)]
        admin_password: Option<String>,
    },
    /// Delete a tenant
    DeleteTenant {
        id: String,
    },
    // --- Users ---
    ListUsers,
    CreateUser {
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        role: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        tenant_id: Option<String>
    },
    DeleteUser {
        username: String,
    },
    // --- Warehouses ---
    ListWarehouses,
    CreateWarehouse {
        name: String,
        #[arg(long, default_value = "s3")]
        type_: String,
        #[arg(long)]
        bucket: Option<String>,
        #[arg(long)]
        access_key: Option<String>,
        #[arg(long)]
        secret_key: Option<String>,
        #[arg(long)]
        region: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
    },
    DeleteWarehouse {
        name: String,
    },
    // --- Catalogs ---
    ListCatalogs,
    CreateCatalog {
        name: String,
        #[arg(long)]
        warehouse: String,
    },
    DeleteCatalog {
        name: String,
    },
    // --- Federated Catalogs ---
    /// Create a federated catalog that proxies to another Iceberg REST catalog
    CreateFederatedCatalog {
        /// Name of the federated catalog
        name: String,
        /// Base URL of the remote catalog (e.g., http://remote:8080/v1/catalog_name)
        #[arg(long)]
        base_url: String,
        /// Storage location (required even for federated catalogs)
        #[arg(long)]
        storage_location: String,
        /// Authentication type: None, BasicAuth, BearerToken, ApiKey
        #[arg(long, default_value = "None")]
        auth_type: String,
        /// Bearer token (required if auth_type is BearerToken)
        #[arg(long)]
        token: Option<String>,
        /// Username (required if auth_type is BasicAuth)
        #[arg(long)]
        username: Option<String>,
        /// Password (required if auth_type is BasicAuth)
        #[arg(long)]
        password: Option<String>,
        /// API key (required if auth_type is ApiKey)
        #[arg(long)]
        api_key: Option<String>,
        /// Timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u32,
    },
    /// List all federated catalogs
    ListFederatedCatalogs,
    /// Delete a federated catalog
    DeleteFederatedCatalog {
        name: String,
    },
    /// Test connectivity to a federated catalog
    TestFederatedCatalog {
        name: String,
    },
    // --- Governance: Permissions ---
    ListPermissions {
        #[arg(long)]
        role: Option<String>,
        #[arg(long)]
        user: Option<String>,
    },
    GrantPermission {
        username: String,
        action: String,
        resource: String,
    },
    RevokePermission {
         role: String,
         action: String,
         resource: String,
    },
    // --- Governance: Metadata ---
    GetMetadata {
        #[arg(short, long)]
        entity_type: String, // catalog, warehouse, etc.
        #[arg(short, long)]
        entity_id: String,
    },
    SetMetadata {
        #[arg(short, long)]
        entity_type: String,
        #[arg(short, long)]
        entity_id: String,
        key: String,
        value: String,
    },
    /// Exit the REPL
    Exit,
    /// Clear the screen
    Clear,
}
