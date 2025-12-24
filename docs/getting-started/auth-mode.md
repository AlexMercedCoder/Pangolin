
# Pangolin Authentication Guide

This guide details how authentication works in Pangolin when the system is running in standard production mode (Auth Mode).

## 1. The Root User

The **Root User** is the super-administrator of the entire Pangolin system. This user exists outside of any specific tenant.

- **Scope**: System-wide. No specific `tenant_id`.
- **Capabilities**:
  - Create and delete Tenants.
  - Create the initial Tenant Admin for a Tenant.
  - View global system statistics (e.g., total tenants, total tables).
- **Limitations**:
  - Cannot query data within a Tenant's catalogs unless explicitly granted permission (conceptually separate).
  - Cannot be assigned to a specific Tenant.

**Configuration**:
You can define the initial credentials for the Root User using environment variables. This is critical for securing your production deployment.

```bash
# Default values if not specified: admin / password
export PANGOLIN_ROOT_USER="super_admin"
export PANGOLIN_ROOT_PASSWORD="complex_secure_password"
```

**Login Behavior**:
- **API**: Login with `username` and `password`, leaving `tenant_id` null.
- **UI**: Use the dedicated Root Login toggle or route (e.g., `/login` with "Root" checked).

## 2. Authentication Flows

### Generating Tokens (API/CLI)

Authentication tokens (JWTs) act as your digital keys.

**For Tenant Admins & Users**:
You must provide the `tenant_id` along with credentials.

```bash
# Example Request
POST /api/v1/users/login
{
  "username": "data_analyst",
  "password": "secure_password",
  "tenant_id": "123e4567-e89b-12d3..."
}

# Response
{
  "token": "eyJhbGciOiJIUz...",
  "expires_in": 86400
}
```

**For Root**:
Omit the `tenant_id`.

### Logging into the UI

The UI login page adapts based on the user type:

1.  **Tenant Login**:
    - **Username**: Your username.
    - **Password**: Your password.
    - **Tenant ID**: The UUID of your organization/tenant.
2.  **Root Login**:
    - Click "Login as System Root".
    - Enter only Username and Password.

## 3. Testing with PyIceberg

When running in Auth Mode, PyIceberg requires a valid token and the tenant context.

```python
from pyiceberg.catalog import load_catalog

# 1. Obtain a token (e.g., via script or manually from UI)
token = "YOUR_GENERATED_JWT_TOKEN"
tenant_id = "YOUR_TENANT_UUID"

# 2. Configure PyIceberg
catalog = load_catalog(
    "production",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/api/v1/catalogs/sales/iceberg",
        "token": token,
        # Required for Multi-tenancy routing
        "header.X-Pangolin-Tenant": tenant_id
    }
)
```

## 4. Permissions Matrix

Pangolin uses a hierarchical RBAC system.

### Roles

| Role | Scope | Description |
| :--- | :--- | :--- |
| **Root** | System | Manages Tenants. Cannot manage data inside a tenant directly without assuming a tenant context (if allowed). |
| **Tenant Admin** | Tenant | Full control over a specific Tenant's resources (Users, Catalogs, Roles). |
| **Tenant User** | Tenant | Zero access by default. Must be granted specific permissions. |

### Permissions Granting

Permissions are additive. A user can be granted specific Actions on specific Scopes.

| Scope | Description | Implied Children |
| :--- | :--- | :--- |
| **Tenant** | Entire Tenant | All Catalogs, Namespaces, Tables |
| **Catalog** | Specific Catalog | All Namespaces, Tables within |
| **Namespace** | Specific Namespace | All Tables within |
| **Table/Asset** | Specific Table | None |

| Action | Description |
| :--- | :--- |
| **Read** | Read metadata and data. |
| **Write** | Insert, update, delete data. |
| **Create** | Create new resources (Tables, Namespaces). |
| **Delete** | Drop resources. |
| **List** | See that the resource exists (Discovery). |
| **ManageDiscovery** | Mark assets as strictly discoverable (metadata visible, data locked). |

### Common Scenarios

- **Data Analyst**: Grant `Read` on specific `Namespace`.
- **Data Engineer**: Grant `Read`, `Write`, `Create` on specific `Catalog`.
- **Auditor**: Grant `List` (Discovery) on `Tenant` + `Read` on `audit_logs` (conceptually).
