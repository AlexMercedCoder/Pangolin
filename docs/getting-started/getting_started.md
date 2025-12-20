# Getting Started with Pangolin

This guide will walk you through setting up Pangolin and performing an end-to-end workflow.

## 🔐 Authentication Modes

Pangolin supports two primary operational modes: **No-Auth Mode** and **Auth Mode**.

### 1. No-Auth Mode (`PANGOLIN_NO_AUTH=true`)
Designed for rapid local evaluation and testing.
- **Single Tenant**: All operations are automatically tied to a fixed "default" tenant.
- **No Headers Required**: You don't need `Authorization` or `X-Pangolin-Tenant` headers.
- **Anonymous Access**: Direct access to all catalog features.
- **Limitations**: Only one tenant allowed; no RBAC enforcement.

### 2. Auth Mode (`PANGOLIN_NO_AUTH=false`)
The standard mode for multi-tenant production environments.
- **Tenant Isolation**: Every request MUST include a valid `Authorization: Bearer <JWT>` token.
- **RBAC Enforcement**: Permissions are checked at the Catalog, Namespace, and Asset levels.
- **Tenant Context**: Users only see resources belonging to their specific tenant.
- **Multiple Tenants**: Supports infinite logical isolation across different organizations.

---

## 👥 User Scopes

Pangolin uses a three-tier identity model to balance platform oversight with organizational autonomy.

| Scope | Typical ID | Key Responsibilities | Focus |
| :--- | :--- | :--- | :--- |
| **Root User** | `admin` | Create Tenants, Configure Global Settings, System Metrics. | Platform |
| **Tenant Admin** | Organization Lead | Manage Users, Warehouses, Catalogs, Audit Logs. | Governance |
| **Tenant User** | Analyst / Engineer | Create Namespaces, Manage Tables, Branching, Tagging. | Data |

### Role Details:
- **Root**: Has "god mode" over the platform but cannot see into tenant table data directly unless granted a role.
- **Tenant Admin**: Controls everything *within* their tenant. They set up the warehouses (S3/Azure) and the catalogs.
- **Tenant User**: Restricted to the data assets. They cannot manage other users or infrastructure.

---

## 🏗️ Installation & Setup

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/your-repo/pangolin.git
    cd pangolin
    ```

2.  **Run the Server**
    
    **For Testing (NO_AUTH mode):**
    ```bash
    PANGOLIN_NO_AUTH=true cargo run --bin pangolin_api
    ```
    
    **For Production:**
    ```bash
    cargo run --bin pangolin_api
    ```
    
    The server will start at `http://localhost:8080`.

## End-to-End Walkthrough

We will simulate a real-world scenario: A "Data Team" setting up their environment, creating tables, experimenting on a branch, and merging changes back.

### 1. Create a Tenant
Pangolin is multi-tenant by default. First, create a tenant for your organization.

```bash
curl -X POST http://localhost:8080/api/v1/tenants \
  -u admin:password \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corp",
    "id": "00000000-0000-0000-0000-000000000001"
  }'
```
*Note: We are using a fixed UUID for simplicity. The `-u admin:password` corresponds to the default Root User credentials in `example.env`.*

### 2. Create a Warehouse
A warehouse stores storage credentials and configuration for **Iceberg table data**.

> **Storage Clarification**:
> - **Catalog Metadata Storage** (controlled by `PANGOLIN_STORAGE_TYPE` env var): Where Pangolin stores its catalog metadata (tenants, catalogs, branches, etc.)
> - **Table Data Storage** (controlled by warehouse config): Where Iceberg table data files are stored
> 
> These are separate! For example, you can store catalog metadata in Postgres while table data lives in S3.

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "main_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "s3",
      "bucket": "demo-bucket",
      "region": "us-east-1"
    }
  }'
```
*Note: We use `use_sts: false` for this quick start. For production with STS credential vending, see [Warehouse Management](../features/../features/warehouse_management.md).*

### 3. Create a Catalog
A catalog references a warehouse and specifies a storage location. The catalog name is what clients use in their connection URIs.

```bash
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "analytics",
    "warehouse_name": "main_warehouse",
    "storage_location": "s3://demo-bucket/analytics"
  }'
```

### 4. Create a Namespace
Namespaces organize your tables (like schemas in a database).

```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{"namespace": ["data_team"]}'
```
*Note: The URL uses the catalog name `analytics` as the prefix.*

### 5. Create a Table on `main`
Create an Iceberg table in the `data_team` namespace. By default, operations target the `main` branch.

```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces/data_team/tables \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "users",
    "location": "s3://demo-bucket/analytics/data_team/users",
    "schema": {
      "type": "struct",
      "fields": [
        {"id": 1, "name": "id", "type": "int", "required": true},
        {"id": 2, "name": "name", "type": "string", "required": false}
      ]
    }
  }'
```

### 6. Create a Feature Branch (`dev`)
Create a new branch named `dev` to experiment safely. We will use **Partial Branching** to only track the `users` table on this branch.

```bash
curl -X POST http://localhost:8080/api/v1/branches \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dev",
    "catalog": "analytics",
    "from_branch": "main",
    "branch_type": "experimental",
    "assets": ["data_team.users"]
  }'
```

### 7. Update Table on `dev`
Modify the table schema on the `dev` branch. Note the `?branch=dev` query parameter.

```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces/data_team/tables/users?branch=dev \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "updates": [
      {"action": "add-column", "name": "email", "type": "string"}
    ]
  }'
```

### 8. Verify Isolation
Check that the `main` branch is unaffected.

```bash
# Check main (should NOT have email column)
curl http://localhost:8080/v1/analytics/namespaces/data_team/tables/users?branch=main \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001"

# Check dev (SHOULD have email column)
curl http://localhost:8080/v1/analytics/namespaces/data_team/tables/users?branch=dev \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001"
```

### 9. Merge `dev` into `main`
Once satisfied, merge the changes back to `main`.

```bash
curl -X POST http://localhost:8080/api/v1/branches/merge \
  -H "X-Pangolin-Tenant: 00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: application/json" \
  -d '{
    "catalog": "analytics",
    "source_branch": "dev",
    "target_branch": "main"
  }'
```

Now, `main` will contain the schema updates made in `dev`.

## Next Steps

- Explore [Branch Management](../features/branch_management.md) for advanced strategies.
- Configure [S3 Storage](../warehouse/s3.md) for production data.
- Learn about [Warehouse Management](../features/warehouse_management.md) and credential vending.
- Set up [Client Configuration](client_configuration.md) for PyIceberg, PySpark, Trino, or Dremio.

## Production Setup

For production deployments, disable NO_AUTH mode and use Bearer token authentication:

### 1. Start Server (No NO_AUTH variable)

```bash
# Set JWT secret for production
export PANGOLIN_JWT_SECRET="your-secret-key-here"
cargo run --release --bin pangolin_api
```

### 2. Generate Tokens

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "username": "user@example.com",
    "expires_in_hours": 24
  }'
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "expires_at": "2025-12-14T02:37:49+00:00",
  "tenant_id": "00000000-0000-0000-0000-000000000001"
}
```

### 3. Use Token with PyIceberg

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",  # From /api/v1/tokens
    }
)
```

### 4. Tenant Creation Restriction

In NO_AUTH mode, attempting to create additional tenants will fail:

```bash
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "test-tenant"}'
```

Response (403 Forbidden):
```json
{
  "error": "Cannot create additional tenants in NO_AUTH mode",
  "message": "NO_AUTH mode is only meant for evaluation and testing with a single default tenant. Please enable authentication and use Bearer tokens if you want to create multiple tenants.",
  "hint": "Remove PANGOLIN_NO_AUTH environment variable and use /api/v1/tokens endpoint to generate tokens"
}
```

## Next Steps

- [PyIceberg Testing Guide](../features/pyiceberg_testing.md) - Comprehensive PyIceberg testing
- [Client Configuration](./client_configuration.md) - Configure various Iceberg clients
- [Warehouse Management](./../features/warehouse_management.md) - Manage warehouses and catalogs
- [API Reference](../api/) - Complete API documentation
