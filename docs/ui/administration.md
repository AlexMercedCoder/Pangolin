# Administration

Pangolin provides dedicated interfaces for both **Root** and **Tenant** administrators to manage the platform's resources.

## Root Administration
*Only available to the system owner.*

### Tenant Management
- **List Tenants**: View all active tenants in the system.
- **Create Tenant**: Provision new isolated environments.
- **Delete Tenant**: Deprovision environments (requires confirmation).

## Tenant Administration
*Available to users with `TenantAdmin` role.*

### 1. User Management
Manage your team's access to the lakehouse.
- **Create User**: Add new users with specific roles.
- **Assign Roles**: Update existing user permissions.
- **Revoke Access**: Deactivate or delete users.

### 2. Warehouse Configuration
Connect Pangolin to your cloud object storage.
- **Supported Providers**: AWS S3, Google Cloud Storage (GCS), Azure Blob Storage, and local filesystem (MinIO).
- **Credential Vending**: Configure automatic token vending for compliant Iceberg clients.

### 3. Catalog Management
Create and link Iceberg catalogs to warehouses.
- **Internal Catalogs**: Managed directly by Pangolin.
- **Federated Catalogs**: Connect to external REST catalogs (e.g., Snowflake, Tabular, Databricks) and proxy traffic through Pangolin.

### 4. RBAC Configuration
Define granular access policies. [Learn more about RBAC UI](./rbac.md).
