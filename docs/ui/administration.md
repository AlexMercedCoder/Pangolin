# Administration

Administrative tools in Pangolin are split between global system owners (Root) and individual team lead (Tenant Admins).

## 🏢 Root-Level Management

Root users have access to the **Root Dashboard** for cross-tenant operations.

### Tenant Management
- **Lifecycle**: Create, Update, and Delete tenants.
- **Isolation**: Each tenant represents a fully isolated environment with its own users and catalogs.

### System Settings
- Configure global properties like JWT secrets, allowed email domains for OAuth, and system-wide maintenance windows.

---

## 🏗️ Tenant-Level Management

Tenant Admins manage the resources within their specific project or organization.

### 1. Warehouse Management
Warehouses define the connection and authentication to your storage (S3, GCS, Azure, etc.).
- **Vending Strategy**: Choose between `AwsStatic`, `AwsSts`, `AzureSas`, or `GcpDownscoped` for credential management.
- **Connection Test**: Verify connectivity to your storage bucket directly from the creation form.

### 2. Catalog Management
Connect warehouses to your namespace hierarchies.
- **Internal Catalogs**: Managed by Pangolin's metadata store.
- **Federated Catalogs**: Proxy external REST catalogs (e.g., Tabular, Snowflake). The UI allows you to configure secret headers for these connections.

### 3. Service Users
For machine-to-machine integrations (CI/CD, internal tools).
- **API Keys**: Generate long-lived API keys for non-human users. Keys are displayed **once** upon creation or rotation.
- **Key Rotation**: Securely rotate keys to invalidate old credentials without deleting the user.
- **Scoped Identity**: Service users inherit specific roles just like regular users.

---

## 🛠️ UI Dashboard
The **Dashboard** tab provides quick metrics:
- **Tenant Health**: Total catalogs and active users.
- **Storage Metrics**: (Feature plan) Usage statistics per warehouse.
- **Recent Operations**: A subset of the audit logs showing recent management actions.
