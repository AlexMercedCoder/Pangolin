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
        /// Optional tenant ID for tenant-scoped login
        #[arg(short = 't', long)]
        tenant_id: Option<String>,
    },
    /// Switch tenant context (Root operations only)
    Use {
        name: String
    },
    /// List all tenants (Root Only)
    ListTenants {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
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
    /// Update a tenant
    UpdateTenant {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: Option<String>,
    },
    // --- Users ---
    ListUsers {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
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
    /// Update a user
    UpdateUser {
        #[arg(long)]
        id: String,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        active: Option<bool>,
    },
    // --- Warehouses ---
    ListWarehouses {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
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
        #[arg(short = 'P', long = "property")]
        properties: Vec<String>,
    },
    DeleteWarehouse {
        name: String,
    },
    /// Update a warehouse
    UpdateWarehouse {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: Option<String>,
    },
    // --- Catalogs ---
    ListCatalogs {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    CreateCatalog {
        name: String,
        #[arg(long)]
        warehouse: String,
    },
    DeleteCatalog {
        name: String,
    },
    /// Update a catalog
    UpdateCatalog {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: Option<String>,
    },
    // --- Federated Catalogs ---
    /// Create a federated catalog with flexible properties
    CreateFederatedCatalog {
        /// Name of the federated catalog
        name: String,
        /// Storage location (required)
        #[arg(long)]
        storage_location: String,
        /// Configuration properties (key=value). E.g. -P uri=http://... -P token=...
        #[arg(short = 'P', long = "property")]
        properties: Vec<String>,
    },
    /// List all federated catalogs
    ListFederatedCatalogs {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
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
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
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
    // --- Service Users ---
    /// Create a new service user for API access
    CreateServiceUser {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value = "TenantUser")]
        role: String,
        #[arg(long)]
        expires_in_days: Option<i64>,
    },
    /// List all service users
    ListServiceUsers {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Get service user details
    GetServiceUser {
        #[arg(long)]
        id: String,
    },
    /// Update a service user
    UpdateServiceUser {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        active: Option<bool>,
    },
    /// Delete a service user
    DeleteServiceUser {
        #[arg(long)]
        id: String,
    },
    /// Rotate service user API key
    RotateServiceUserKey {
        #[arg(long)]
        id: String,
    },
    // --- Token Management ---
    /// Revoke your own token (logout)
    RevokeToken,
    /// Revoke a token by ID (admin only)
    RevokeTokenById {
        #[arg(long)]
        id: String,
    },
    /// List tokens for a specific user (admin only)
    ListUserTokens {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Delete a specific token (admin only)
    DeleteToken {
        #[arg(long)]
        token_id: String,
    },
    // --- System Configuration ---
    /// Get system settings (admin only)
    GetSystemSettings,
    /// Update system settings (admin only)
    UpdateSystemSettings {
        #[arg(long)]
        allow_public_signup: Option<bool>,
        #[arg(long)]
        default_warehouse_bucket: Option<String>,
        #[arg(long)]
        default_retention_days: Option<i32>,
    },
    // --- Federated Catalog Operations ---
    /// Trigger sync for a federated catalog
    SyncFederatedCatalog {
        name: String,
    },
    /// Get sync stats for a federated catalog
    GetFederatedStats {
        name: String,
    },
    // --- Data Explorer ---
    /// List namespace tree for a catalog
    ListNamespaceTree {
        catalog: String,
    },
    // --- Merge Operations ---
    /// List merge operations
    ListMergeOperations {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Get merge operation details
    GetMergeOperation {
        #[arg(long)]
        id: String,
    },
    /// List conflicts in a merge
    ListConflicts {
        #[arg(long)]
        merge_id: String,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Resolve a merge conflict
    ResolveConflict {
        #[arg(long)]
        merge_id: String,
        #[arg(long)]
        conflict_id: String,
        #[arg(long)]
        resolution: String,
    },
    /// Complete a merge operation
    CompleteMerge {
        #[arg(long)]
        id: String,
    },
    /// Abort a merge operation
    AbortMerge {
        #[arg(long)]
        id: String,
    },
    // --- Business Metadata ---
    /// Delete business metadata from an asset
    DeleteMetadata {
        #[arg(long)]
        asset_id: String,
    },
    /// Request access to an asset
    RequestAccess {
        #[arg(long)]
        asset_id: String,
        #[arg(long)]
        reason: String,
    },
    /// List all access requests
    ListAccessRequests {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Update an access request status
    UpdateAccessRequest {
        #[arg(long)]
        id: String,
        #[arg(long)]
        status: String,
    },
    /// Get asset details
    GetAssetDetails {
        #[arg(long)]
        id: String,
    },
    // --- Audit Logging ---
    /// List audit events with optional filtering
    ListAuditEvents {
        #[arg(long, help = "Filter by user ID")]
        user_id: Option<String>,
        #[arg(long, help = "Filter by action (e.g., create_table)")]
        action: Option<String>,
        #[arg(long, help = "Filter by resource type (e.g., table, catalog)")]
        resource_type: Option<String>,
        #[arg(long, help = "Filter by result (success or failure)")]
        result: Option<String>,
        #[arg(long, help = "Filter by tenant ID (Root only)")]
        tenant_id: Option<String>,
        #[arg(long, help = "Maximum number of results", default_value = "50")]
        limit: usize,
    },
    /// Count audit events with optional filtering
    CountAuditEvents {
        #[arg(long, help = "Filter by user ID")]
        user_id: Option<String>,
        #[arg(long, help = "Filter by action")]
        action: Option<String>,
        #[arg(long, help = "Filter by resource type")]
        resource_type: Option<String>,
        #[arg(long, help = "Filter by result")]
        result: Option<String>,
    },
    /// Get a specific audit event by ID
    GetAuditEvent {
        #[arg(long)]
        id: String,
    },
    // --- Optimization Commands ---
    /// Get dashboard statistics
    Stats,
    /// Get catalog summary
    CatalogSummary {
        /// Catalog name
        name: String,
    },
    /// Search for assets
    Search {
        /// Search query
        query: String,
        /// Filter by catalog
        #[arg(long)]
        catalog: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Bulk delete assets
    BulkDelete {
        /// Comma-separated list of asset UUIDs
        #[arg(long)]
        ids: String,
        /// Skip confirmation prompt
        #[arg(long)]
        confirm: bool,
    },
    /// Validate resource names
    Validate {
        /// Resource type (catalog, warehouse)
        #[arg(value_parser = ["catalog", "warehouse"])]
        resource_type: String,
        /// Names to validate
        names: Vec<String>,
    },
    /// Exit the REPL
    Exit,
    /// Clear the screen
    Clear,
}
