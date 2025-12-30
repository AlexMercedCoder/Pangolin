# Tenants Reference

Tenants are the top-level isolation unit in Pangolin. Each tenant has its own set of users, catalogs, and resources.

## 1. API

**Base Endpoint**: `/api/v1/tenants`

### List Tenants
*   **Method**: `GET`
*   **Path**: `/api/v1/tenants`
*   **Auth**: Root Access Token
*   **Parameters**:
    *   `limit` (query, optional): Max results
    *   `offset` (query, optional): Pagination offset

```bash
curl -X GET http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer <root-token>"
```

### Create Tenant
*   **Method**: `POST`
*   **Path**: `/api/v1/tenants`
*   **Auth**: Root Access Token
*   **Body**:
    ```json
    {
      "name": "acme-corp"
    }
    ```

```bash
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer <root-token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "acme-corp"}'
```

### Initial Administrator
When a tenant is created, you usually create a `tenant-admin` user immediately after, or use the CLI which can do both.

---

## 2. CLI

The `pangolin-admin` CLI provides convenient commands for tenant management.

### List Tenants
```bash
pangolin-admin list-tenants
```

### Create Tenant
You can create a tenant and an optional admin user in one go.

```bash
# Create tenant only
pangolin-admin create-tenant --name "acme-corp"

# Create tenant AND admin user
pangolin-admin create-tenant \
  --name "acme-corp" \
  --admin-username "admin" \
  --admin-password "securePass123"
```

### Switch Context
To perform operations on a specific tenant using the CLI:

```bash
pangolin-admin use "acme-corp"
```
This sets the context for subsequent commands (like creating users or warehouses).

---

## 3. Python SDK (`pypangolin`)

### List Tenants
```python
from pypangolin import PangolinClient

client = PangolinClient(uri="http://localhost:8080", username="root", password="root_password")

tenants = client.tenants.list()
for t in tenants:
    print(t.name, t.id)
```

### Create Tenant
```python
new_tenant = client.tenants.create(name="acme-corp")
print(f"Created tenant: {new_tenant.id}")
```

### Switch Context
```python
# Switch client context to specific tenant
client.tenants.switch("acme-corp")

# Subsequent calls now apply to acme-corp
client.users.list() 
```

---

## 4. UI

1.  **Log in** as the **Root User**.
2.  Navigate to **Tenants** in the sidebar.
3.  **List**: You will see a table of all existing tenants.
4.  **Create**: Click the **"Create Tenant"** button.
    *   Enter the Tenant Name.
    *   (Optional) Enter Admin Username/Password to auto-provision an admin.
    *   Click **Save**.
5.  **Switch Context**: Click the **"Switch"** (or log-in) icon next to a tenant to switch your active session to that tenant.
